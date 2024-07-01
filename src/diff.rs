use crate::items::ShopItem;

pub fn diff_items(old: ShopItem, new: ShopItem) -> Option<String> {
    if old == new {
        // The items are the exact same
        return None;
    }

    let mut result = Vec::new();
    result.push("*Item updated*".into());

    if old.full_name != new.full_name {
        result.push(format!("*Name:* {} → {}", old.full_name, new.full_name));
    } else {
        result.push(format!("*Name:* {}", new.full_name));
    }

    if old.description != new.description {
        result.push(format!(
            "*Description:* {} → {}",
            old.description.unwrap_or("_not set_".into()),
            new.description.unwrap_or("_not set_".into())
        ));
    }

    if old.price != new.price {
        result.push(format!("*Price:* {} → {}", old.price, new.price));
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

#[cfg(test)]
mod test {
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
            diff_items(old, new),
            Some(
                indoc! {"
                *Item updated*
                *Name:* Test
                *Price:* 1 → 2"}
                .into()
            )
        );
    }
}
