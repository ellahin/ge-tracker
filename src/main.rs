mod repo;

use crate::repo::data::osrs::Osrs;
use crate::repo::sql::Database;

use std::env;

use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    if env::var("DATABASE_URL").is_err() {
        panic!("DATABASE_URL not in environment vars");
    }
    let database = Database::new(env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let data = Osrs::new(database.clone()).await.unwrap();

    for stuff in data.get_high_alch_profit().await {
        println!(
            "Name: {}, id: {}, ge: {}, HA: {}, Profit: {}",
            stuff.name, stuff.id, stuff.ge_val, stuff.highalch, stuff.profit
        );
    }
}
