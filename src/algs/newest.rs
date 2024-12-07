use crate::common::{
    error::AppError,
    model::{Observation, Score, ScoredItem},
};
use crate::util::now_utc_millis;
use serde::{Deserialize, Serialize};
use sqlx::{query_as, FromRow, Sqlite, Transaction};

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
struct NewestItem {
    pub item_id: i32,
    pub created_at: i64,
}

impl Score for Observation<NewestItem> {
    fn score(&self) -> f32 {
        let age_hours = (self.sample_time - self.data.created_at) as f32 / 60.0 / 60.0;
        1.0 / age_hours
    }
}

pub async fn get_ranking(tx: &mut Transaction<'_, Sqlite>) -> Result<Vec<ScoredItem>, AppError> {
    let sample_time = now_utc_millis();
    let scored_items: Vec<ScoredItem> = query_as::<_, NewestItem>(
        "
        select
              item_id
            , created_at
        from item
        order by created_at desc
        limit 1500
        ",
    )
    .fetch_all(&mut **tx)
    .await?
    .iter()
    .map(|item| Observation {
        sample_time,
        data: item.clone(),
    })
    .map(|obs| ScoredItem {
        item_id: obs.data.item_id,
        score: obs.score(),
    })
    .collect();

    Ok(scored_items)
}