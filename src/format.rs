use indoc::formatdoc;

use crate::items::ShopItem;

pub fn format_item_diff(old: &ShopItem, new: &ShopItem) -> Option<String> {
    if old == new {
        // The items are the exact same
        return None;
    }

    let mut result = Vec::new();

    if old.full_name != new.full_name {
        result.push(format!("*Name:* {} → {}", old.full_name, new.full_name));
    } else {
        result.push(format!("*Name:* {}", new.full_name));
    }

    if old.price != new.price {
        result.push(format!("*Price:* {} → {}", old.price, new.price));
    }

    if old.description != new.description {
        result.push(format!(
            "*Description:* {} → {}",
            old.description.as_ref().unwrap_or(&"_not set_".into()),
            new.description.as_ref().unwrap_or(&"_not set_".into())
        ));
    }

    if old.fulfillment_description != new.fulfillment_description {
        result.push(format!(
            "*Fulfillment info:* {} → {}",
            old.fulfillment_description
                .as_ref()
                .unwrap_or(&"_not set_".into()),
            new.fulfillment_description
                .as_ref()
                .unwrap_or(&"_not set_".into())
        ));
    }

    if old.stock != new.stock {
        result.push(format!(
            "*Stock:* {} → {}",
            old.stock
                .map(|stock| stock.to_string())
                .unwrap_or("Unlimited".into()),
            new.stock
                .map(|stock| stock.to_string())
                .unwrap_or("Unlimited".into())
        ));
    }

    Some(result.join("\n"))
}

pub fn format_new_item(item: &ShopItem) -> String {
    formatdoc! {
        "*New item added:* {full_name}
        *Description:* {description}
        *Fulfillment info:* {fullfilment_info}
        *Price:* {price}
        *Stock:* {stock}",
        full_name = item.full_name,
        description = item.description.as_ref().unwrap_or(&"_not set_".into()),
        fullfilment_info = item.fulfillment_description.as_ref().unwrap_or(&"_not set_".into()),
        price = item.price,
        stock = item.stock
            .map(|stock| stock.to_string())
            .unwrap_or("Unlimited".into()),
    }
}

pub fn format_deleted_item(item: &ShopItem) -> String {
    formatdoc! {
        "*Item DELETED:* {full_name}
        *Description:* {description}
        *Price:* {price}",
        full_name = item.full_name,
        description = item.description.as_ref().unwrap_or(&"_not set_".into()),
        price = item.price,
    }
}

#[cfg(test)]
mod format_new_tests {
    use super::*;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn item_formatted() {
        let item = ShopItem {
            full_name: "Test".into(),
            description: Some("Lorem ipsum".into()),
            fulfillment_description: Some("Dolor sit amet".into()),
            price: 1,
            stock: Some(10),
            ..Default::default()
        };

        assert_eq!(
            format_new_item(&item),
            indoc! {"
            *New item added:* Test
            *Description:* Lorem ipsum
            *Fulfillment info:* Dolor sit amet
            *Price:* 1
            *Stock:* 10"}
        );
    }
}

#[cfg(test)]
mod diff_tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn price_diff() {
        let old = ShopItem {
            full_name: "Test".into(),
            price: 1,
            ..Default::default()
        };

        let new = ShopItem {
            full_name: "Test".into(),
            price: 2,
            ..Default::default()
        };

        assert_eq!(
            format_item_diff(&old, &new),
            Some(
                indoc! {"
                *Name:* Test
                *Price:* 1 → 2"}
                .into()
            )
        );
    }

    #[test]
    fn description_diff() {
        let old = ShopItem {
            full_name: "Test".into(),
            description: Some("Lorem ipsum".into()),
            ..Default::default()
        };

        let new = ShopItem {
            full_name: "Test".into(),
            description: Some("Dolor sit amet".into()),
            ..Default::default()
        };

        assert_eq!(
            format_item_diff(&old, &new),
            Some(
                indoc! {"
                *Name:* Test
                *Description:* Lorem ipsum → Dolor sit amet"}
                .into()
            )
        );
    }

    #[test]
    fn stock_diff_limited_update() {
        let old = ShopItem {
            full_name: "Test".into(),
            stock: Some(10),
            ..Default::default()
        };

        let new = ShopItem {
            full_name: "Test".into(),
            stock: Some(9),
            ..Default::default()
        };

        assert_eq!(
            format_item_diff(&old, &new),
            Some(
                indoc! {"
                *Name:* Test
                *Stock:* 10 → 9"}
                .into()
            )
        );
    }

    #[test]
    fn stock_diff_limited_to_unlimited() {
        let old = ShopItem {
            full_name: "Test".into(),
            stock: Some(10),
            ..Default::default()
        };

        let new = ShopItem {
            full_name: "Test".into(),
            stock: None,
            ..Default::default()
        };

        assert_eq!(
            format_item_diff(&old, &new),
            Some(
                indoc! {"
                *Name:* Test
                *Stock:* 10 → Unlimited"}
                .into()
            )
        );
    }

    #[test]
    fn stock_diff_unlimited_to_limited() {
        let old = ShopItem {
            full_name: "Test".into(),
            stock: None,
            ..Default::default()
        };

        let new = ShopItem {
            full_name: "Test".into(),
            stock: Some(10),
            ..Default::default()
        };

        assert_eq!(
            format_item_diff(&old, &new),
            Some(
                indoc! {"
                *Name:* Test
                *Stock:* Unlimited → 10"}
                .into()
            )
        );
    }

    #[test]
    fn test_fulfillment_info() {
        let old = ShopItem {
            full_name: "Test".into(),
            fulfillment_description: Some("Lorem ipsum".into()),
            ..Default::default()
        };

        let new = ShopItem {
            full_name: "Test".into(),
            fulfillment_description: Some("Dolor sit amet".into()),
            ..Default::default()
        };

        assert_eq!(
            format_item_diff(&old, &new),
            Some(
                indoc! {"
                *Name:* Test
                *Fulfillment info:* Lorem ipsum → Dolor sit amet"}
                .into()
            )
        );
    }

    #[test]
    fn equal_items_no_diff() {
        let item = ShopItem {
            full_name: "Test".into(),
            price: 1,
            ..Default::default()
        };

        assert_eq!(format_item_diff(&item, &item), None);
    }
}
