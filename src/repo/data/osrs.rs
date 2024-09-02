use crate::Database;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use reqwest::header::USER_AGENT;
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::Value;
use tokio;

#[derive(Clone)]
pub struct Osrs {
    maps: Arc<Mutex<HashMap<i64, OsrsMap>>>,
    ge: Arc<Mutex<HashMap<i64, GePrice>>>,
    high_alch_profit: Arc<Mutex<Vec<HighAlchProfit>>>,
    low_alch_profit: Arc<Mutex<Vec<LowAlchProfit>>>,
    crafting: Arc<Vec<CraftingItem>>,
    crafting_profit: Arc<Mutex<Vec<CraftingItemProfit>>>,
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

        match database.insert_ge_price_bulk(&temp_map).await {
            Ok(_) => (),
            Err(e) => return Err("Cannot insert_ge_price_bulk".to_string()),
        };

        let hap = Osrs::gen_high_alch_profit(&temp_map, &temp_ge_map);
        let lap = Osrs::gen_low_alch_profit(&temp_map, &temp_ge_map);
        let ci_temp = Osrs::fetch_crafting().await;
        let ci = Osrs::convert_crafting(ci_temp, temp_ge_map.clone());
        let ci_ge = Osrs::convert_crafting_profit(&ci, temp_map.clone());

        let crafting_clone = ci.clone();

        let maps = Arc::new(Mutex::new(temp_ge_map));
        let high_alch_profit = Arc::new(Mutex::new(hap));
        let low_alch_profit = Arc::new(Mutex::new(lap));
        let ge = Arc::new(Mutex::new(temp_map));
        let crafting = Arc::new(ci);
        let crafting_profit = Arc::new(Mutex::new(ci_ge));

        let maps_copy = maps.clone();
        let high_alch_profit_copy = high_alch_profit.clone();
        let low_alch_profit_copy = low_alch_profit.clone();
        let ge_copy = ge.clone();
        let database_copy = database.clone();
        let crafting_profit_copy = crafting_profit.clone();

        tokio::spawn(async move {
            Osrs::update_schedule(
                maps_copy,
                ge_copy,
                high_alch_profit_copy,
                low_alch_profit_copy,
                crafting_clone,
                crafting_profit_copy,
                database_copy,
            )
            .await;
        });

        return Ok(Osrs {
            maps,
            high_alch_profit,
            low_alch_profit,
            ge,
            database,
            crafting_profit,
            crafting,
        });
    }
    fn gen_low_alch_profit(
        ge: &HashMap<i64, GePrice>,
        map: &HashMap<i64, OsrsMap>,
    ) -> Vec<LowAlchProfit> {
        let mut temp_vec: Vec<LowAlchProfit> = Vec::new();
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

            let low_alch = match map_d.lowalch {
                Some(e) => e.clone(),
                None => continue,
            };

            if low_alch < (price + nr_price) {
                continue;
            }

            let profit: i64 = (((low_alch - (price + nr_price)) as f64 / low_alch as f64) * 100_f64)
                .round() as i64;

            temp_vec.push(LowAlchProfit {
                profit_percent: profit,
                profit_per_use: (low_alch as i128 - (price as i128 + nr_price as i128)),
                ge_val: price,
                lowalch: low_alch,
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
        low_alch_profit: Arc<Mutex<Vec<LowAlchProfit>>>,
        crafting: Vec<CraftingItem>,
        craftting_profit: Arc<Mutex<Vec<CraftingItemProfit>>>,
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
            let lap = Osrs::gen_low_alch_profit(&temp_map, &temp_ge_map);
            let ci_ge = Osrs::convert_crafting_profit(&crafting.clone(), temp_map.clone());

            let mut maps_mut = maps.lock().unwrap();
            *maps_mut = temp_ge_map.clone();
            drop(maps_mut);

            let mut ge_mut = ge.lock().unwrap();
            *ge_mut = temp_map;
            drop(ge_mut);

            let mut hap_mut = high_alch_profit.lock().unwrap();
            *hap_mut = hap;
            drop(hap_mut);

            let mut lap_mut = low_alch_profit.lock().unwrap();
            *lap_mut = lap;
            drop(lap_mut);

            let mut ci_ge_mut = craftting_profit.lock().unwrap();
            *ci_ge_mut = ci_ge;
            drop(ci_ge_mut);

            println!("cache updated");
        }
    }

    pub fn get_maps_all(&self) -> HashMap<i64, OsrsMap> {
        let stuff = self.maps.lock().unwrap();

        return stuff.clone();
    }

    pub fn get_ge_all(&self) -> HashMap<i64, GePrice> {
        let stuff = self.ge.lock().unwrap();

        return stuff.clone();
    }

    pub fn get_high_alch_profit(&self) -> Vec<HighAlchProfit> {
        let stuff = self.high_alch_profit.lock().unwrap();
        return stuff.clone();
    }

    pub fn get_low_alch_profit(&self) -> Vec<LowAlchProfit> {
        let stuff = self.low_alch_profit.lock().unwrap();
        return stuff.clone();
    }

    pub fn get_crafting_profit(&self) -> Vec<CraftingItemProfit> {
        let stuff = self.crafting_profit.lock().unwrap();
        return stuff.clone();
    }

    pub fn get_ge_one(&self, id: &i64) -> Option<GePrice> {
        let stuff = self.ge.lock().unwrap();

        let res = stuff.get(id);

        return match res {
            Some(e) => Some(e.clone()),
            None => None,
        };
    }

    async fn fetch_crafting() -> HashMap<String, CraftingRequestItem> {
        let client = reqwest::Client::new();

        let mut offset: usize = 0;

        let mut items: HashMap<String, CraftingRequestItem> = HashMap::new();

        loop {
            let url = format!("https://oldschool.runescape.wiki/w/Special:Ask/class%3Dsortable-20wikitable-20smwtable/format%3Djson/headers%3Dshow/link%3Dall/mainlabel%3D/searchlabel%3DJSON/sort%3D/order%3Dasc/offset%3D{}/limit%3D500/-5B-5BProduction-20JSON::%2B-5D-5D/-3FProduction-20JSON/prettyprint%3Dtrue/unescape%3Dtrue", offset);
            let temp_items: CraftingRequest = match client
                .get(url)
                .header(USER_AGENT, "gecalculator - ellabella on discord")
                .send()
                .await
            {
                Ok(e) => {
                    let text = e.text().await.unwrap();
                    if text.len() <= 5 {
                        break;
                    }
                    match serde_json::from_str(&text) {
                        Ok(i) => i,

                        Err(i) => panic!("Can't deserialize json {:?}", i),
                    }
                }
                Err(e) => {
                    println!("Error fetching maps: {:?}", e);
                    break;
                }
            };

            offset = offset + temp_items.rows as usize;
            for (k, i) in temp_items.results {
                items.insert(k, i);
            }
        }

        return items;
    }

    fn convert_crafting(
        request_items: HashMap<String, CraftingRequestItem>,
        map: HashMap<i64, OsrsMap>,
    ) -> Vec<CraftingItem> {
        let mut cleaned_map: HashMap<String, OsrsMap> = HashMap::new();

        for (k, d) in map {
            cleaned_map.insert(d.name.clone(), d.clone());
        }

        let mut crafting_items: Vec<CraftingItem> = Vec::new();

        for (k, d) in request_items {
            let item_map = match cleaned_map.get(&k) {
                Some(e) => e.clone(),
                None => {
                    println!("Cannot find item \"{}\" in map", k);
                    continue;
                }
            };

            let production_raw = d.printouts.production_json.clone();

            let mut production: Vec<CraftingRequestPoduction> = Vec::new();

            for (i, p) in production_raw.iter().enumerate() {
                let some: CraftingRequestPoduction = match serde_json::from_str(&p) {
                    Ok(e) => e,
                    Err(_) => {
                        println!(
                            "Cannot pass json on production {} on item {}",
                            i, item_map.name
                        );
                        continue;
                    }
                };

                production.push(some);
            }

            for (i, p) in production.iter().enumerate() {
                let mut materials: Vec<CraftingMaterial> = Vec::new();

                for m in p.materials.clone() {
                    let mat_map = match cleaned_map.get(&m.name) {
                        Some(e) => e.clone(),
                        None => {
                            println!(
                                "Cannot find item \"{}\" in map for production {} in item {}",
                                m.name, i, item_map.name
                            );
                            continue;
                        }
                    };

                    materials.push(CraftingMaterial {
                        name: mat_map.name.clone(),
                        id: mat_map.id.clone(),
                        icon: mat_map.icon.clone(),
                        count: match m.quantity.clone().parse() {
                            Ok(e) => e,
                            Err(_) => {
                                println!(
                                "Cannot pass quantity on item \"{}\" for production {} in item {}",
                                m.name, i, item_map.name
                            );
                                continue;
                            }
                        },
                    })
                }

                if materials.len() != p.materials.len() {
                    println!(
                        "Missing materials for production {} in item {}",
                        i, item_map.name
                    );
                    continue;
                }

                crafting_items.push(CraftingItem {
                    name: item_map.name.clone(),
                    icon: item_map.icon.clone(),
                    id: item_map.id,
                    materials,
                    skills: p.skills.clone(),
                    facilities: p.facilities.clone(),
                    ticks: p.ticks.clone(),
                    members: p.members.clone(),
                    output: match p.output.quantity.parse() {
                        Ok(e) => e,
                        Err(_) => {
                            println!(
                                "Cannot pass quantity on output for production {} in item {}",
                                i, item_map.name
                            );
                            continue;
                        }
                    },
                })
            }
        }
        return crafting_items;
    }

    fn convert_crafting_profit(
        crafting_items: &Vec<CraftingItem>,
        ge: HashMap<i64, GePrice>,
    ) -> Vec<CraftingItemProfit> {
        let mut res: Vec<CraftingItemProfit> = Vec::new();

        for c in crafting_items {
            let gedata = match ge.get(&c.id) {
                Some(e) => e,
                None => continue,
            };

            let price = match gedata.high {
                Some(e) => e,
                None => continue,
            };

            let mut material_data: Vec<CraftingMaterialCost> = Vec::new();

            for m in c.materials.clone() {
                let matgedata = match ge.get(&m.id) {
                    Some(e) => e,
                    None => continue,
                };

                material_data.push(CraftingMaterialCost {
                    name: m.name.clone(),
                    icon: m.icon.clone(),
                    id: m.id.clone(),
                    count: m.count.clone(),
                    cost: match matgedata.high {
                        Some(e) => e.clone(),
                        None => continue,
                    },
                });
            }

            if material_data.len() != c.materials.len() {
                continue;
            }

            let mut total_cost: i64 = 0;

            for m in &material_data {
                total_cost += m.cost;
            }

            let profit = price - total_cost;

            let profit_margin =
                (((price - total_cost) as f64 / price as f64) * 100_f64).round() as f32;

            res.push(CraftingItemProfit {
                name: c.name.clone(),
                icon: c.icon.clone(),
                id: c.id,
                materials: material_data,
                skills: c.skills.clone(),
                facilities: c.facilities.clone(),
                ticks: c.ticks.clone(),
                members: c.members.clone(),
                output: c.output.clone(),
                total_cost,
                price,
                profit_margin,
                profit,
            })
        }

        res.sort_by_key(|d| d.profit);
        res.reverse();

        return res;
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CraftingRequest {
    pub printrequests: Vec<CraftingPrintrequest>,
    pub results: HashMap<String, CraftingRequestItem>,
    pub serializer: String,
    pub version: i64,
    pub rows: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CraftingPrintrequest {
    pub label: String,
    pub key: String,
    pub redi: String,
    pub typeid: String,
    pub mode: i64,
    pub format: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CraftingRequestItem {
    pub printouts: CraftingPrintouts,
    pub fulltext: String,
    pub fullurl: String,
    pub namespace: i64,
    pub exists: String,
    pub displaytitle: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CraftingPrintouts {
    #[serde(rename = "Production JSON")]
    pub production_json: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CraftingRequestPoduction {
    pub ticks: String,
    pub materials: Vec<CraftingRequestPoductionMaterial>,
    pub facilities: Option<String>,
    pub skills: Vec<CraftingSkill>,
    pub members: String,
    pub output: CraftingRequestPoductionOutput,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CraftingRequestPoductionMaterial {
    pub name: String,
    pub quantity: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CraftingRequestPoductionOutput {
    pub cost: i64,
    pub quantity: String,
    pub name: String,
    pub subtxt: String,
    pub image: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CraftingSkill {
    pub experience: String,
    pub level: String,
    pub name: String,
    pub boostable: String,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CraftingItem {
    pub name: String,
    pub icon: String,
    pub id: i64,
    pub ticks: String,
    pub materials: Vec<CraftingMaterial>,
    pub facilities: Option<String>,
    pub skills: Vec<CraftingSkill>,
    pub members: String,
    pub output: u16,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CraftingMaterial {
    pub name: String,
    pub icon: String,
    pub id: i64,
    pub count: u8,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CraftingItemProfit {
    pub name: String,
    pub icon: String,
    pub id: i64,
    pub ticks: String,
    pub materials: Vec<CraftingMaterialCost>,
    pub facilities: Option<String>,
    pub skills: Vec<CraftingSkill>,
    pub members: String,
    pub output: u16,
    pub total_cost: i64,
    pub price: i64,
    pub profit: i64,
    pub profit_margin: f32,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CraftingMaterialCost {
    pub name: String,
    pub icon: String,
    pub id: i64,
    pub count: u8,
    pub cost: i64,
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HighAlchProfit {
    pub name: String,
    pub id: i64,
    pub members: bool,
    pub highalch: i64,
    pub icon: String,
    pub ge_val: i64,
    pub profit_percent: i64,
    pub profit_per_use: i128,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LowAlchProfit {
    pub name: String,
    pub id: i64,
    pub members: bool,
    pub lowalch: i64,
    pub icon: String,
    pub ge_val: i64,
    pub profit_percent: i64,
    pub profit_per_use: i128,
}
