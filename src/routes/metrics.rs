use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

use crate::repo::data::osrs::GePrice;
use crate::AppState;

pub async fn get(State(state): State<AppState>) -> String {
    let mut res = "".to_string();

    let ge_price = state.osrs.get_ge_all();

    res = format!("{}# Ge high price\n", res);

    let map = state.osrs.get_maps_all();

    for (k, d) in ge_price.iter() {
        match d.high {
            Some(e) => {
                let name = match map.get(k) {
                    Some(e) => e.name.clone(),
                    None => continue,
                };
                res = format!(
                    "{}ge_item_high_price{{item_id=\"{}\",item_name=\"{}\"}} {}\n",
                    res, k, name, e
                );
            }
            None => (),
        };
    }

    res = format!("{}# Ge low price\n", res);

    for (k, d) in ge_price.iter() {
        match d.low {
            Some(e) => {
                let name = match map.get(k) {
                    Some(e) => e.name.clone(),
                    None => continue,
                };

                res = format!(
                    "{}ge_item_low_price{{item_id=\"{}\",item_name=\"{}\"}} {}\n",
                    res, k, name, e
                );
            }
            None => (),
        };
    }

    drop(ge_price);

    res = format!("{}# High Alch profit percent\n", res);

    let high_alch = state.osrs.get_high_alch_profit();

    for d in high_alch.clone() {
        let name = match map.get(&d.id) {
            Some(e) => e.name.clone(),
            None => continue,
        };

        res = format!(
            "{}ge_high_alch_profit_percent{{item_id=\"{}\",item_name=\"{}\",members=\"{}\"}} {}\n",
            res, d.id, name, d.members, d.profit_percent
        );
    }

    res = format!("{}# High Alch profit\n", res);

    for d in high_alch {
        let name = match map.get(&d.id) {
            Some(e) => e.name.clone(),
            None => continue,
        };

        res = format!(
            "{}ge_high_alch_profit{{item_id=\"{}\",item_name=\"{}\",members=\"{}\"}} {}\n",
            res, d.id, name, d.members, d.profit_per_use
        );
    }

    res = format!("{}# Low Alch profit percent\n", res);

    let low_alch = state.osrs.get_low_alch_profit();

    for d in &low_alch {
        let name = match map.get(&d.id) {
            Some(e) => e.name.clone(),
            None => continue,
        };

        res = format!(
            "{}ge_low_alch_profit_percent{{item_id=\"{}\",item_name=\"{}\",members=\"{}\"}} {}\n",
            res, d.id, name, d.members, d.profit_percent
        );
    }

    res = format!("{}# Low Alch profit\n", res);

    for d in &low_alch {
        let name = match map.get(&d.id) {
            Some(e) => e.name.clone(),
            None => continue,
        };

        res = format!(
            "{}ge_low_alch_profit{{item_id=\"{}\",item_name=\"{}\",members=\"{}\"}} {}\n",
            res, d.id, name, d.members, d.profit_per_use
        );
    }

    drop(low_alch);

    let crafting = state.osrs.get_crafting_profit();

    res = format!("{}# Crafting profit\n", res);

    for d in &crafting {
        let name = match map.get(&d.id) {
            Some(e) => e.name.clone(),
            None => continue,
        };

        res = format!(
            "{}ge_crafting_profit{{item_id=\"{}\",item_name=\"{}\",members=\"{}\"}} {}\n",
            res, d.id, name, d.members, d.profit
        );
    }

    res = format!("{}# Crafting profit margin\n", res);

    for d in &crafting {
        let name = match map.get(&d.id) {
            Some(e) => e.name.clone(),
            None => continue,
        };

        res = format!(
            "{}ge_crafting_profit_margin{{item_id=\"{}\",item_name=\"{}\",members=\"{}\"}} {}\n",
            res, d.id, name, d.members, d.profit_margin
        );
    }

    res = format!("{}# Crafting price\n", res);

    for d in &crafting {
        let name = match map.get(&d.id) {
            Some(e) => e.name.clone(),
            None => continue,
        };

        res = format!(
            "{}ge_crafting_price{{item_id=\"{}\",item_name=\"{}\",members=\"{}\"}} {}\n",
            res, d.id, name, d.members, d.price
        );
    }

    return res;
}
