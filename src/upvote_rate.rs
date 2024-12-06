use crate::error::AppError;
use crate::model::{ItemWithRanks, QnStatsObservation, Score};
use crate::util::now_millis;
use anyhow::Result;
use itertools::Itertools;
use sqlx::{query, Sqlite, Transaction};

pub async fn sample_ranks(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<axum::http::StatusCode, AppError> {
    // TODO: come up with a better initialization logic

    let n_items: i32 = sqlx::query_scalar("select count(*) from item")
        .fetch_one(&mut **tx)
        .await?;
    if n_items == 0 {
        println!("Waiting for items to rank - Skipping...");
        return Ok(axum::http::StatusCode::OK);
    }

    let rank_history_size: i32 = sqlx::query_scalar("select count(*) from rank_history")
        .fetch_one(&mut **tx)
        .await?;
    if rank_history_size == 0 {
        println!("No ranks recorded yet - Initializing...");
        sqlx::query(
            "
            with items_in_pool as (
                select
                      item_id
                    , unixepoch('subsec') * 1000 as sample_time
                    , row_number() over (order by created_at desc) as rank_top
                    , row_number() over (order by created_at desc) as rank_new
                from item
                limit 1500
            )
            insert into rank_history
            select * from items_in_pool
            ",
        )
        .fetch_all(&mut **tx)
        .await?;
        return Ok(axum::http::StatusCode::OK);
    }

    let previous_sample_time: i64 =
        sqlx::query_scalar("select max(sample_time) from stats_history")
            .fetch_one(&mut **tx)
            .await?;

    let sample_time = now_millis();

    println!("Sampling stats at: {:?}", sample_time);

    let new_stats: Vec<QnStatsObservation> =
        calc_and_insert_newest_stats(tx, sample_time, previous_sample_time).await?;

    calc_and_insert_newest_ranks(tx, &new_stats).await?;

    Ok(axum::http::StatusCode::OK)
}

async fn calc_and_insert_newest_stats(
    tx: &mut Transaction<'_, Sqlite>,
    sample_time: i64,
    previous_sample_time: i64,
) -> Result<Vec<QnStatsObservation>, AppError> {
    let previous_stats: Vec<QnStatsObservation> =
        get_items_with_stats(&mut *tx, previous_sample_time)
            .await
            .unwrap();

    let sitewide_upvotes = get_sitewide_new_upvotes(&mut *tx, previous_sample_time)
        .await
        .unwrap();

    let mut new_stats = Vec::<QnStatsObservation>::new();

    for s in &previous_stats {
        let expected_upvote_share = get_expected_upvote_share(&mut *tx, s.item_id).await?;
        let new_upvotes = get_current_upvote_count(&mut *tx, s.item_id).await?;
        let new_expected_upvotes =
            s.expected_upvotes + (expected_upvote_share * sitewide_upvotes as f32);

        let new_stat = QnStatsObservation {
            item_id: s.item_id,
            submission_time: s.submission_time,
            sample_time,
            upvotes: new_upvotes,
            expected_upvotes: new_expected_upvotes,
        };

        query(
            "
            insert into stats_history (
                  item_id
                , sample_time
                , upvotes
                , expected_upvotes
            ) values (?, ?, ?, ?)
            ",
        )
        .bind(new_stat.item_id)
        .bind(new_stat.sample_time)
        .bind(new_stat.upvotes)
        .bind(new_stat.expected_upvotes)
        .execute(&mut **tx)
        .await?;

        new_stats.push(new_stat);
    }

    Ok(new_stats)
}

async fn get_current_upvote_count(
    tx: &mut Transaction<'_, Sqlite>,
    item_id: i32,
) -> Result<i32, AppError> {
    let upvote_count: i32 = sqlx::query_scalar(
        "
        select count(*)
        from item i
        join vote_event ve
        on i.item_id = ve.item_id
        where vote = 1
        and i.item_id = ?
        ",
    )
    .bind(item_id)
    .fetch_one(&mut **tx)
    .await?;

    Ok(upvote_count)
}

async fn calc_and_insert_newest_ranks(
    tx: &mut Transaction<'_, Sqlite>,
    stats: &Vec<QnStatsObservation>,
) -> Result<axum::http::StatusCode, AppError> {
    let newest_ranks: Vec<ItemWithRanks> = stats
        .into_iter()
        .sorted_by(|a, b| a.score().partial_cmp(&b.score()).unwrap().reverse())
        .enumerate()
        .sorted_by(|(_, a), (_, b)| a.submission_time.partial_cmp(&b.submission_time).unwrap())
        .enumerate()
        .map(|(rank_new, (rank_top, stat))| ItemWithRanks {
            item_id: stat.item_id,
            sample_time: stat.sample_time,
            rank_top: rank_top as i32 + 1,
            rank_new: rank_new as i32 + 1,
        })
        .collect();

    for r in &newest_ranks {
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
        .bind(r.item_id)
        .bind(r.sample_time)
        .bind(r.rank_top)
        .bind(r.rank_new)
        .execute(&mut **tx)
        .await?;
    }

    Ok(axum::http::StatusCode::OK)
}

pub async fn get_items_with_stats(
    tx: &mut Transaction<'_, Sqlite>,
    sample_time: i64,
) -> Result<Vec<QnStatsObservation>, AppError> {
    let newest_stories = sqlx::query_as::<_, QnStatsObservation>(
        "
        with newest_items as (
            select
                  item_id
                , created_at
            from item
            order by created_at desc
            limit 1500
        )
        , latest_stats_history as (
            select *
            from stats_history
            where sample_time = ?
        )
        , with_expected_upvotes as (
            select
                  ni.item_id
                , ni.created_at as submission_time
                , lsh.sample_time
                , coalesce(lsh.upvotes, 0) as upvotes
                , coalesce(lsh.expected_upvotes, 0.0) as expected_upvotes
            from newest_items ni
            left outer join latest_stats_history lsh
            on ni.item_id = lsh.item_id
        )
        select
              item_id
            , submission_time
            , sample_time
            , upvotes
            , expected_upvotes
        from with_expected_upvotes
        order by upvotes desc
        ",
    )
    .bind(sample_time)
    .fetch_all(&mut **tx)
    .await?;

    Ok(newest_stories)
}

async fn get_sitewide_new_upvotes(
    tx: &mut Transaction<'_, Sqlite>,
    previous_sample_time: i64,
) -> Result<i32, AppError> {
    let sitewide_upvotes: i32 = sqlx::query_scalar(
        "
        select count(*)
        from vote_event
        where vote = 1
        and created_at > ?
        ",
    )
    .bind(previous_sample_time)
    .fetch_one(&mut **tx)
    .await?;

    Ok(sitewide_upvotes)
}

// TODO: replace with an actual model of expected upvotes by rank combination (and other factors)
async fn get_expected_upvote_share(
    tx: &mut Transaction<'_, Sqlite>,
    item_id: i32,
) -> Result<f32, AppError> {
    let ranking_entries: i32 = sqlx::query_scalar(
        "
        select count(*)
        from rank_history
        where item_id = ?
        ",
    )
    .bind(item_id)
    .fetch_one(&mut **tx)
    .await?;

    // previously unseen items
    if ranking_entries == 0 {
        return Ok(0.0);
    }

    let expected_upvotes: f32 = sqlx::query_scalar(
        "
        select us.upvote_share_at_rank
        from rank_history rh
        left outer join upvote_share us
        on rh.rank_top = us.rank_top
        where sample_time = (
            select max(sample_time)
            from rank_history
        )
        and item_id = ?
        ",
    )
    .bind(item_id)
    .fetch_one(&mut **tx)
    .await?;

    Ok(expected_upvotes)
}
