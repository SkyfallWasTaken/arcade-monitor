# Arcade Monitor: HC's beloved shop ~stalker~ monitor

<img src="https://github.com/user-attachments/assets/3fe6199a-7d82-4620-ad40-9ab2c10044da" align="left" width="246" style="padding-right: 3rem"/>

Arcade Monitor was a **monitor for the Hack Club Arcade Shop.** It continuously checked for new items, stock changes, and item updates, keeping you in the loop on the stuff you want. At its peak, it chattered away in the [#arcade-monitor channel](https://hackclub.slack.com/archives/C079RG9HJ81) to over 400 members across Slack and ntfy.

This has been one of my most successful projects: I managed to get 400 active users of the channel, including many from Hack Club HQ, got third-party contributions to my project for the first time, and managed to nearly 60 stars, by far my most starred project on GitHub _(will you be the next stargazer? 😉)_

If I were to make this project again, I'd probably port it to more platforms, such as Discord webhooks, in order to make it possible for people to use the software they want to use, rather than being forced to use Slack or ntfy :)

Arcade Monitor is also fully open source, and licensed under the [MIT License!](https://github.com/SkyfallWasTaken/arcade-monitor/blob/main/LICENSE.md)

The rest of the readme has been archived for posterity. Thank you to everyone who checked out Arcade Monitor! 🫡

---

## Usage on Slack
_(note: you'll need to be a member of the [Hack Club Slack](https://hackclub.com/slack/) for this!)_

Using the bot with Slack is really easy! Just join the [#arcade-monitor channel](https://hackclub.slack.com/archives/C079RG9HJ81), and you'll be added to the Arcade Monitor user group. You'll now be pinged for every update!

## Usage with ntfy

You'll need [the ntfy app](https://ntfy.sh) for this!

Once you have the app, set the server URL to `https://ntfy.hackclub.app`, and the topic to `arcade-monitor`.

That's it! You'll now receive a notification for every shop update.

## A history lesson (if you're into that sort of thing :))
<details>
<summary>Click for more details...</summary>
	
### The first version

The first version of Arcade Monitor was a Cloudflare Worker written in TypeScript. In hindsight, a lot of the code was needlessly complicated, and the bot was _severely_ buggy. Take a look at this:

![image](https://github.com/user-attachments/assets/22320356-c8a9-418e-b799-623e1a7b9e9f)

Yeah, that wasn't the greatest first impression... 😅

It also created random "updates" that were completely unclear - what had actually happened was that the _description_ had updated, but it showed a price update.

![image](https://github.com/user-attachments/assets/0c0633a7-ed35-4894-bf66-39cd3f9ef76f)

What was worse was that Hack Club had created a new channel called [#arcade-bulletin](https://google.com) that seemingly made the bot obsolete.

![image](https://github.com/user-attachments/assets/b09b07bf-a824-4a51-a739-16931a88eea1)

I did quickly realise, however, that the bot could track things like stock updates that were infeasible for Hack Club staff to do every time someone bought a YubiKey.

> (Also, fun fact: the YubiKeys are technically limited, but there's over 1,000 of them. The more you know 🤷‍♂️)

In the future, I would definitely use libraries such as Zod to validate my scraper - it would have saved _so_ much time.

### How the scraper works

It's actually surprisingly simple! Under the hood, Hack Club's website uses Next.js, and Next.js includes a `script` tag with the ID of `__NEXT_DATA__` with a bunch of useful information. That useful information just so happens to contain the Arcade Shop's items in a nice and easy-to-use format.

<details>
  <summary>View the scraper's code (~50 lines of Rust)</summary>

  ```rust
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
        .await
        .unwrap();
    let doc_html = response.text().await.unwrap();

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
  ```
</details>

<details>
    <summary>View the original scraper's code (30 lines of TypeScript)</summary>
  <p>Note that the type definitions are incorrect.</p>
  
  ```typescript
  import { parse as parseHtml } from "node-html-parser";

  export type ShopItem = {
	  name: string;
	  description: string;
	  fulfillmentDescription: string;
	  id: string;
	  imageUrl: string;
	  maxOrderQuantity: number;
	  price: number;
  };

  export default function parseArcadeShopHtml(html: string): ShopItem[] {
	  const document = parseHtml(html);
	  const inner = document.getElementById("\_\_NEXT_DATA\_\_")?.text!;

  const items = JSON.parse(inner).props.pageProps.availableItems.map((item: any) => {
	  return {
			name: item\["Name"],
			description: item\["Description"],
			fulfillmentDescription: item\["Fulfillment Description"],
			id: item.id,
			imageUrl: item\["Image URL"],
			maxOrderQuantity: item\["Max Order Quantity"],
			price: item\["Cost Hours"],
		};
	});

  return items;
```
</details>

## v2: the Rust version

Since my code was borked anyway, I decided that I might as well just rewrite the whole thing. I decided to rewrite it in Rust! Technically, I could've made the rewrite in TypeScript and it would've been fine, but I decided to rewrite it in Rust instead for a couple reasons:

- Serde is great and reduces boilerplate
- Rust's error handling is _superb._ I would absolutely love `Result<T, E>` to come to Go or TypeScript (and actually be used by the wider community)
- I just like Rust :D

The new version took a _lot_ less time to implement (around 3 hours) - in fact, it took less time to write than the initial version written in TypeScript! Most of my code after that was for things like enhancements and new features.

Three hours later... it worked!

![image](https://github.com/user-attachments/assets/302c1ed2-5c4e-44e4-9659-842dfdad7324)

Since then, most of the changes have been things like the ticket/price ratio or prettier Slack messages that use Block Kit. 

Overall, I'd call the Rust rewrite a huge success!

</details>

---

_© 2024 Mahad Kalam_

_Licensed under the [MIT License](https://github.com/SkyfallWasTaken/arcade-monitor/blob/main/LICENSE.md)_
