// SUMMARY:
// ============================================================================================
// FOR EACH STORY:
// --------------------------------------------------------------------------------------------
// REQUIRED (at sampletime):
// - current best rank (according to this metric)
// - current upvote count
// - current cumulative expected upvotes
// --------------------------------------------------------------------------------------------
// DERIVED:
// - upvote_rate
// --------------------------------------------------------------------------------------------
// REQUIRED:
// - a starting condition: if the rank_samples table is empty, there needs to be a default
// - then, the scheme is properly initialized
// - or better yet, for the first sample, all ranks are NULL
// - and then, ranks are calculated by another tie break (e.g., created_at)
//
// NOTES:
// - sitewide upvotes is just count of all upvotes that happened after the last sampletime
// - we don't necessarily want users of the service to have to track where votes occured, so we
// still needa scheme to estimate upvote share per page
//
// AGENDA:
// - [ ] create a model that estimates page coefficients (schedule can be more drawn out, eg once a
// week)
//
// TODO: get ranks for each of the posts in the current sample pool
// - so we need some sort of a get ranks function
// - This is different from the Quality News setup. In QN, we observe the HN pages, we don't
// assign ranks ourselves. Here, we are in control of the entire system. We can get the ranks
// from within our system.
//
// TODO: get previous cumulative expected upvotes, then calculate expected upvotes for this
// tick -> add, and voila, we have cumulative_expected_upvotes

use crate::model::{PostWithRanks, PostWithStats, UpvotesByRank};
use anyhow::Result;
use axum::response::IntoResponse;
use sqlx::{query, sqlite::SqlitePool};
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn sample_ranks(pool: &SqlitePool) -> impl IntoResponse {
    let newest_stories: Vec<PostWithStats> = get_newest_posts_with_stats(&pool).await.unwrap();

    let sample_time: i64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as i64;

    println!("Sampling posts at: {:?}", sample_time);

    let ranks = get_ranks_from_previous_tick(&pool).await.unwrap();

    // TODO: eventually, rank by upvote rate
    let new_ranks: Vec<PostWithRanks> = ranks.iter().map(|ranked_post| PostWithRanks { post_id: ranked_post.post_id, sample_time, rank_top: ranked_post.rank_top }).collect();
    insert_ranks_from_current_tick(&pool, &new_ranks).await;

    let upvotes_by_rank = get_avg_upvotes_by_rank(&pool).await.unwrap();
    println!("Upvotes by Rank at {:?}", sample_time);
    for ur in &upvotes_by_rank {
        println!("{:?}", ur);
    }

    for ns in &newest_stories {
        if let Err(_) = query(
            "
            insert into stats_history (
                  post_id
                , sample_time
                , submission_time
                , upvote_count
            ) values (?, ?, ?, ?)
            ",
        )
        .bind(ns.post_id)
        .bind(sample_time)
        .bind(ns.submission_time)
        .bind(ns.upvote_count)
        .execute(pool)
        .await
        {
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    Ok(axum::http::StatusCode::OK)
}

async fn insert_ranks_from_current_tick(pool: &SqlitePool, ranks: &Vec<PostWithRanks>) -> impl IntoResponse {
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

    if ranks.len() == 0 {
        println!("No ranks recorded yet - Initializing...");
        let init_ranks: Vec<PostWithRanks> = sqlx::query_as(
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
        return Ok(init_ranks)
    }

    Ok(ranks)
}

async fn get_newest_posts_with_stats(pool: &SqlitePool) -> Result<Vec<PostWithStats>> {
    let newest_stories = sqlx::query_as::<_, PostWithStats>(
        "
        with newest_posts as (
            select
                post_id
                , created_at
            from post
            order by created_at desc
            limit 1500
        )
        , vote_counts as (
            select
                np.post_id
                , np.created_at as submission_time
                , count(*) as upvote_count
            from newest_posts np
            join vote_event ve
            on np.post_id = ve.post_id
            where ve.vote = 1
            group by np.post_id
        )
        select *
        from vote_counts
        order by upvote_count desc
        ",
    )
    .fetch_all(pool)
    .await
    .expect("Failed to get newest stories");

    Ok(newest_stories)
}

async fn get_avg_upvotes_by_rank(pool: &SqlitePool) -> Result<Vec<UpvotesByRank>> {
    let upvotes_by_rank = sqlx::query_as::<_, UpvotesByRank>(
        "
        with upvotes_in_time_window as (
          select
            post_id
            , sample_time
            , upvote_count - lag(upvote_count) over (
                partition by post_id
                order by sample_time
            ) as upvotes_in_time_window
          from stats_history
        )
        , upvote_window as (
          select
            post_id
            , sample_time
            , coalesce(upvotes_in_time_window, 0) as upvotes_in_time_window
          from upvotes_in_time_window
        )
        , ranks_with_upvote_count as (
          select
            rh.post_id
            , rh.sample_time
            , rh.rank_top
            , uw.upvotes_in_time_window
          from rank_history rh
          join upvote_window uw
          on rh.post_id = uw.post_id
          and rh.sample_time = uw.sample_time
        )
        select
          rank_top
          , avg(upvotes_in_time_window) as avg_upvotes
        from ranks_with_upvote_count
        group by rank_top
        ",
    )
    .fetch_all(pool)
    .await
    .expect("Failed to get upvotes by rank");

    Ok(upvotes_by_rank)
}

