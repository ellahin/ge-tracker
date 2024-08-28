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

    println!("{:?}", data.get_ge_all().await);
}
