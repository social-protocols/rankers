use crate::common::{
    error::AppError,
    model::{RankingPage, Score, ScoredItem},
    time::now_utc_millis,
};
use serde::{Deserialize, Serialize};
use sqlx::{query_as, FromRow, Sqlite, Transaction};

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
struct NewestStats {
    item_id: i32,
    sample_time: i64,
    submission_time: i64,
}

impl Score for NewestStats {
    fn score(&self) -> f32 {
        let age_hours = (self.sample_time - self.submission_time) as f32 / 60.0 / 60.0;
        1.0 / age_hours
    }
}

pub async fn get_ranking(tx: &mut Transaction<'_, Sqlite>) -> Result<Vec<ScoredItem>, AppError> {
    let sample_time = now_utc_millis();
    let scored_items: Vec<ScoredItem> = query_as::<_, NewestStats>(
        "
        select
              item_id
            , ? as sample_time
            , created_at as submission_time
        from item
        order by created_at desc
        limit 1500
        ",
    )
    .bind(sample_time)
    .fetch_all(&mut **tx)
    .await?
    .iter()
    .enumerate()
    .map(|(i, stat)| ScoredItem {
        item_id: stat.item_id,
        rank: i as i32 + 1,
        page: RankingPage::Newest,
        score: stat.score(),
    })
    .collect();

    Ok(scored_items)
}
