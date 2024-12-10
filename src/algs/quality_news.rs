use crate::common::{
    error::AppError,
    model::{Observation, RankingPage, Score, ScoredItem},
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

    let latest_sample_time: i64 = query_scalar("select max(sample_time) from stats_history")
        .fetch_one(&mut **tx)
        .await?;

    let scored_items: Vec<ScoredItem> = repository::get_stats(tx, latest_sample_time)
        .await?
        .into_iter()
        .sorted_by(|a, b| a.score().partial_cmp(&b.score()).unwrap().reverse())
        .enumerate()
        .map(|(i, item)| ScoredItem {
            item_id: item.data.item_id,
            rank: i as i32 + 1,
            page: RankingPage::QualityNews,
            score: item.score(),
        })
        .collect();

    Ok(scored_items)
}

pub async fn sample_stats(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<axum::http::StatusCode, AppError> {
    let sample_time = now_utc_millis();
    info!("Sampling stats at: {:?}", sample_time);

    if let Err(_) = query_scalar::<_, i32>("select 1 from item limit 1")
        .fetch_one(&mut **tx)
        .await
    {
        info!("Waiting for items to rank - Skipping...");
        return Ok(axum::http::StatusCode::OK);
    }

    if let Err(_) = query_scalar::<_, i32>("select 1 from rank_history limit 1")
        .fetch_one(&mut **tx)
        .await
    {
        info!("No ranks recorded yet - Initializing...");

        // TODO: come up with a better initialization logic
        repository::initialize_rank_history(tx, sample_time).await?;
        return Ok(axum::http::StatusCode::OK);
    }

    let previous_sample_time: i64 =
        sqlx::query_scalar("select max(sample_time) from stats_history")
            .fetch_one(&mut **tx)
            .await?;

    let previous_stats: Vec<Observation<QnStats>> =
        repository::get_stats(&mut *tx, previous_sample_time).await?;

    let sitewide_upvotes =
        repository::get_sitewide_new_upvotes(&mut *tx, previous_sample_time).await?;

    let updated_stats: Vec<Observation<QnStats>> =
        calc_updated_stats(tx, sample_time, previous_stats, sitewide_upvotes).await?;
    repository::insert_stats_observations(tx, &updated_stats).await?;

    let new_ranks = calc_updated_ranks(&updated_stats);
    repository::insert_rank_observations(tx, &new_ranks).await?;

    Ok(axum::http::StatusCode::OK)
}

async fn calc_updated_stats(
    tx: &mut Transaction<'_, Sqlite>,
    sample_time: i64,
    previous_stats: Vec<Observation<QnStats>>,
    sitewide_upvotes: i32,
) -> Result<Vec<Observation<QnStats>>, AppError> {
    let items = repository::get_items_in_pool(tx).await?;
    let expected_upvote_share_by_item = repository::get_expected_upvote_share_by_item(&items);
    let upvote_count_by_item = repository::get_current_upvote_count_by_item(&mut *tx).await?;

    let updated_stats: Vec<Observation<QnStats>> = previous_stats
        .iter()
        .map(|stat| {
            let expected_upvote_share = expected_upvote_share_by_item
                .get(&stat.data.item_id)
                .unwrap(); // TODO
            let new_upvotes = *upvote_count_by_item.get(&stat.data.item_id).unwrap(); // TODO
            let new_expected_upvotes =
                stat.data.expected_upvotes + (expected_upvote_share * (sitewide_upvotes as f32));
            let actual_upvote_share = if sitewide_upvotes != 0 {
                new_upvotes as f32 / sitewide_upvotes as f32
            } else {
                0.0
            };

            Observation {
                sample_time,
                data: QnStats {
                    item_id: stat.data.item_id,
                    submission_time: stat.data.submission_time,
                    upvotes: new_upvotes,
                    upvote_share: actual_upvote_share,
                    expected_upvotes: new_expected_upvotes,
                },
            }
        })
        .collect();

    Ok(updated_stats)
}

fn calc_updated_ranks(stats: &Vec<Observation<QnStats>>) -> Vec<Observation<ItemWithRanks>> {
    stats
        .into_iter()
        .sorted_by(|a, b| a.score().partial_cmp(&b.score()).unwrap().reverse())
        .enumerate()
        .sorted_by(|(_, a), (_, b)| {
            a.data
                .submission_time
                .partial_cmp(&b.data.submission_time)
                .unwrap()
        })
        .enumerate()
        .map(|(rank_new, (rank_top, stat))| Observation {
            sample_time: stat.sample_time,
            data: ItemWithRanks {
                item_id: stat.data.item_id,
                rank_top: rank_top as i32 + 1,
                rank_new: rank_new as i32 + 1,
            },
        })
        .collect()
}
