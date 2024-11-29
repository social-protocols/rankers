use anyhow::Result;
use axum::extract::State;
use dotenv::dotenv;
use http_server::start_http_server;
use tokio_cron_scheduler::{Job, JobScheduler};

mod api;
mod calc_sample_space_stats;
mod database;
mod error;
mod http_server;
mod model;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let scheduler = JobScheduler::new().await?;

    let pool = database::setup_database()
        .await
        .expect("Failed to create database pool");

    scheduler
        .add(Job::new_async("0 * * * * *", |_uuid, _l| {
            Box::pin(async move {
                let pool = database::setup_database()
                    .await
                    .expect("Failed to create database pool");
                calc_sample_space_stats::sample_ranks(State(pool)).await;
            })
        })?)
        .await?;

    scheduler.start().await?;

    start_http_server(pool).await?;

    Ok(())
}
