use crate::repo::data::osrs::GePrice;

use std::collections::HashMap;
use std::path::Path;

use sqlx::migrate::Migrator;
use sqlx::types::chrono::{NaiveDateTime, Utc};
use sqlx::Postgres;
use sqlx::{PgPool, Pool};

#[derive(Clone)]
pub struct Database {
    database: Pool<Postgres>,
}

impl Database {
    pub async fn new(database_url: String) -> Result<Self, String> {
        let migration_path = Path::new("./migrations");

        let sql_pool = PgPool::connect(&database_url).await.unwrap();

        Migrator::new(migration_path)
            .await
            .unwrap()
            .run(&sql_pool)
            .await
            .unwrap();
        return Ok(Database { database: sql_pool });
    }

    pub async fn insert_ge_price_bulk(
        &self,
        ge_price: &HashMap<i64, GePrice>,
    ) -> Result<(), DatabaseErrors> {
        let now = Utc::now().naive_utc();
        for (k, d) in ge_price.iter() {
            sqlx::query!("insert into ge.price(item, high, high_time, low, low_time, created) values($1, $2, $3, $4, $5, $6)", &k, d.high, d.high_time, d.low, d.low_time, &now).execute(&self.database).await;
        }
        return Ok(());
    }
}

pub enum DatabaseErrors {
    CannotInsert,
}
