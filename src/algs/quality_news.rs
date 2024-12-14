use crate::common::{
    error::AppError,
    model::{RankingPage, Score, ScoredItem},
    time::now_utc_millis,
};
use anyhow::Result;
use itertools::Itertools;
use model::{ItemWithRanks, QnSample, QnSampleWithPrediction, QnStats};
use sqlx::{Sqlite, Transaction};
use tracing::info;

mod model;
mod repository;

pub async fn get_ranking(tx: &mut Transaction<'_, Sqlite>) -> Result<Vec<ScoredItem>, AppError> {
    repository::check_sampling_initialized(tx).await?;

    let now = now_utc_millis();
    let stats = repository::get_stats(tx, now).await?;

    let scored_items: Vec<ScoredItem> = stats
        .iter()
        .sorted_by(|a, b| {
            a.score()
                .partial_cmp(&b.score())
                .unwrap_or(std::cmp::Ordering::Equal)
                .reverse()
        })
        .enumerate()
        .map(|(rank, stat)| ScoredItem {
            item_id: stat.item_id,
            rank: rank as i32 + 1,
            page: RankingPage::QualityNews,
            score: stat.score(),
        })
        .collect();

    Ok(scored_items)
}

pub async fn record_sample(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<axum::http::StatusCode, AppError> {
    if let Err(_) = repository::check_items_exist(tx).await {
        info!("Waiting for items to rank - Skipping...");
        return Ok(axum::http::StatusCode::OK);
    }

    let sample_time = now_utc_millis();
    info!("Recording quality news sample at: {:?}", sample_time);

    if let Err(_) = repository::check_sampling_initialized(tx).await {
        info!("Initializing quality news sampling...");
        let initial_stats = repository::get_stats(tx, sample_time).await?;
        let next_sampling_interval = repository::insert_sample_interval(tx, sample_time).await?;
        let ranks = calc_ranks(&initial_stats);
        repository::insert_rank_observations(tx, &ranks, &next_sampling_interval).await?;

        return Ok(axum::http::StatusCode::OK);
    }

    let sampling_interval = repository::get_latest_sample_interval(tx).await?;

    // Evaluate stats in current sampling interval
    let sitewide_upvotes =
        repository::get_sitewide_upvotes_in_interval(tx, sample_time, sampling_interval.start_time)
            .await?;
    let sample = repository::get_sample_in_interval(tx, &sampling_interval, sample_time).await?;
    let sample_with_predictions = calc_expected_upvote_shares(&sample, sitewide_upvotes).await?;
    for s in &sample_with_predictions {
        repository::insert_sample(tx, s).await?;
    }
    let updated_stats = repository::get_stats(tx, sample_time).await?;

    // Initialize next sampling interval
    let next_ranks = calc_ranks(&updated_stats);
    let next_sampling_interval = repository::insert_sample_interval(tx, sample_time).await?;
    repository::insert_rank_observations(tx, &next_ranks, &next_sampling_interval).await?;

    Ok(axum::http::StatusCode::OK)
}

async fn calc_expected_upvote_shares(
    stats: &Vec<QnSample>,
    sitewide_upvotes: i32,
) -> Result<Vec<QnSampleWithPrediction>, AppError> {
    let stats_with_predictions: Vec<QnSampleWithPrediction> = stats
        .iter()
        .map(|r| {
            let expected_upvote_share = repository::get_expected_upvote_share(stats.len() as i32);
            QnSampleWithPrediction {
                sample: r.clone(),
                expected_upvotes: sitewide_upvotes as f32 * expected_upvote_share,
                expected_upvote_share,
            }
        })
        .collect();

    Ok(stats_with_predictions)
}

fn calc_ranks(stats: &Vec<QnStats>) -> Vec<ItemWithRanks> {
    stats
        .into_iter()
        .sorted_by(|a, b| {
            a.score()
                .partial_cmp(&b.score())
                .unwrap_or(std::cmp::Ordering::Equal)
                .reverse()
        })
        .enumerate()
        .sorted_by(|(_, a), (_, b)| a.submission_time.cmp(&b.submission_time).reverse())
        .enumerate()
        .map(|(rank_new, (rank_top, obs))| ItemWithRanks {
            item_id: obs.item_id,
            rank_top: rank_top as i32 + 1,
            rank_new: rank_new as i32 + 1,
        })
        .collect()
}
