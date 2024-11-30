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

use crate::model::{PostWithRanks, PostWithStats};
use anyhow::Result;
use axum::{extract::State, response::IntoResponse};
use sqlx::{query, sqlite::SqlitePool};
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn sample_ranks(State(pool): State<SqlitePool>) -> impl IntoResponse {
    let newest_stories: Vec<PostWithStats> = get_newest_posts_with_stats(State(pool.clone()))
        .await
        .unwrap();

    let sample_time: i64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as i64;

    println!("Sampling posts at: {:?}", sample_time);

    get_ranks_from_previous_tick(State(pool.clone()))
        .await
        .unwrap();

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
        .execute(&pool)
        .await
        {
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    Ok(axum::http::StatusCode::OK)
}

async fn get_ranks_from_previous_tick(
    State(pool): State<SqlitePool>,
) -> Result<Vec<PostWithRanks>> {
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
    .fetch_all(&pool)
    .await
    .expect("Failed to get ranks for current tick");

    if ranks.len() == 0 {
        println!("No ranks recorded yet - Initializing...");
        let _posts_in_pool: Vec<PostWithRanks> = sqlx::query_as(
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
            ",
        )
        .fetch_all(&pool)
        .await
        .expect("Failed to get posts in pool");
    } else {
        for r in &ranks {
            println!("Rank: {:?}", r);
        }
    }

    Ok(vec![])
}

async fn get_newest_posts_with_stats(State(pool): State<SqlitePool>) -> Result<Vec<PostWithStats>> {
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
    .fetch_all(&pool)
    .await
    .expect("Failed to get newest stories");

    Ok(newest_stories)
}
