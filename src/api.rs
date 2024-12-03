use crate::error::AppError;
use crate::model;
use anyhow::Result;
use axum::{extract::State, response::IntoResponse, Json};
use sqlx::{query, sqlite::SqlitePool};

pub async fn health_check() -> Result<axum::http::StatusCode, AppError> {
    Ok(axum::http::StatusCode::OK)
}

pub async fn create_post(
    State(pool): State<SqlitePool>,
    Json(payload): Json<model::NewsAggregatorPost>,
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
    .bind(&payload.post_id)
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
            , vote_event_time
        ) values (?, ?, ?, ?)
        ",
    )
    .bind(&payload.vote_event_id)
    .bind(payload.post_id)
    .bind(payload.vote)
    .bind(payload.vote_event_time)
    .execute(&pool)
    .await
    {
        return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(axum::http::StatusCode::OK)
}

pub async fn get_hacker_news_ranking(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<model::HNScoredPost>>, AppError> {
    let rows: Vec<model::HNPost> = sqlx::query_as::<_, model::HNPost>(
        "
        with upvote_counts as (
          select
            post_id
            , count(*) as upvotes
          from vote_event
          where vote = 1
          group by post_id
        )
        , age_hours as (
          select
            p.post_id
            , (unixepoch('subsec') * 1000 - p.created_at) / 1000 / 60 / 60 as age_hours
          from post p
        )
        select
          p.post_id as post_id
          , uc.upvotes as upvotes
          , ah.age_hours as age_hours
        from post p
        join upvote_counts uc
        on p.post_id = uc.post_id
        join age_hours ah
        on p.post_id = ah.post_id
        where p.parent_id is null
        order by p.created_at desc
        limit 1500
        ",
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch row");

    let scored_posts: Vec<model::HNScoredPost> = rows
        .into_iter()
        .map(model::HNScoredPost::from_hn_post)
        .collect();

    Ok(Json(scored_posts))
}
