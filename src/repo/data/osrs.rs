use crate::Database;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use reqwest::header::USER_AGENT;
use serde::{Deserialize, Serialize};
use serde_json;
use tokio;

#[derive(Clone)]
pub struct Osrs {
    maps: Arc<Mutex<HashMap<i64, OsrsMap>>>,
    ge: Arc<Mutex<HashMap<i64, GePrice>>>,
    high_alch_profit: Arc<Mutex<Vec<HighAlchProfit>>>,
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
            let temp: i64 = k.clone().parse().unwrap();

            let mut temp_data = d.clone();

            match temp_data.low {
                Some(e) => match temp_data.high {
                    Some(g) => {
                        if e > g {
                            temp_data.high = Some(e.clone());
                        }
                    }
                    None => (),
                },
                None => (),
            };

            temp_map.insert(temp, temp_data);
        }

        //match database.insert_ge_price_bulk(&temp_map).await {
        //Ok(_) => (),
        //Err(e) => return Err("Cannot insert_ge_price_bulk".to_string()),
        //};

        let hap = Osrs::gen_high_alch_profit(&temp_map, &temp_ge_map);

        let maps = Arc::new(Mutex::new(temp_ge_map));
        let high_alch_profit = Arc::new(Mutex::new(hap));
        let ge = Arc::new(Mutex::new(temp_map));

        let maps_copy = maps.clone();
        let high_alch_profit_copy = high_alch_profit.clone();
        let ge_copy = ge.clone();
        let database_copy = database.clone();

        tokio::spawn(async move {
            Osrs::update_schedule(maps_copy, ge_copy, high_alch_profit_copy, database_copy).await;
        });

        return Ok(Osrs {
            maps,
            maps_age: chrono::Utc::now(),
            high_alch_profit,
            ge,
            ge_age: chrono::Utc::now(),
            database: database,
        });
    }

    fn gen_high_alch_profit(
        ge: &HashMap<i64, GePrice>,
        map: &HashMap<i64, OsrsMap>,
    ) -> Vec<HighAlchProfit> {
        let mut temp_vec: Vec<HighAlchProfit> = Vec::new();
        let nr_price = match ge.get(&561_i64) {
            Some(e) => match e.high {
                Some(e) => e,
                None => panic!("no nature ruin price"),
            },

            None => panic!("no nature ruin price"),
        };

        for (ge_k, ge_d) in ge.iter() {
            let price = match ge_d.high {
                Some(e) => e.clone(),
                None => match ge_d.low {
                    Some(e) => e.clone(),
                    None => continue,
                },
            };

            let map_d = match map.get(ge_k) {
                Some(e) => e,
                None => continue,
            };

            let high_alch = match map_d.highalch {
                Some(e) => e.clone(),
                None => continue,
            };

            if high_alch < (price + nr_price) {
                continue;
            }

            let profit: i64 = (((high_alch - (price + nr_price)) as f64 / high_alch as f64)
                * 100_f64)
                .round() as i64;

            temp_vec.push(HighAlchProfit {
                profit_percent: profit,
                profit_per_use: (high_alch as i128 - (price as i128 + nr_price as i128)),
                ge_val: price,
                highalch: high_alch,
                name: map_d.name.clone(),
                id: map_d.id.clone(),
                members: map_d.members.clone(),
                icon: map_d.icon.clone(),
            })
        }

        temp_vec.sort_by_key(|d| d.profit_per_use);
        temp_vec.reverse();

        return temp_vec;
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

    async fn update_schedule(
        maps: Arc<Mutex<HashMap<i64, OsrsMap>>>,
        ge: Arc<Mutex<HashMap<i64, GePrice>>>,
        high_alch_profit: Arc<Mutex<Vec<HighAlchProfit>>>,
        database: Database,
    ) {
        println!("starting thread");
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
            println!("updating Cache");

            let data = match Osrs::fetch_maps().await {
                Ok(e) => e,
                Err(e) => {
                    println!("cannot fetch maps");
                    continue;
                }
            };

            let mut temp_ge_map: HashMap<i64, OsrsMap> = HashMap::new();

            for thing in data {
                let temp = thing.id.clone();

                temp_ge_map.insert(temp, thing.clone());
            }

            let data = match Osrs::fetch_ge().await {
                Ok(e) => e,
                Err(e) => {
                    println!("cannot fetch maps");
                    continue;
                }
            };

            let mut temp_map: HashMap<i64, GePrice> = HashMap::new();

            for (k, d) in data.iter() {
                let temp: i64 = k.clone().parse().unwrap();

                let mut temp_data = d.clone();

                match temp_data.low {
                    Some(e) => match temp_data.high {
                        Some(g) => {
                            if e > g {
                                temp_data.high = Some(e.clone());
                            }
                        }
                        None => (),
                    },
                    None => (),
                };

                temp_map.insert(temp, temp_data);
            }

            match database.insert_ge_price_bulk(&temp_map).await {
                Ok(_) => (),
                Err(_) => print!("Cannot insert_ge_price_bulk"),
            };

            let hap = Osrs::gen_high_alch_profit(&temp_map, &temp_ge_map);

            let mut maps_mut = maps.lock().unwrap();
            *maps_mut = temp_ge_map.clone();
            drop(maps_mut);

            let mut ge_mut = ge.lock().unwrap();
            *ge_mut = temp_map;
            drop(ge_mut);

            let mut hap_mut = high_alch_profit.lock().unwrap();
            *hap_mut = hap;
            drop(hap_mut);

            println!("cache updated");
        }
    }

    pub async fn get_maps_all(&self) -> HashMap<i64, OsrsMap> {
        let stuff = self.maps.lock().unwrap();

        return stuff.clone();
    }

    pub async fn get_ge_all(&self) -> HashMap<i64, GePrice> {
        let stuff = self.ge.lock().unwrap();

        return stuff.clone();
    }

    pub async fn get_high_alch_profit(&self) -> Vec<HighAlchProfit> {
        println!("waiting for lock");
        let stuff = self.high_alch_profit.lock().unwrap();
        println!("locked");
        return stuff.clone();
    }

    pub async fn get_ge_one(&self, id: &i64) -> Option<GePrice> {
        let stuff = self.ge.lock().unwrap();

        let res = stuff.get(id);

        return match res {
            Some(e) => Some(e.clone()),
            None => None,
        };
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OsrsMap {
    pub examine: String,
    pub id: i64,
    pub members: bool,
    pub lowalch: Option<u128>,
    pub limit: Option<u128>,
    pub value: u128,
    pub highalch: Option<u128>,
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
    pub high: Option<u128>,
    pub high_time: Option<i64>,
    pub low: Option<u128>,
    pub low_time: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HighAlchProfit {
    pub name: String,
    pub id: i64,
    pub members: bool,
    pub highalch: u128,
    pub icon: String,
    pub ge_val: u128,
    pub profit_percent: i64,
    pub profit_per_use: i128,
}
