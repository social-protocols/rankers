use crate::error::AppError;
use crate::upvote_rate;
use sqlx::SqlitePool;
use sqlx::{Sqlite, Transaction};
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

pub async fn start_scheduler(pool: Arc<SqlitePool>) -> Result<(), AppError> {
    let scheduler = JobScheduler::new().await?;

    let job_pool = Arc::clone(&pool);

    // TODO: change to once a minute for production
    let cron_expression = "1/12 * * * * *";

    scheduler
        .add(Job::new_async(cron_expression, move |_uuid, _l| {
            let job_pool = Arc::clone(&job_pool);
            Box::pin(async move {
                // TODO: start and commit transaction here and rollback if sample_ranks returns Err

                let mut tx: Transaction<'_, Sqlite> =
                    job_pool.begin().await.expect("Couldn't create transaction");
                match upvote_rate::sample_ranks(&mut tx).await {
                    Ok(_) => {
                        tx.commit().await.unwrap();
                    }
                    Err(e) => {
                        tx.rollback().await.unwrap();
                        eprintln!("Error sampling ranks: {:?}", e);
                    }
                };
            })
        })?)
        .await?;

    scheduler.start().await?;

    Ok(())
}
