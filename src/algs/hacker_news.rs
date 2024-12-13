use crate::common::time::now_utc_millis;
use crate::common::{
    error::AppError,
    model::{RankingPage, Score, ScoredItem},
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Sqlite, Transaction};

#[derive(FromRow, Debug, Serialize, Deserialize)]
pub struct HnStats {
    pub item_id: i32,
    pub sample_time: i64,
    pub submission_time: i64,
    pub upvotes: i32,
}

impl Score for HnStats {
    fn score(&self) -> f32 {
        let age_hours = (self.sample_time - self.submission_time) as f32 / 60.0 / 60.0;
        (self.upvotes as f32).powf(0.8) / (age_hours + 2.0).powf(1.8)
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
          , ? as sample_time
          , ni.created_at as submission_time
          , coalesce(uc.upvotes, 0) as upvotes
        from newest_items ni
        left outer join upvote_counts uc
        on ni.item_id = uc.item_id
        ",
    )
    .bind(sample_time)
    .fetch_all(&mut **tx)
    .await?;

    let stats: Vec<HnStats> = current_stats
        .into_iter()
        .map(|stat| HnStats {
            item_id: stat.item_id,
            sample_time,
            submission_time: stat.submission_time,
            upvotes: stat.upvotes,
        })
        .collect();

    let scored_items: Vec<ScoredItem> = stats
        .into_iter()
        .enumerate()
        .map(|(i, stat)| ScoredItem {
            item_id: stat.item_id,
            rank: i as i32 + 1,
            page: RankingPage::HackerNews,
            score: stat.score(),
        })
        .sorted_by(|a, b| {
            a.score
                .partial_cmp(&b.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .reverse()
        })
        .collect();

    Ok(scored_items)
}
