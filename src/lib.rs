use indoc::formatdoc;
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
    let mut result: Vec<String> = Vec::new();
    for item in &available_items {
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
