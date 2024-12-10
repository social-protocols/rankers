use crate::common::{
    error::AppError,
    model::{RankingPage, ScoredItem},
    time::now_utc_millis,
};
use anyhow::Result;
use itertools::Itertools;
use model::{ItemWithRanks, QnStats};
use sqlx::{query_scalar, Sqlite, Transaction};
use tracing::info;

mod model;
mod repository;

pub async fn get_ranking(tx: &mut Transaction<'_, Sqlite>) -> Result<Vec<ScoredItem>, AppError> {
    let recorded_stats_exist: bool = query_scalar("select exists (select 1 from stats_history)")
        .fetch_one(&mut **tx)
        .await?;

    if !recorded_stats_exist {
        info!("No stats recorded yet, returning empty QN ranking");
        return Ok(vec![]);
    }

    let now = now_utc_millis();

    let sampling_interval = repository::get_latest_sample_interval(tx).await?;

    let scored_items: Vec<ScoredItem> =
        repository::get_stats_in_interval(tx, sampling_interval.start_time, now)
            .await?
            .into_iter()
            // TODO: eventually sort by score, not upvotes:
            // .sorted_by(|a, b| a.score().partial_cmp(&b.score()).unwrap().reverse())
            .sorted_by(|a, b| a.upvotes.partial_cmp(&b.upvotes).unwrap().reverse())
            .enumerate()
            .map(|(i, item)| ScoredItem {
                item_id: item.item_id,
                rank: i as i32 + 1,
                page: RankingPage::QualityNews,
                score: item.upvotes as f32,
            })
            .collect();

    Ok(scored_items)
}

pub async fn record_sample(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<axum::http::StatusCode, AppError> {
    if let Err(_) = check_items_exist(tx).await {
        info!("Waiting for items to rank - Skipping...");
        return Ok(axum::http::StatusCode::OK);
    }

    let sample_time = now_utc_millis();
    info!("Recording quality news sample at: {:?}", sample_time);

    if let Err(_) = check_sampling_initialized(tx).await {
        info!("Initializing quality news sampling...");
        let sample_interval = repository::insert_sample_interval(tx, sample_time).await?;
        let stats = repository::get_stats_in_interval(tx, 0, sample_time).await?;
        let ranks = calc_ranks(&stats);
        repository::insert_rank_observations(tx, &ranks, &sample_interval).await?;
        return Ok(axum::http::StatusCode::OK);
    }

    let sampling_interval = repository::get_latest_sample_interval(tx).await?;

    // TODO: use this item pool to get stats
    // let item_pool = repository::get_item_pool(tx, sample_time).await?;

    let stats =
        repository::get_stats_in_interval(tx, sampling_interval.start_time, sample_time).await?;
    // TODO: predict expected upvote share -> use in ranking

    let ranks = calc_ranks(&stats);
    let next_sample_interval = repository::insert_sample_interval(tx, sample_time).await?;
    repository::insert_rank_observations(tx, &ranks, &next_sample_interval).await?;

    Ok(axum::http::StatusCode::OK)
}

async fn check_items_exist(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<axum::http::StatusCode, AppError> {
    query_scalar::<_, i32>("select 1 from item limit 1")
        .fetch_one(&mut **tx)
        .await?;

    return Ok(axum::http::StatusCode::OK);
}

async fn check_sampling_initialized(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<axum::http::StatusCode, AppError> {
    query_scalar::<_, i32>("select 1 from qn_sample_interval limit 1")
        .fetch_one(&mut **tx)
        .await?;

    return Ok(axum::http::StatusCode::OK);
}

// async fn calc_stats(
//     tx: &mut Transaction<'_, Sqlite>,
//     sample_time: i64,
//     previous_stat: Observation<QnStats>,
//     sitewide_upvotes: i32,
// ) -> Result<Observation<QnStats>, AppError> {
//     // TODO: move to expected_upvote_share calculation
//     // n_items: i32,
//     // let expected_upvote_share = repository::get_expected_upvote_share(n_items);
//     // let new_expected_upvotes =
//     //     previous_stat.data.expected_upvotes + (expected_upvote_share * (sitewide_upvotes as f32));
// }

fn calc_ranks(stats: &Vec<QnStats>) -> Vec<ItemWithRanks> {
    stats
        .into_iter()
        // TODO
        // .sorted_by(|a, b| a.score().partial_cmp(&b.score()).unwrap().reverse())
        .sorted_by(|a, b| a.upvotes.partial_cmp(&b.upvotes).unwrap().reverse())
        .enumerate()
        .sorted_by(|(_, a), (_, b)| a.submission_time.partial_cmp(&b.submission_time).unwrap())
        .enumerate()
        .map(|(rank_new, (rank_top, item))| ItemWithRanks {
            item_id: item.item_id,
            rank_top: rank_top as i32 + 1,
            rank_new: rank_new as i32 + 1,
        })
        .collect()
}
