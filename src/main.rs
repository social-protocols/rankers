use anyhow::Result;
use dotenv::dotenv;
use std::sync::Arc;

mod algs;
mod api;
mod common;
mod database;
mod http_server;
mod scheduler;
mod util;

#[tokio::main]
async fn main() -> Result<(), common::error::AppError> {
    dotenv().ok();
    let pool: sqlx::SqlitePool = database::setup_database().await?;
    scheduler::start_scheduler(Arc::clone(&Arc::new(pool.clone()))).await?;
    http_server::start_http_server(pool).await?;

    Ok(())
}
