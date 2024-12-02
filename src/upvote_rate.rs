// SUMMARY:
// ============================================================================================
// NOTES:
// - sitewide upvotes is just count of all upvotes that happened after the last sampletime
// - we don't necessarily want users of the service to have to track where votes occured, so we
// still need a scheme to estimate upvote share per page
//
// TODO:
// - [ ] calculate cumulative expected upvotes as previous cumulative expected upvotes + expected
// upvotes at rank combination at current tick
// - [ ] create a model that estimates page coefficients (schedule can be more drawn out, eg daily)

use crate::model::{PostWithRanks, StatsObservation};
use anyhow::Result;
use axum::response::IntoResponse;
use sqlx::{query, sqlite::SqlitePool};
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn sample_ranks(
    pool: &SqlitePool,
) -> Result<axum::http::StatusCode, axum::http::StatusCode> {
    // TODO: come up with a better initialization logic

    let n_posts: i32 = sqlx::query_scalar("select count(*) from post")
        .fetch_one(pool)
        .await
        .expect("Couldn't get post count");
    if n_posts == 0 {
        println!("Waiting for posts to rank - Skipping...");
        return Ok(axum::http::StatusCode::OK);
    }

    let rank_history_size: i32 = sqlx::query_scalar("select count(*) from rank_history")
        .fetch_one(pool)
        .await
        .expect("Couldn't get rank history size");
    if rank_history_size == 0 {
        println!("No ranks recorded yet - Initializing...");
        let _: Vec<PostWithRanks> = sqlx::query_as(
            "
            with posts_in_pool as (
                select
                      post_id
                    , 0 as sample_time
                    , row_number() over (order by created_at desc) as rank_top
                from post
                limit 1500
            )
            insert into rank_history
            select * from posts_in_pool
            returning *
            ",
        )
        .fetch_all(pool)
        .await
        .expect("Failed to get posts in pool");
    }

    let sample_time: i64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Couldn't get current time to record sample time")
        .as_millis() as i64;

    println!("Sampling posts at: {:?}", sample_time);

    let _ = insert_stats_from_current_tick(&pool, sample_time).await;

    // TODO: eventually, rank by upvote rate
    let ranks = get_ranks_from_previous_tick(&pool).await.unwrap();
    let new_ranks: Vec<PostWithRanks> = ranks
        .iter()
        .map(|ranked_post| PostWithRanks {
            post_id: ranked_post.post_id,
            sample_time,
            rank_top: ranked_post.rank_top,
        })
        .collect();
    insert_ranks_from_current_tick(&pool, &new_ranks).await;

    Ok(axum::http::StatusCode::OK)
}

async fn insert_stats_from_current_tick(
    pool: &SqlitePool,
    sample_time: i64,
) -> Result<axum::http::StatusCode, axum::http::StatusCode> {
    let stats: Vec<StatsObservation> = get_posts_with_stats_for_current_tick(&pool).await.unwrap();

    for s in &stats {
        let newly_observed_expected_upvotes = get_expected_upvotes_at_tick(&pool, s.post_id)
            .await
            .unwrap();
        let current_upvote_count = get_current_upvote_count(&pool, s.post_id).await.unwrap();
        if let Err(_) = query(
            "
            insert into stats_history (
                  post_id
                , sample_time
                , cumulative_upvotes
                , cumulative_expected_upvotes
            ) values (?, ?, ?, ?)
            ",
        )
        .bind(s.post_id)
        .bind(sample_time)
        .bind(current_upvote_count)
        .bind(s.cumulative_expected_upvotes + newly_observed_expected_upvotes)
        .execute(pool)
        .await
        {
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    Ok(axum::http::StatusCode::OK)
}

async fn get_current_upvote_count(pool: &SqlitePool, post_id: i32) -> Result<i32> {
    let upvote_count: i32 = sqlx::query_scalar(
        "
        select count(*)
        from post p
        join vote_event ve
        on p.post_id = ve.post_id
        where vote = 1
        and p.post_id = ?
        ",
    )
    .bind(post_id)
    .fetch_one(pool)
    .await
    .expect("Failed to get current vote count");

    Ok(upvote_count)
}

async fn insert_ranks_from_current_tick(
    pool: &SqlitePool,
    ranks: &Vec<PostWithRanks>,
) -> impl IntoResponse {
    for r in ranks {
        if let Err(_) = query(
            "
            insert into rank_history (
                  post_id
                , sample_time
                , rank_top
            ) values (?, ?, ?)
            ",
        )
        .bind(r.post_id)
        .bind(r.sample_time)
        .bind(r.rank_top)
        .execute(pool)
        .await
        {
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    Ok(axum::http::StatusCode::OK)
}

async fn get_posts_with_stats_for_current_tick(pool: &SqlitePool) -> Result<Vec<StatsObservation>> {
    let newest_stories = sqlx::query_as::<_, StatsObservation>(
        "
        with newest_posts as (
            select
                  post_id
                , created_at
            from post
            order by created_at desc
            limit 1500
        )
        , latest_stats_history as (
            select *
            from stats_history
            where sample_time = (
                select max(sample_time)
                from stats_history
            )
        )
        , with_cumulative_expected_upvotes as (
            select
                  np.post_id
                , lsh.sample_time
                , coalesce(lsh.cumulative_upvotes, 0) as cumulative_upvotes
                , coalesce(lsh.cumulative_expected_upvotes, 0.0) as cumulative_expected_upvotes
            from newest_posts np
            left outer join latest_stats_history lsh
            on np.post_id = lsh.post_id
        )
        select
              post_id
            , sample_time
            , cumulative_upvotes
            , cumulative_expected_upvotes
        from with_cumulative_expected_upvotes
        order by cumulative_upvotes desc
        ",
    )
    .fetch_all(pool)
    .await
    .expect("Failed to get newest stories");

    println!("Currently tracked stories: {:?}", newest_stories.len());

    Ok(newest_stories)
}

// TODO: replace with an actual model of expected upvotes by rank combination (and other factors)
async fn get_expected_upvotes_at_tick(pool: &SqlitePool, post_id: i32) -> Result<f32> {
    let expected_upvotes_by_post: f32 = sqlx::query_scalar(
        "
        select ubr.avg_upvotes
        from rank_history rh
        left outer join upvotes_by_rank ubr
        on rh.rank_top = ubr.rank_top
        where sample_time = (
            select max(sample_time)
            from rank_history
        )
        and post_id = ?
        ",
    )
    .bind(post_id)
    .fetch_one(pool)
    .await
    .expect("Failed to get expected upvotes at current tick");

    Ok(expected_upvotes_by_post)
}

// TODO: remove once ranking by upvote rate works
async fn get_ranks_from_previous_tick(pool: &SqlitePool) -> Result<Vec<PostWithRanks>> {
    let ranks: Vec<PostWithRanks> = sqlx::query_as(
        "
        select
              post_id
            , sample_time
            , rank_top
        from rank_history
        where rank_top <= 90
        and sample_time = (
          select max(sample_time)
          from rank_history
        )
        ",
    )
    .fetch_all(pool)
    .await
    .expect("Failed to get ranks for current tick");

    Ok(ranks)
}
