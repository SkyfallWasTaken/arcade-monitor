use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use worker::*;

#[event(fetch, respond_with_errors)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let router = Router::new();

    router
        .on_async("/", |_req, ctx| async move {
            let body = "Hello, World!";

            run_scrape(ctx.env).await?;

            Response::ok(body)
        })
        .run(req, env)
        .await
}

#[derive(Serialize, Deserialize)]
struct ShopItem {
    #[serde(rename = "Name")]
    name: String,

    #[serde(rename = "Small Name")]
    small_name: Option<String>,

    #[serde(rename = "Description")]
    description: Option<String>,

    #[serde(rename = "Fulfillment Description")]
    fulfillment_description: Option<String>,

    #[serde(rename = "Cost Hours")]
    price: i32,
}

async fn run_scrape(env: Env) -> Result<()> {
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

    for item in available_items {
        let item_name = item.small_name.unwrap_or(item.name.clone());
        let item_description = item
            .description
            .unwrap_or(item.fulfillment_description.unwrap_or("".to_string()));
        let item_price = item.price;

        let message = format!("{}: {} - ${}", item_name, item_description, item_price);
        console_debug!("{message}");
    }

    Ok(())
}
