use anyhow::Result;
use dotenv::dotenv;
use http_server::start_http_server;

mod api;
mod database;
mod error;
mod http_server;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let pool = database::setup_database()
        .await
        .expect("Failed to create database pool");

    start_http_server(pool).await?;

    Ok(())
}
