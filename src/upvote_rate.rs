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

use crate::model::{Post, PostWithRanks, StatsObservation};
use anyhow::Result;
use axum::response::IntoResponse;
use sqlx::{query, sqlite::SqlitePool, Sqlite, Transaction};
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn sample_ranks(
    pool: &SqlitePool,
) -> Result<axum::http::StatusCode, axum::http::StatusCode> {
    // TODO: come up with a better initialization logic

    let mut tx = pool
        .begin()
        .await
        .expect("Failed to create transaction for rank sampling");

    let n_posts: i32 = sqlx::query_scalar("select count(*) from post")
        .fetch_one(&mut *tx)
        .await
        .expect("Couldn't get post count");
    if n_posts == 0 {
        println!("Waiting for posts to rank - Skipping...");
        return Ok(axum::http::StatusCode::OK);
    }

    let rank_history_size: i32 = sqlx::query_scalar("select count(*) from rank_history")
        .fetch_one(&mut *tx)
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
        .fetch_all(&mut *tx)
        .await
        .expect("Failed to get posts in pool");
    }

    let sample_time: i64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Couldn't get current time to record sample time")
        .as_millis() as i64;

    println!("Sampling posts at: {:?}", sample_time);

    let _ = insert_stats_from_current_tick(&mut tx, sample_time).await;

    let mut current_stats_for_ranking = get_posts_with_stats_for_current_tick(&mut tx)
        .await
        .unwrap();

    current_stats_for_ranking.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap().reverse());

    let new_ranks: Vec<PostWithRanks> = current_stats_for_ranking
        .iter()
        .enumerate()
        .map(|(i, stat)| PostWithRanks {
            post_id: stat.post_id,
            sample_time,
            rank_top: i as i32 + 1,
        })
        .collect();

    insert_ranks_from_current_tick(&mut tx, &new_ranks).await;

    tx.commit()
        .await
        .expect("Failed to commit transaction for rank sampling");

    Ok(axum::http::StatusCode::OK)
}

async fn insert_stats_from_current_tick(
    tx: &mut Transaction<'_, Sqlite>,
    sample_time: i64,
) -> Result<axum::http::StatusCode, axum::http::StatusCode> {
    let stats: Vec<StatsObservation> = get_posts_with_stats_for_current_tick(&mut *tx)
        .await
        .unwrap();
    let sitewide_upvotes = get_sitewide_upvotes(&mut *tx).await.unwrap();

    for s in &stats {
        let expected_upvote_share = get_expected_upvote_share(&mut *tx, s.post_id)
            .await
            .unwrap();
        let current_upvote_count = get_current_upvote_count(&mut *tx, s.post_id).await.unwrap();
        let current_expected_upvote_count =
            s.cumulative_expected_upvotes + (expected_upvote_share * sitewide_upvotes as f32);
        let post: Post = sqlx::query_as(
            "
            select *
            from post
            where post_id = ?
            ",
        )
        .bind(s.post_id)
        .fetch_one(&mut **tx)
        .await
        .expect("Failed to get current post");
        let age_hours = (sample_time - post.created_at) as f32 / 60.0 / 60.0;

        let score = calc_score(
            age_hours,
            current_upvote_count,
            current_expected_upvote_count,
        );

        let _result = query(
            "
            insert into stats_history (
                  post_id
                , sample_time
                , cumulative_upvotes
                , cumulative_expected_upvotes
                , score
            ) values (?, ?, ?, ?, ?)
            ",
        )
        .bind(s.post_id)
        .bind(sample_time)
        .bind(current_upvote_count)
        .bind(current_expected_upvote_count)
        .bind(score)
        .execute(&mut **tx)
        .await
        .expect("Failed to insert new stats history entry");
    }

    Ok(axum::http::StatusCode::OK)
}

fn calc_score(age: f32, upvotes: i32, expected_upvotes: f32) -> f32 {
    // TODO: sane default for 0.0 expected upvotes
    let estimated_upvote_rate: f32 = if expected_upvotes != 0.0 {
        upvotes as f32 / expected_upvotes
    } else {
        0.0
    };
    let numerator = (age * estimated_upvote_rate).powf(0.8);
    let denominator = (age + 2.0).powf(1.8);
    numerator / denominator
}

async fn get_current_upvote_count(tx: &mut Transaction<'_, Sqlite>, post_id: i32) -> Result<i32> {
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
    .fetch_one(&mut **tx)
    .await
    .expect("Failed to get current vote count");

    Ok(upvote_count)
}

async fn insert_ranks_from_current_tick(
    tx: &mut Transaction<'_, Sqlite>,
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
        .execute(&mut **tx)
        .await
        {
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    Ok(axum::http::StatusCode::OK)
}

async fn get_posts_with_stats_for_current_tick(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<Vec<StatsObservation>> {
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
                , coalesce(lsh.score, 0.0) as score
            from newest_posts np
            left outer join latest_stats_history lsh
            on np.post_id = lsh.post_id
        )
        select
              post_id
            , sample_time
            , cumulative_upvotes
            , cumulative_expected_upvotes
            , score
        from with_cumulative_expected_upvotes
        order by cumulative_upvotes desc
        ",
    )
    .fetch_all(&mut **tx)
    .await
    .expect("Failed to get newest stories");

    println!("Currently tracked stories: {:?}", newest_stories.len());

    Ok(newest_stories)
}

async fn get_sitewide_upvotes(tx: &mut Transaction<'_, Sqlite>) -> Result<i32> {
    let sitewide_upvotes: i32 = sqlx::query_scalar(
        "
        with upvotes_at_sample_time as (
            select
                post_id
                , sample_time
                , coalesce(
                    cumulative_upvotes - lag(cumulative_upvotes) over (
                        partition by post_id
                        order by sample_time
                    ),
                    0
                ) as upvotes_at_sample_time
            from stats_history
        )
        select sum(upvotes_at_sample_time) as sitewide_upvotes
        from upvotes_at_sample_time
        where sample_time = (
            select max(sample_time)
            from upvotes_at_sample_time
        )
        ",
    )
    .fetch_one(&mut **tx)
    .await
    .expect("Failed to get sitewide upvotes at current tick");

    Ok(sitewide_upvotes)
}

// TODO: replace with an actual model of expected upvotes by rank combination (and other factors)
async fn get_expected_upvote_share(tx: &mut Transaction<'_, Sqlite>, post_id: i32) -> Result<f32> {
    let previously_unranked: i32 = sqlx::query_scalar(
        "
        select count(*) = 0
        from rank_history
        where post_id = ?
        ",
    )
    .bind(post_id)
    .fetch_one(&mut **tx)
    .await
    .expect("Failed to determine previous ranking status");

    if previously_unranked == 1 {
        return Ok(0.0);
    }

    let expected_upvotes_by_post: f32 = sqlx::query_scalar(
        "
        select us.upvote_share_at_rank
        from rank_history rh
        left outer join upvote_share us
        on rh.rank_top = us.rank_top
        where sample_time = (
            select max(sample_time)
            from rank_history
        )
        and post_id = ?
        ",
    )
    .bind(post_id)
    .fetch_one(&mut **tx)
    .await
    .expect("Failed to get expected upvotes at current tick");

    Ok(expected_upvotes_by_post)
}
