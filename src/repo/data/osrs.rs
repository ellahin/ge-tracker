use crate::Database;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use reqwest::header::USER_AGENT;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Clone)]
pub struct Osrs {
    maps: Arc<Mutex<HashMap<i64, OsrsMap>>>,
    ge: Arc<Mutex<HashMap<i64, GePrice>>>,
    maps_age: DateTime<Utc>,
    ge_age: DateTime<Utc>,
    database: Database,
}

impl Osrs {
    pub async fn new(database: Database) -> Result<Self, String> {
        let data = match Osrs::fetch_maps().await {
            Ok(e) => e,
            Err(e) => return Err(e),
        };

        let mut temp_ge_map: HashMap<i64, OsrsMap> = HashMap::new();

        for thing in data {
            let temp = thing.id.clone();

            temp_ge_map.insert(temp, thing.clone());
        }

        let data = match Osrs::fetch_ge().await {
            Ok(e) => e,
            Err(e) => return Err(e),
        };

        let mut temp_map: HashMap<i64, GePrice> = HashMap::new();

        for (k, d) in data.iter() {
            println!("{}", k);
            let temp: i64 = k.clone().parse().unwrap();

            temp_map.insert(temp, d.clone());
        }

        match database.insert_ge_price_bulk(&temp_map).await {
            Ok(_) => (),
            Err(e) => return Err("Cannot insert_ge_price_bulk".to_string()),
        };

        return Ok(Osrs {
            maps: Arc::new(Mutex::new(temp_ge_map)),
            maps_age: chrono::Utc::now(),
            ge: Arc::new(Mutex::new(temp_map)),
            ge_age: chrono::Utc::now(),
            database: database,
        });
    }

    async fn fetch_maps() -> Result<OsrsMapsRaw, String> {
        let client = reqwest::Client::new();

        let res = match client
            .get("https://prices.runescape.wiki/api/v1/osrs/mapping")
            .header(USER_AGENT, "gecalculator - ellabella on discord")
            .send()
            .await
        {
            Ok(e) => e,
            Err(e) => {
                println!("Error fetching maps: {:?}", e);
                return Err("Couldn't fetch mappings".to_string());
            }
        };

        let raw = res.text().await.unwrap();

        let obj: OsrsMapsRaw = serde_json::from_str(&raw).unwrap();

        return Ok(obj);
    }

    async fn fetch_ge() -> Result<HashMap<String, GePrice>, String> {
        let client = reqwest::Client::new();

        let res = match client
            .get("https://prices.runescape.wiki/api/v1/osrs/latest")
            .header(USER_AGENT, "gecalculator - ellabella on discord")
            .send()
            .await
        {
            Ok(e) => e,
            Err(e) => {
                println!("Error fetching maps: {:?}", e);
                return Err("Couldn't fetch mappings".to_string());
            }
        };

        let raw = res.text().await.unwrap();

        let obj: OsrsGeData = serde_json::from_str(&raw).unwrap();

        return Ok(obj.data);
    }

    pub async fn get_maps_all(&self) -> HashMap<i64, OsrsMap> {
        let stuff = self.maps.lock().unwrap();

        return stuff.clone();
    }

    pub async fn get_ge_all(&self) -> HashMap<i64, GePrice> {
        let stuff = self.ge.lock().unwrap();

        return stuff.clone();
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OsrsMap {
    pub examine: String,
    pub id: i64,
    pub members: bool,
    pub lowalch: Option<i64>,
    pub limit: Option<i64>,
    pub value: i64,
    pub highalch: Option<i64>,
    pub icon: String,
    pub name: String,
}

pub type OsrsMapsRaw = Vec<OsrsMap>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OsrsGeData {
    data: HashMap<String, GePrice>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GePrice {
    pub high: Option<i64>,
    pub high_time: Option<i64>,
    pub low: Option<i64>,
    pub low_time: Option<i64>,
}
