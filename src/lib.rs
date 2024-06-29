use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use worker::*;

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

#[derive(Serialize, Deserialize)]
struct ShopItem {
    #[serde(rename = "Full Name")]
    full_name: String,

    #[serde(rename = "Description")]
    description: Option<String>,

    #[serde(rename = "Fulfillment Description")]
    fulfillment_description: Option<String>,

    #[serde(rename = "Cost Hours")]
    price: i32,
}

async fn run_scrape(env: Env) -> Result<String> {
    let shop_url = Url::parse(&env.var("ARCADE_SHOP_URL")?.to_string())?;
    let mut response = Fetch::Url(shop_url).send().await?;

    let doc_html = response.text().await?;
    let doc = Html::parse_document(&doc_html);
    let selector = Selector::parse("#__NEXT_DATA__").unwrap();
    let json: serde_json::Value =
        serde_json::from_str(&doc.select(&selector).next().unwrap().inner_html())?;

    let available_items: Vec<ShopItem> = serde_json::from_value(
        json.pointer("/props/pageProps/availableItems")
            .expect("availableItems not found - is the ARCADE_SHOP_URL correct?")
            .clone(),
    )?;

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
