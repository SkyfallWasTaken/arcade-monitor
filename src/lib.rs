use indoc::formatdoc;
use items::ShopItems;
use serde_json::json;
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
    let webhook_url = env.var("SLACK_WEBHOOK_URL")?.to_string();
    let kv = env.kv("SHOP_ITEMS")?;

    let available_items = items::try_fetch(shop_url).await?;
    let Some(old_items) = kv.get("items").json::<items::ShopItems>().await? else {
        console_debug!("No old items found, storing new items");
        kv.put("items", &available_items)?.execute().await?;
        return Ok("No old items found, storing new items".into());
    };

    // Compare the old items with the new items.
    let result = diff_old_new_items(&old_items, &available_items);

    // If there are any updates/new items, send a message to the Slack webhook.
    if result.is_empty() {
        return Ok("No changes detected".into());
    } else {
        let message = formatdoc! {
            "*Changes detected in the shop:*
            {changes}",
            changes = result.join("\n\n"),
        };

        let request = Request::new_with_init(
            &webhook_url,
            RequestInit::new()
                .with_body(Some(serde_wasm_bindgen::to_value(
                    &json!({ "text": message }),
                )?))
                .with_method(Method::Post),
        )?;

        Fetch::Request(request).send().await?;
    }

    // Now, let's persist the items to the KV store.
    kv.put("items", &available_items)?.execute().await?;

    Ok(result.join("\n\n"))
}

fn diff_old_new_items(old_items: &ShopItems, new_items: &ShopItems) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for item in new_items {
        // TODO: not very efficient.
        let old_item = old_items.iter().find(|i| i.id == item.id);

        match old_item {
            Some(old) => {
                if let Some(diff) = format::format_item_diff(old, item) {
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
    use indoc::indoc;
    use items::ShopItem;

    #[test]
    fn deleted_items_notification() {
        let item_1 = ShopItem {
            full_name: "Item 1".into(),
            description: Some("Description 1".into()),
            id: "1".into(),
            ..Default::default()
        };
        let item_2 = ShopItem {
            full_name: "Item 2".into(),
            description: Some("Description 2".into()),
            id: "2".into(),
            ..Default::default()
        };

        let old_items = vec![item_1.clone(), item_2.clone()];
        let new_items = vec![item_1.clone()];

        let result = diff_old_new_items(&old_items, &new_items);

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            formatdoc! {
                "*Item DELETED:* Item 2
                *Description:* Description 2
                *Price:* 200"
            }
        );
    }
}
