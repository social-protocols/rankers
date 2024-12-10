use crate::algs::quality_news::model::{ItemWithRanks, QnStats};
use crate::common::error::AppError;
use crate::common::model::{Item, Observation};
use sqlx::{query, query_as, query_scalar, Sqlite, Transaction};
use std::collections::HashMap;

pub async fn get_current_upvote_count_by_item(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<HashMap<i32, i32>, AppError> {
    let upvote_counts: Vec<(i32, i32)> = query_as(
        "
        select ip.item_id, count(*)
        from item_pool ip
        join vote v
        on ip.item_id = v.item_id
        where vote = 1
        ",
    )
    .fetch_all(&mut **tx)
    .await?;

    let mut upvote_count_by_item = HashMap::new();
    for (item_id, upvote_count) in upvote_counts {
        upvote_count_by_item.insert(item_id.clone(), upvote_count.clone());
    }

    Ok(upvote_count_by_item)
}

pub async fn get_sitewide_new_upvotes(
    tx: &mut Transaction<'_, Sqlite>,
    previous_sample_time: i64,
) -> Result<i32, AppError> {
    let sitewide_upvotes: i32 = query_scalar(
        "
        select count(*)
        from vote
        where vote = 1
        and created_at > ?
        ",
    )
    .bind(previous_sample_time)
    .fetch_one(&mut **tx)
    .await?;

    Ok(sitewide_upvotes)
}

pub async fn get_items_in_pool(tx: &mut Transaction<'_, Sqlite>) -> Result<Vec<Item>, AppError> {
    let items: Vec<Item> = query_as("select * from item_pool")
        .fetch_all(&mut **tx)
        .await?;

    Ok(items)
}

// TODO: create poisson regression model to estimate upvote share
pub fn get_expected_upvote_share_by_item(items: &Vec<Item>) -> HashMap<i32, f32> {
    if items.is_empty() {
        return HashMap::new();
    }

    let default_upvote_share = 1.0 / items.len() as f32;

    let mut upvote_share_by_item = HashMap::new();
    for item in items.iter() {
        upvote_share_by_item.insert(item.item_id, default_upvote_share);
    }

    upvote_share_by_item
}

pub async fn insert_stats_observations(
    tx: &mut Transaction<'_, Sqlite>,
    stats: &Vec<Observation<QnStats>>,
) -> Result<axum::http::StatusCode, AppError> {
    for stat in stats {
        query(
            "
            insert into stats_history (
                  item_id
                , sample_time
                , upvotes
                , upvote_share
                , expected_upvotes
            ) values (?, ?, ?, ?, ?)
            ",
        )
        .bind(stat.data.item_id)
        .bind(stat.sample_time)
        .bind(stat.data.upvotes)
        .bind(stat.data.upvote_share)
        .bind(stat.data.expected_upvotes)
        .execute(&mut **tx)
        .await?;
    }

    Ok(axum::http::StatusCode::OK)
}

pub async fn insert_rank_observations(
    tx: &mut Transaction<'_, Sqlite>,
    rank_observations: &Vec<Observation<ItemWithRanks>>,
) -> Result<axum::http::StatusCode, AppError> {
    for r in rank_observations {
        query(
            "
            insert into rank_history (
                  item_id
                , sample_time
                , rank_top
                , rank_new
            ) values (?, ?, ?, ?)
            ",
        )
        .bind(r.data.item_id)
        .bind(r.sample_time)
        .bind(r.data.rank_top)
        .bind(r.data.rank_new)
        .execute(&mut **tx)
        .await?;
    }

    Ok(axum::http::StatusCode::OK)
}

pub async fn initialize_rank_history(
    tx: &mut Transaction<'_, Sqlite>,
    sample_time: i64,
) -> Result<axum::http::StatusCode, AppError> {
    query(
        "
        with ranked_item_pool as (
            select
                  item_id
                , ? as sample_time
                , row_number() over (order by created_at desc) as rank_top
                , row_number() over (order by created_at desc) as rank_new
            from item_pool
        )
        insert into rank_history
        select * from ranked_item_pool
        ",
    )
    .bind(sample_time)
    .fetch_all(&mut **tx)
    .await?;

    return Ok(axum::http::StatusCode::OK);
}

pub async fn get_stats(
    tx: &mut Transaction<'_, Sqlite>,
    sample_time: i64,
) -> Result<Vec<Observation<QnStats>>, AppError> {
    let stats_observations = sqlx::query_as::<_, QnStats>(
        "
        with stats_at_sample_time as (
            select *
            from stats_history
            where sample_time = ?
        )
        , stats as (
            select
                  ip.item_id
                , ip.created_at as submission_time
                , coalesce(sast.upvotes, 0) as upvotes
                , coalesce(sast.upvote_share, 0.0) as upvote_share
                , coalesce(sast.expected_upvotes, 0.0) as expected_upvotes
            from item_pool ip
            left outer join stats_at_sample_time sast
            on ip.item_id = sast.item_id
        )
        select
              item_id
            , submission_time
            , upvotes
            , upvote_share
            , expected_upvotes
        from stats
        order by upvotes desc
        ",
    )
    .bind(sample_time)
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(|stat| Observation {
        sample_time,
        data: QnStats {
            item_id: stat.item_id,
            submission_time: stat.submission_time,
            upvotes: stat.upvotes,
            upvote_share: stat.upvote_share,
            expected_upvotes: stat.expected_upvotes,
        },
    })
    .collect();

    Ok(stats_observations)
}
