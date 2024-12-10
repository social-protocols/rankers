use crate::algs::quality_news::model::{ItemWithRanks, QnSampleInterval, QnStats};
use crate::common::error::AppError;
use crate::common::model::Item;
use sqlx::{query, query_as, query_scalar, Sqlite, Transaction};

pub async fn insert_sample_interval(
    tx: &mut Transaction<'_, Sqlite>,
    start_time: i64,
) -> Result<QnSampleInterval, AppError> {
    let sample_interval: QnSampleInterval = query_as(
        "
        insert into qn_sample_interval
        values (?)
        returning *
        ",
    )
    .bind(start_time)
    .fetch_one(&mut **tx)
    .await?;

    Ok(sample_interval)
}

pub async fn get_sitewide_upvotes_in_interval(
    tx: &mut Transaction<'_, Sqlite>,
    sample_time: i64,
    previous_sample_time: i64,
) -> Result<i32, AppError> {
    let sitewide_upvotes: i32 = query_scalar(
        "
        select count(*)
        from vote
        where vote = 1
        and created_at <= ?
        and created_at > ?
        ",
    )
    .bind(sample_time)
    .bind(previous_sample_time)
    .fetch_one(&mut **tx)
    .await?;

    Ok(sitewide_upvotes)
}

#[allow(dead_code)]
pub async fn get_item_pool(
    tx: &mut Transaction<'_, Sqlite>,
    sample_time: i64,
) -> Result<Vec<Item>, AppError> {
    let items: Vec<Item> = query_as(
        "
        select *
        from item
        where created_at <= ?
        order by created_at desc
        limit 1500;
        ",
    )
    .bind(sample_time)
    .fetch_all(&mut **tx)
    .await?;

    Ok(items)
}

// TODO: create poisson regression model to estimate upvote share
#[allow(dead_code)]
pub fn get_expected_upvote_share(n_items: i32) -> f32 {
    1.0 / n_items as f32
}

pub async fn insert_rank_observations(
    tx: &mut Transaction<'_, Sqlite>,
    ranked_items: &Vec<ItemWithRanks>,
    sample_interval: &QnSampleInterval,
) -> Result<axum::http::StatusCode, AppError> {
    for r in ranked_items {
        query(
            "
            insert into rank_history (
                  item_id
                , interval_id
                , rank_top
                , rank_new
            ) values (?, ?, ?, ?)
            ",
        )
        .bind(r.item_id)
        .bind(sample_interval.interval_id)
        .bind(r.rank_top)
        .bind(r.rank_new)
        .execute(&mut **tx)
        .await?;
    }

    Ok(axum::http::StatusCode::OK)
}

pub async fn get_latest_sample_interval(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<QnSampleInterval, AppError> {
    let latest_interval: QnSampleInterval = query_as(
        "
        select *
        from qn_sample_interval
        where interval_id = (
            selectm max(interval_id)
            from qn_sample_interval
        )
        ",
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(latest_interval)
}

pub async fn get_stats_in_interval(
    tx: &mut Transaction<'_, Sqlite>,
    start_time: i64,
    sample_time: i64,
) -> Result<Vec<QnStats>, AppError> {
    let sitewide_upvotes = get_sitewide_upvotes_in_interval(tx, start_time, sample_time).await?;

    let stats = query_as::<_, QnStats>(
        "
        with upvotes_by_item_in_interval as (
            select item_id, count(*) as upvotes
            from vote
            where created_at > ?
            and created_at <= ?
        )
        select
            i.item_id
            , i.created_at
            , ubii.upvotes
            , ubii.upvotes / ?
        from item i
        left outer join upvotes_by_item_in_interval ubii
        where i.created_at > ?
        and i.created_at <= ?
        ",
    )
    .bind(start_time)
    .bind(sample_time)
    .bind(sitewide_upvotes as f32)
    .fetch_all(&mut **tx)
    .await?;

    Ok(stats)
}
