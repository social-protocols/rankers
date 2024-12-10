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
    if let Err(_) = query_scalar::<_, i32>("select 1 from item limit 1")
        .fetch_one(&mut **tx)
        .await
    {
        info!("Waiting for items to rank - Skipping...");
        return Ok(axum::http::StatusCode::OK);
    }

    let sample_time = now_utc_millis();
    info!("Sampling stats at: {:?}", sample_time);

    match query_scalar("select max(sample_time) from stats_history")
        .fetch_optional(&mut **tx)
        .await?
    {
        // initialize stats and ranks histories
        None => {
            let items = repository::get_items_in_pool(tx).await?;

            let mut initial_stats = Vec::<Observation<QnStats>>::new();
            for item in &items {
                let init_observation = Observation {
                    sample_time,
                    data: QnStats {
                        item_id: item.item_id,
                        submission_time: item.created_at,
                        upvotes: repository::get_current_upvote_count(tx, item.item_id).await?,
                        upvote_share: 0.0,
                        expected_upvotes: 0.0,
                    },
                };
                initial_stats.push(init_observation);
            }
            repository::insert_stats_observations(tx, &initial_stats).await?;

            let initial_ranks = calc_updated_ranks(&initial_stats);
            repository::insert_rank_observations(tx, &initial_ranks).await?;
        }
        // update stats and ranks histories
        Some(previous_sample_time) => {
            let previous_stats: Vec<Observation<QnStats>> =
                repository::get_stats(&mut *tx, previous_sample_time).await?;

            let n_items = previous_stats.len() as i32;

            let sitewide_upvotes =
                repository::get_sitewide_new_upvotes(&mut *tx, sample_time, previous_sample_time)
                    .await?;

            let mut updated_stats = Vec::<Observation<QnStats>>::new();
            for stat in previous_stats {
                let updated_stat: Observation<QnStats> =
                    calc_updated_stats(tx, sample_time, stat, sitewide_upvotes, n_items).await?;
                updated_stats.push(updated_stat);
            }
            repository::insert_stats_observations(tx, &updated_stats).await?;

            let new_ranks = calc_updated_ranks(&updated_stats);
            repository::insert_rank_observations(tx, &new_ranks).await?;
        }
    }

    Ok(axum::http::StatusCode::OK)
}

async fn calc_updated_stats(
    tx: &mut Transaction<'_, Sqlite>,
    sample_time: i64,
    previous_stat: Observation<QnStats>,
    sitewide_upvotes: i32,
    n_items: i32,
) -> Result<Observation<QnStats>, AppError> {
    if sitewide_upvotes == 0 {
        return Ok(Observation {
            sample_time,
            data: QnStats {
                item_id: previous_stat.data.item_id,
                submission_time: previous_stat.data.submission_time,
                upvotes: previous_stat.data.upvotes,
                upvote_share: 0.0,
                expected_upvotes: 0.0,
            },
        });
    }

    let expected_upvote_share = repository::get_expected_upvote_share(n_items);
    let new_upvotes = repository::get_current_upvote_count(tx, previous_stat.data.item_id).await?;
    let new_expected_upvotes =
        previous_stat.data.expected_upvotes + (expected_upvote_share * (sitewide_upvotes as f32));
    let actual_upvote_share =
        (new_upvotes - previous_stat.data.upvotes) as f32 / sitewide_upvotes as f32;

    Ok(Observation {
        sample_time,
        data: QnStats {
            item_id: previous_stat.data.item_id,
            submission_time: previous_stat.data.submission_time,
            upvotes: new_upvotes,
            upvote_share: actual_upvote_share,
            expected_upvotes: new_expected_upvotes,
        },
    })
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
