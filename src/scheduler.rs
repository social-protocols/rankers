use crate::error::AppError;
use crate::upvote_rate;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

pub async fn start_scheduler(pool: Arc<SqlitePool>) -> Result<(), AppError> {
    let scheduler = JobScheduler::new().await?;

    let job_pool = Arc::clone(&pool);

    // Job runs every minute
    scheduler
        .add(Job::new_async("0 * * * * *", move |_uuid, _l| {
            let job_pool = Arc::clone(&job_pool);
            Box::pin(async move {
                upvote_rate::sample_ranks(&job_pool).await;
            })
        })?)
        .await?;

    scheduler.start().await?;

    Ok(())
}
