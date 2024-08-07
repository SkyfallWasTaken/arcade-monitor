use std::collections::HashMap;

use items::ShopItems;
use reqwest::Client;
use worker::*;

mod format;
mod items;

#[event(fetch, respond_with_errors)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    console_error_panic_hook::set_once();
    let router = Router::new();

    router
        .on_async("/", |_req, ctx| async move {
            let body = run_scrape(ctx.env).await?;
            Response::ok(body)
        })
        .on_async("/repo", |_req, _ctx| async move {
            Response::redirect(
                Url::parse("https://github.com/SkyfallWasTaken/arcade-monitor").unwrap(),
            )
        })
        .run(req, env)
        .await
}

#[event(scheduled)]
pub async fn scheduled(event: ScheduledEvent, env: Env, _ctx: ScheduleContext) {
    run_scrape(env)
        .await
        .unwrap_or_else(|_| panic!("failed to run scheduled scrape: {}", event.schedule()));
}

async fn run_scrape(env: Env) -> Result<String> {
    let shop_url = Url::parse(&env.var("ARCADE_SHOP_URL")?.to_string())?;
    let slack_webhook_url = env.secret("SLACK_WEBHOOK_URL")?.to_string();
    let ntfy_url = env.var("NTFY_URL")?.to_string();
    let ntfy_auth_token = env.secret("NTFY_AUTH_TOKEN")?.to_string();
    let slack_group_id = env.var("SLACK_GROUP_ID")?.to_string();

    let client = Client::new();

    let kv = env.kv("SHOP_ITEMS")?;

    let available_items = items::try_fetch(shop_url).await?;
    let Some(old_items) = kv.get("items").json::<items::ShopItems>().await? else {
        console_debug!("No old items found, storing new items");
        kv.put("items", &available_items)?.execute().await?;
        return Ok("No old items found, storing new items".into());
    };
    let Some(real_prices) = kv.get("real_prices").json::<HashMap<String, i32>>().await? else {
        console_debug!("No real prices found!");
        return Err("No real prices found! This is a bug.".into());
    };

    // Compare the old items with the new items.
    let result = diff_old_new_items(&old_items, &available_items, real_prices);

    // Check if there are any updates.
    if result.is_empty() {
        console_debug!("No changes detected");
        return Ok("No changes detected".into());
    }

    // If there are any updates/new items, send a message to the Slack webhook.
    let changes = result.join("\n\n");

    // slack webhook
    let slack_body = format::get_slack_body(&result, slack_group_id);
    client
        .post(&slack_webhook_url)
        .body(slack_body.to_string())
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // ntfy webhook
    client
        .post(ntfy_url)
        .bearer_auth(ntfy_auth_token)
        .header("X-Title", "Changes detected in Arcade Shop")
        .header("X-Priority", "high")
        .body(changes)
        .send()
        .await
        .unwrap();

    // Now, let's persist the items to the KV store.
    kv.put("items", &available_items)?.execute().await?;

    Ok(result.join("\n\n"))
}

fn diff_old_new_items(
    old_items: &ShopItems,
    new_items: &ShopItems,
    real_prices: HashMap<String, i32>,
) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for item in new_items {
        // TODO: not very efficient.
        let old_item = old_items.iter().find(|i| i.id == item.id);

        match old_item {
            Some(old) => {
                if let Some(diff) = format::format_item_diff(old, item, real_prices.get(&item.id)) {
                    result.push(diff);
                }
            }
            None => {
                result.push(format::format_new_item(item));
            }
        }
    }

    // Check if any items have been removed.
    for item in old_items {
        if !new_items.iter().any(|i| i.id == item.id) {
            result.push(format::format_deleted_item(item));
        }
    }

    result
}

#[cfg(test)]
mod diff_old_new_items_tests {
    use super::*;

    use indoc::formatdoc;
    use items::ShopItem;
    use maplit::hashmap;
    use pretty_assertions::assert_eq;

    #[test]
    fn deleted_items_notification() {
        let item_1 = ShopItem {
            full_name: "Item 1".into(),
            description: Some("Description 1".into()),
            price: 200,
            id: "1".into(),
            ..Default::default()
        };
        let item_2 = ShopItem {
            full_name: "Item 2".into(),
            description: Some("Description 2".into()),
            price: 50,
            id: "2".into(),
            ..Default::default()
        };

        let old_items = vec![item_1.clone(), item_2.clone()];
        let new_items = vec![item_1.clone()];

        let result = diff_old_new_items(
            &old_items,
            &new_items,
            hashmap! {
                "1".into() => 100,
                "2".into() => 200,
            },
        );

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            formatdoc! {
                "*Item DELETED:* {full_name}
                *Description:* {description}
                *Price:* {price}",
                full_name = item_2.full_name,
                description = item_2.description.as_ref().unwrap_or(&"_not set_".into()),
                price = item_2.price,
            }
        );
    }
}
