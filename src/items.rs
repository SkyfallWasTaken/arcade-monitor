use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Serialize, Deserialize, PartialEq, Eq, Default, Clone)]
pub struct ShopItem {
    #[serde(rename = "Full Name")]
    pub full_name: String,

    #[serde(rename = "Description")]
    pub description: Option<String>,

    #[serde(rename = "Fulfillment Description")]
    pub fulfillment_description: Option<String>,

    #[serde(rename = "Cost Hours")]
    pub price: i32,

    #[serde(rename = "Stock")]
    pub stock: Option<i32>,

    pub id: String,
}

pub type ShopItems = Vec<ShopItem>;

const USER_AGENT: &str = "Arcade-Monitor/1.0 (+@SkyfallWasTaken)";

pub async fn try_fetch(shop_url: Url) -> Result<ShopItems> {
    let client = Client::new();
    let response = client
        .get(shop_url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await?;
    let doc_html = response.text().await?;

    let doc = Html::parse_document(&doc_html);
    let selector = Selector::parse("#__NEXT_DATA__").unwrap();
    let json: serde_json::Value = serde_json::from_str(
        &doc.select(&selector)
            .next()
            .ok_or("no #__NEXT_DATA__ element in document")?
            .inner_html(),
    )?;

    let available_items = serde_json::from_value(
        json.pointer("/props/pageProps/availableItems")
            .ok_or("availableItems not found - is the ARCADE_SHOP_URL correct?")?
            .clone(),
    )?;

    Ok(available_items)
}
