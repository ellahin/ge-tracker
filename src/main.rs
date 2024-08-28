mod repo;
mod routes;

use crate::repo::data::osrs::Osrs;
use crate::repo::sql::Database;

use std::env;

use axum::{
    routing::{get, post},
    Router,
};
use dotenvy::dotenv;
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    dotenv().expect(".env file not found");

    if env::var("DATABASE_URL").is_err() {
        panic!("DATABASE_URL not in environment vars");
    }
    let database = Database::new(env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "with_axum_htmx_askama=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let osrs = Osrs::new(database.clone()).await.unwrap();

    let state = AppState { database, osrs };

    info!("initializing router...");
    let assets_path = std::env::current_dir().unwrap();

    let router = Router::new()
        .route("/", get(routes::index::get))
        .route("/highalch", get(routes::highalch::get))
        .nest_service(
            "/public",
            ServeDir::new(format!("{}/public", assets_path.to_str().unwrap())),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

#[derive(Clone)]
pub struct AppState {
    database: Database,
    osrs: Osrs,
}
