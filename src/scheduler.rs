use crate::algs::quality_news;
use crate::common::error::AppError;
use sqlx::{Sqlite, SqlitePool, Transaction};
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::error;

pub async fn start_scheduler(pool: Arc<SqlitePool>) -> Result<(), AppError> {
    let scheduler = JobScheduler::new().await?;

    let job_pool = Arc::clone(&pool);

    // TODO: change to once a minute for production
    let cron_expression = "1/5 * * * * *";

    scheduler
        .add(Job::new_async(cron_expression, move |_uuid, _l| {
            let job_pool = Arc::clone(&job_pool);
            Box::pin(async move {
                let mut tx: Transaction<'_, Sqlite> =
                    job_pool.begin().await.expect("Couldn't create transaction");
                match quality_news::record_sample(&mut tx).await {
                    Ok(_) => {
                        tx.commit().await.unwrap();
                    }
                    Err(e) => {
                        tx.rollback().await.unwrap();
                        error!("Error recording sample: {:?}", e);
                    }
                };
            })
        })?)
        .await?;

    scheduler.start().await?;

    Ok(())
}
