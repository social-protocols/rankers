use crate::algs::quality_news::model::{ItemWithRanks, QnSample, QnSampleInterval};
use crate::common::error::AppError;
use sqlx::{query, query_as, query_scalar, Sqlite, Transaction};
use tracing::info;

pub async fn insert_sample_interval(
    tx: &mut Transaction<'_, Sqlite>,
    start_time: i64,
) -> Result<QnSampleInterval, AppError> {
    let sample_interval: QnSampleInterval = query_as(
        "
        insert into qn_sample_interval (start_time)
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

pub async fn get_latest_finished_sample_interval(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<QnSampleInterval, AppError> {
    let latest_interval: QnSampleInterval = query_as(
        "
        select
              max(interval_id)
            , start_time
        from qn_sample_interval
        where interval_id != (
            select max(interval_id)
            from qn_sample_interval
        )
        ",
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(latest_interval)
}

pub async fn get_latest_sample_interval(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<QnSampleInterval, AppError> {
    let latest_interval: QnSampleInterval = query_as(
        "
        select
              interval_id
            , start_time
        from qn_sample_interval
        where interval_id = (
            select max(interval_id)
            from qn_sample_interval
        )
        ",
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(latest_interval)
}

pub async fn get_stats(
    tx: &mut Transaction<'_, Sqlite>,
    interval: &QnSampleInterval,
    sample_time: i64,
) -> Result<Vec<QnSample>, AppError> {
    let stats = query_as::<_, QnSample>(
        "
        with item_pool as (
            select
                  item_id
                , ? as interval_id
                , created_at as submission_time
            from item
            where created_at <= ?
            order by created_at desc
            limit 1500
        )
        , interval as (
            select
                  interval_id
                , start_time
            from qn_sample_interval
            where interval_id = ?
        )
        , stats as (
            select
                  item_id
                , interval_id
                , upvotes
                , upvote_share
            from stats_history
            where interval_id = ?
        )
        , ranks as (
            select
                  item_id
                , interval_id
                , rank_top
                , rank_new
            from rank_history
            where interval_id = ?
        )
        select
              ip.item_id
            , iv.interval_id
            , ? as sample_time
            , ip.submission_time
            , r.rank_top
            , r.rank_new
            , s.upvotes
            , s.upvote_share
        from item_pool ip
        left outer join interval iv
        on ip.interval_id = iv.interval_id
        left outer join ranks r
        on ip.item_id = r.item_id
        and ip.interval_id = r.interval_id
        left outer join stats s
        on ip.item_id = s.item_id
        and ip.interval_id = s.interval_id
        ",
    )
    .bind(interval.interval_id)
    .bind(sample_time)
    .bind(interval.interval_id)
    .bind(interval.interval_id)
    .bind(interval.interval_id)
    .bind(sample_time)
    .fetch_all(&mut **tx)
    .await?;

    info!("Stats: {:?}", stats);

    Ok(stats)
}

pub async fn insert_stats(
    tx: &mut Transaction<'_, Sqlite>,
    stats: &QnSample,
) -> Result<axum::http::StatusCode, AppError> {
    query(
        "
        insert into stats_history (
              item_id
            , interval_id
            , upvotes
            , upvote_share
        )
        values (?, ?, ?, ?)
        ",
    )
    .bind(stats.item_id)
    .bind(stats.interval.interval_id)
    .bind(stats.upvotes)
    .bind(stats.upvote_share)
    .execute(&mut **tx)
    .await?;

    Ok(axum::http::StatusCode::OK)
}

pub async fn calc_stats_in_interval(
    tx: &mut Transaction<'_, Sqlite>,
    interval: &QnSampleInterval,
    sample_time: i64,
) -> Result<Vec<QnSample>, AppError> {
    let sitewide_upvotes =
        get_sitewide_upvotes_in_interval(tx, interval.start_time, sample_time).await? as f32;
    let start_time = interval.start_time;
    let interval_id = interval.interval_id;

    let stats = query_as::<_, QnSample>(
        "
        with upvotes_by_item_in_interval as (
            select
                  item_id
                , count(*) as upvotes
            from vote
            where created_at > ?
            and created_at <= ?
            group by item_id
        )
        , item_pool as (
            select
                  item_id
                , ? as interval_id
                , created_at as submission_time
            from item
            where created_at <= ?
            order by created_at desc
            limit 1500
        )
        select
              i.item_id
            , i.interval_id
            , ? as start_time
            , ? as sample_time
            , i.submission_time
            , r.rank_top
            , r.rank_new
            , u.upvotes
            , u.upvotes / ? as upvote_share
        from item_pool i
        left outer join upvotes_by_item_in_interval u
        on i.item_id = u.item_id
        left outer join rank_history r
        on i.item_id = r.item_id
        and i.interval_id = r.interval_id
        ",
    )
    .bind(start_time)
    .bind(sample_time)
    .bind(interval_id)
    .bind(sample_time)
    .bind(start_time)
    .bind(sample_time)
    .bind(sitewide_upvotes)
    .fetch_all(&mut **tx)
    .await?;

    Ok(stats)
}

pub async fn check_items_exist(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<axum::http::StatusCode, AppError> {
    query_scalar::<_, i32>("select 1 from item limit 1")
        .fetch_one(&mut **tx)
        .await?;

    return Ok(axum::http::StatusCode::OK);
}

pub async fn check_sampling_initialized(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<axum::http::StatusCode, AppError> {
    query_scalar::<_, i32>("select 1 from qn_sample_interval limit 1")
        .fetch_one(&mut **tx)
        .await?;

    return Ok(axum::http::StatusCode::OK);
}
