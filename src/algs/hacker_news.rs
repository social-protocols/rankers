use crate::common::{
    error::AppError,
    model::{Observation, RankingPage, Score, ScoredItem},
};
use crate::util::now_utc_millis;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Sqlite, Transaction};

#[derive(FromRow, Debug, Serialize, Deserialize)]
pub struct HnStats {
    pub item_id: i32,
    pub submission_time: i64,
    pub upvotes: i32,
}

impl Score for Observation<HnStats> {
    fn score(&self) -> f32 {
        let age_hours = (self.sample_time - self.data.submission_time) as f32 / 60.0 / 60.0;
        (self.data.upvotes as f32).powf(0.8) / (age_hours + 2.0).powf(1.8)
    }
}

pub async fn get_ranking(tx: &mut Transaction<'_, Sqlite>) -> Result<Vec<ScoredItem>, AppError> {
    let sample_time = now_utc_millis();

    let current_stats: Vec<HnStats> = sqlx::query_as::<_, HnStats>(
        "
        with newest_items as (
            select *
            from item
            order by created_at desc
            limit 1500
        )
        , upvote_counts as (
          select
              item_id
            , count(*) as upvotes
          from vote
          where vote = 1
          group by item_id
        )
        select
            ni.item_id
          , ni.created_at as submission_time
          , uc.upvotes
        from newest_items ni
        join upvote_counts uc
        on ni.item_id = uc.item_id
        ",
    )
    .fetch_all(&mut **tx)
    .await?;

    let stats_observations: Vec<Observation<HnStats>> = current_stats
        .into_iter()
        .map(|stat| Observation {
            sample_time,
            data: HnStats {
                item_id: stat.item_id,
                submission_time: stat.submission_time,
                upvotes: stat.upvotes,
            },
        })
        .collect();

    let scored_items: Vec<ScoredItem> = stats_observations
        .into_iter()
        .enumerate()
        .map(|(i, item)| ScoredItem {
            item_id: item.data.item_id,
            rank: i as i32 + 1,
            page: RankingPage::HackerNews,
            score: item.score(),
        })
        .collect();

    Ok(scored_items)
}
