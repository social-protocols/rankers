use crate::common::{
    error::AppError,
    model::{RankingPage, Score, ScoredItem},
    time::now_utc_millis,
};
use anyhow::Result;
use itertools::Itertools;
use model::{ItemWithRanks, QnSample, QnSampleWithPrediction};
use sqlx::{Sqlite, Transaction};
use tracing::info;

mod model;
mod repository;

pub async fn get_ranking(tx: &mut Transaction<'_, Sqlite>) -> Result<Vec<ScoredItem>, AppError> {
    repository::check_sampling_initialized(tx).await?;

    let now = now_utc_millis();
    info!("getting quality news ranking at: {:?}", now);
    let sampling_interval = repository::get_latest_finished_sample_interval(tx).await?;

    let stats = repository::get_stats(tx, &sampling_interval, now).await?;
    info!("stats: {:?}", stats);

    let stats_with_predictions = calc_expected_upvote_shares(&stats).await?;

    let scored_items: Vec<ScoredItem> = stats_with_predictions
        .iter()
        .sorted_by(|a, b| a.score().partial_cmp(&b.score()).unwrap().reverse())
        .enumerate()
        .map(|(rank, stat)| ScoredItem {
            item_id: stat.sample.item_id,
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
        let next_sampling_interval = repository::insert_sample_interval(tx, sample_time).await?;

        let stats =
            repository::calc_stats_in_interval(tx, &next_sampling_interval, sample_time).await?;
        let stats_with_predictions = calc_expected_upvote_shares(&stats).await?;

        let ranks = calc_ranks(&stats_with_predictions);
        repository::insert_rank_observations(tx, &ranks, &next_sampling_interval).await?;

        return Ok(axum::http::StatusCode::OK);
    }

    let sampling_interval = repository::get_latest_sample_interval(tx).await?;

    // Evaluate stats in current sampling interval
    let stats = repository::calc_stats_in_interval(tx, &sampling_interval, sample_time).await?;
    for s in &stats {
        repository::insert_stats(tx, s).await?;
    }

    // Initialize next sampling interval
    let stats_with_predictions = calc_expected_upvote_shares(&stats).await?;
    let next_ranks = calc_ranks(&stats_with_predictions);
    let next_sampling_interval = repository::insert_sample_interval(tx, sample_time).await?;
    repository::insert_rank_observations(tx, &next_ranks, &next_sampling_interval).await?;

    Ok(axum::http::StatusCode::OK)
}

async fn calc_expected_upvote_shares(
    stats: &Vec<QnSample>,
) -> Result<Vec<QnSampleWithPrediction>, AppError> {
    let stats_with_predictions: Vec<QnSampleWithPrediction> = stats
        .iter()
        .map(|r| QnSampleWithPrediction {
            sample: r.clone(),
            expected_upvote_share: repository::get_expected_upvote_share(r.item_id),
        })
        .collect();

    Ok(stats_with_predictions)
}

fn calc_ranks(stats: &Vec<QnSampleWithPrediction>) -> Vec<ItemWithRanks> {
    stats
        .into_iter()
        .sorted_by(|a, b| a.score().partial_cmp(&b.score()).unwrap().reverse())
        .enumerate()
        .sorted_by(|(_, a), (_, b)| {
            a.sample
                .submission_time
                .partial_cmp(&b.sample.submission_time)
                .unwrap()
        })
        .enumerate()
        .map(|(rank_new, (rank_top, obs))| ItemWithRanks {
            item_id: obs.sample.item_id,
            interval_id: obs.sample.interval.interval_id,
            rank_top: rank_top as i32 + 1,
            rank_new: rank_new as i32 + 1,
        })
        .collect()
}
