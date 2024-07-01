use worker::*;

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

    let available_items = items::try_fetch(shop_url).await?;
    let mut result = Vec::new();
    for item in available_items {
        result.push(format!(
            "`{full_name}` - {price} {}",
            if item.price == 1 { "ticket" } else { "tickets" },
            full_name = item.full_name.trim(),
            price = item.price,
        ));
    }

    Ok(result.join("\n"))
}
