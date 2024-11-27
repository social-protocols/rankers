use crate::error::AppError;
use anyhow::Result;
use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use sqlx::{query, sqlite::SqlitePool};

#[derive(Deserialize)]
pub struct VoteEvent {
    vote_event_id: i32,
    post_id: i32,
    vote: i32,
    vote_event_time: i64,
}

#[derive(Deserialize)]
pub struct NewsAggregatorPost {
    post_id: i32,
    parent_id: Option<i32>,
    content: String,
    created_at: i64,
}

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct HNPost {
    post_id: i32,
    upvotes: i32,
    age_hours: f32,
}

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct HNScoredPost {
    post_id: i32,
    score: f32,
}

impl HNScoredPost {
    fn from_hn_post(post: HNPost) -> HNScoredPost {
        HNScoredPost {
            post_id: post.post_id,
            score: (post.upvotes as f32).powf(0.8) / (post.age_hours + 2.0).powf(1.8),
        }
    }
}

pub async fn create_post(
    State(pool): State<SqlitePool>,
    Json(payload): Json<NewsAggregatorPost>,
) -> impl IntoResponse {
    if let Err(_) = query(
        "
        insert into post (
              post_id
            , parent_id
            , content
            , created_at
        ) values (?, ?, ?, ?)
        ",
    )
    .bind(&payload.post_id)
    .bind(payload.parent_id)
    .bind(payload.content)
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
    Json(payload): Json<VoteEvent>,
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
) -> Result<Json<Vec<HNScoredPost>>, AppError> {
    let rows: Vec<HNPost> = sqlx::query_as::<_, HNPost>(
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

    let scored_posts: Vec<HNScoredPost> =
        rows.into_iter().map(HNScoredPost::from_hn_post).collect();

    Ok(Json(scored_posts))
}
