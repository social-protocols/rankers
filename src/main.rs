use anyhow::Result;
use dotenv::dotenv;
use http_server::start_http_server;
use std::sync::Arc;

mod api;
mod database;
mod error;
mod http_server;
mod model;
mod scheduler;
mod upvote_rate;
mod util;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let pool: sqlx::SqlitePool = database::setup_database().await?;

    scheduler::start_scheduler(Arc::clone(&Arc::new(pool.clone())))
        .await
        .expect("Couldn't setup scheduler");

    start_http_server(pool).await?;

    Ok(())
}
