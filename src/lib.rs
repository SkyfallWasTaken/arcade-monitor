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

async fn run_scrape(env: Env) -> Result<String> {
    let shop_url = Url::parse(&env.var("ARCADE_SHOP_URL")?.to_string())?;
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
                result.push(format::format_new_item(&item));
            }
        }
    }

    // Now, let's persist the items to the KV store.
    kv.put("items", &available_items)?.execute().await?;

    Ok(result.join("\n\n"))
}
