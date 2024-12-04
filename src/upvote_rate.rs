use crate::model::{PostWithRanks, QnStatsObservation, Score};
use crate::util::now_millis;
use anyhow::Result;
use axum::response::IntoResponse;
use itertools::Itertools;
use sqlx::{query, sqlite::SqlitePool, Sqlite, Transaction};

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
        sqlx::query(
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
        .fetch_all(&mut *tx)
        .await
        .expect("Failed to get posts in pool");
    }

    let previous_sample_time: i64 =
        sqlx::query_scalar("select max(sample_time) from stats_history")
            .fetch_one(&mut *tx)
            .await
            .expect("Failed to get previous sample time");

    let sample_time = now_millis();

    println!("Sampling posts at: {:?}", sample_time);

    let new_stats: Vec<QnStatsObservation> =
        calc_and_insert_newest_stats(&mut tx, sample_time, previous_sample_time)
            .await
            .unwrap();

    calc_and_insert_newest_ranks(&mut tx, &new_stats).await;

    tx.commit()
        .await
        .expect("Failed to commit transaction for rank sampling");

    Ok(axum::http::StatusCode::OK)
}

async fn calc_and_insert_newest_stats(
    tx: &mut Transaction<'_, Sqlite>,
    sample_time: i64,
    previous_sample_time: i64,
) -> Result<Vec<QnStatsObservation>> {
    let previous_stats: Vec<QnStatsObservation> =
        get_posts_with_stats(&mut *tx, previous_sample_time)
            .await
            .unwrap();

    let sitewide_upvotes = get_sitewide_upvotes(&mut *tx).await.unwrap();

    let mut new_stats = Vec::<QnStatsObservation>::new();

    for s in &previous_stats {
        let expected_upvote_share = get_expected_upvote_share(&mut *tx, s.post_id)
            .await
            .unwrap();
        let new_cumulative_upvotes = get_current_upvote_count(&mut *tx, s.post_id).await.unwrap();
        let new_cumulative_expected_upvotes =
            s.cumulative_expected_upvotes + (expected_upvote_share * sitewide_upvotes as f32);

        let new_stat = QnStatsObservation {
            post_id: s.post_id,
            submission_time: s.submission_time,
            sample_time,
            cumulative_upvotes: new_cumulative_upvotes,
            cumulative_expected_upvotes: new_cumulative_expected_upvotes,
        };

        query(
            "
            insert into stats_history (
                  post_id
                , sample_time
                , cumulative_upvotes
                , cumulative_expected_upvotes
            ) values (?, ?, ?, ?)
            ",
        )
        .bind(new_stat.post_id)
        .bind(new_stat.sample_time)
        .bind(new_stat.cumulative_upvotes)
        .bind(new_stat.cumulative_expected_upvotes)
        .execute(&mut **tx)
        .await
        .expect("Failed to insert new stats history entry");

        new_stats.push(new_stat);
    }

    Ok(new_stats)
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

async fn calc_and_insert_newest_ranks(
    tx: &mut Transaction<'_, Sqlite>,
    stats: &Vec<QnStatsObservation>,
) -> impl IntoResponse {
    let newest_ranks: Vec<PostWithRanks> = stats
        .into_iter()
        .sorted_by(|a, b| a.score().partial_cmp(&b.score()).unwrap().reverse())
        .enumerate()
        .map(|(i, stat)| PostWithRanks {
            post_id: stat.post_id,
            sample_time: stat.sample_time,
            rank_top: i as i32 + 1,
        })
        .collect();

    for r in &newest_ranks {
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

async fn get_posts_with_stats(
    tx: &mut Transaction<'_, Sqlite>,
    sample_time: i64,
) -> Result<Vec<QnStatsObservation>> {
    let newest_stories = sqlx::query_as::<_, QnStatsObservation>(
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
            where sample_time = ?
        )
        , with_cumulative_expected_upvotes as (
            select
                  np.post_id
                , np.created_at as submission_time
                , lsh.sample_time
                , coalesce(lsh.cumulative_upvotes, 0) as cumulative_upvotes
                , coalesce(lsh.cumulative_expected_upvotes, 0.0) as cumulative_expected_upvotes
            from newest_posts np
            left outer join latest_stats_history lsh
            on np.post_id = lsh.post_id
        )
        select
              post_id
            , submission_time
            , sample_time
            , cumulative_upvotes
            , cumulative_expected_upvotes
        from with_cumulative_expected_upvotes
        order by cumulative_upvotes desc
        ",
    )
    .bind(sample_time)
    .fetch_all(&mut **tx)
    .await
    .expect("Failed to get newest stories");

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
        and post_id = ?
        ",
    )
    .bind(post_id)
    .fetch_one(&mut **tx)
    .await
    .expect("Failed to get expected upvotes at current tick");

    Ok(expected_upvotes)
}
