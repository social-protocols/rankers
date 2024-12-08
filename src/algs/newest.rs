use crate::common::{
    error::AppError,
    model::{Observation, RankingPage, Score, ScoredItem},
};
use crate::util::now_utc_millis;
use serde::{Deserialize, Serialize};
use sqlx::{query_as, FromRow, Sqlite, Transaction};

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
struct NewestStats {
    item_id: i32,
    created_at: i64,
}

impl Score for Observation<NewestStats> {
    fn score(&self) -> f32 {
        let age_hours = (self.sample_time - self.data.created_at) as f32 / 60.0 / 60.0;
        1.0 / age_hours
    }
}

pub async fn get_ranking(tx: &mut Transaction<'_, Sqlite>) -> Result<Vec<ScoredItem>, AppError> {
    let sample_time = now_utc_millis();
    let scored_items: Vec<ScoredItem> = query_as::<_, NewestStats>(
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
    .enumerate()
    .map(|(i, obs)| ScoredItem {
        item_id: obs.data.item_id,
        rank: i as i32 + 1,
        page: RankingPage::Newest,
        score: obs.score(),
    })
    .collect();

    Ok(scored_items)
}
