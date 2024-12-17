use anyhow::Result;
use dotenv::dotenv;
use std::sync::Arc;
use tracing_subscriber;
use sqlx::migrate::Migrator;

mod algs {
    pub mod hacker_news;
    pub mod newest;
    pub mod quality_news;
}
mod api;
mod common {
    pub mod error;
    pub mod model;
    pub mod time;
}
mod database;
mod http_server;
mod scheduler;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[tokio::main]
async fn main() -> Result<(), common::error::AppError> {
    dotenv().ok();

    tracing_subscriber::fmt::init();

    let pool: sqlx::SqlitePool = database::setup_database().await?;

    MIGRATOR.run(&pool).await?;

    scheduler::start_scheduler(Arc::clone(&Arc::new(pool.clone()))).await?;
    http_server::start_http_server(pool).await?;

    Ok(())
}
