use crate::error::AppError;
use crate::model;
use crate::model::Score;
use anyhow::Result;
use axum::{extract::State, response::IntoResponse, Json};
use sqlx::{query, sqlite::SqlitePool};
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn health_check() -> Result<axum::http::StatusCode, AppError> {
    Ok(axum::http::StatusCode::OK)
}

pub async fn create_post(
    State(pool): State<SqlitePool>,
    Json(payload): Json<model::Post>,
) -> impl IntoResponse {
    if let Err(_) = query(
        "
        insert into post (
              post_id
            , parent_id
            , created_at
        ) values (?, ?, ?)
        ",
    )
    .bind(payload.post_id)
    .bind(payload.parent_id)
    .bind(payload.created_at)
    .execute(&pool)
    .await
    {
        return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(axum::http::StatusCode::OK)
}

pub async fn send_vote_event(
    State(pool): State<SqlitePool>,
    Json(payload): Json<model::VoteEvent>,
) -> Result<impl IntoResponse, axum::http::StatusCode> {
    if let Err(_) = query(
        "
        insert into vote_event (
              vote_event_id
            , post_id
            , vote
            , created_at
        ) values (?, ?, ?, ?)
        ",
    )
    .bind(payload.vote_event_id)
    .bind(payload.post_id)
    .bind(payload.vote)
    .bind(payload.created_at)
    .execute(&pool)
    .await
    {
        return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(axum::http::StatusCode::OK)
}

// TODO: handle unvotes and revotes
pub async fn get_hacker_news_ranking(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<model::ScoredPost>>, AppError> {
    let sample_time: i64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Couldn't get current time to record sample time")
        .as_millis() as i64;

    let rows: Vec<model::HnStatsObservation> = sqlx::query_as::<_, model::HnStatsObservation>(
        "
        with newest_posts as (
            select *
            from post
            order by created_at desc
            limit 1500
        )
        , upvote_counts as (
          select
              post_id
            , count(*) as upvotes
          from vote_event
          where vote = 1
          group by post_id
        )
        select
            np.post_id
          , np.created_at as submission_time
          , ? as sample_time
          , uc.upvotes
        from newest_posts np
        join upvote_counts uc
        on np.post_id = uc.post_id
        ",
    )
    .bind(sample_time)
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch row");

    let scored_posts: Vec<model::ScoredPost> = rows
        .into_iter()
        .map(|elem| model::ScoredPost {
            post_id: elem.post_id,
            score: elem.score(),
        })
        .collect();

    Ok(Json(scored_posts))
}
