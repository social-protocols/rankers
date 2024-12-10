use crate::algs::quality_news::model::{ItemWithRanks, QnStats};
use crate::common::error::AppError;
use crate::common::model::{Item, Observation};
use sqlx::{query, query_as, query_scalar, Sqlite, Transaction};

pub async fn get_current_upvote_count(
    tx: &mut Transaction<'_, Sqlite>,
    item_id: i32,
) -> Result<i32, AppError> {
    let upvote_count: i32 = query_scalar(
        "
        select count(*)
        from item_pool ip
        join vote v
        on ip.item_id = v.item_id
        where vote = 1
        and ip.item_id = ?
        ",
    )
    .bind(item_id)
    .fetch_one(&mut **tx)
    .await?;

    Ok(upvote_count)
}

pub async fn get_sitewide_new_upvotes(
    tx: &mut Transaction<'_, Sqlite>,
    sample_time: i64,
    previous_sample_time: i64,
) -> Result<i32, AppError> {
    let sitewide_upvotes: i32 = query_scalar(
        "
        select count(*)
        from vote
        where vote = 1
        and created_at < ?
        and created_at > ?
        ",
    )
    .bind(sample_time)
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
pub fn get_expected_upvote_share(n_items: i32) -> f32 {
    1.0 / n_items as f32
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

pub async fn get_stats(
    tx: &mut Transaction<'_, Sqlite>,
    sample_time: i64,
) -> Result<Vec<Observation<QnStats>>, AppError> {
    let stats_observations = query_as::<_, QnStats>(
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
