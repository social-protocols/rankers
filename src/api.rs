use crate::algs::{hacker_news, newest, quality_news};
use crate::common::{
    error::AppError,
    model::{Item, ScoredItem, VoteEvent},
};
use anyhow::Result;
use axum::{extract::State, response::IntoResponse, Json};
use sqlx::{query, sqlite::SqlitePool, Sqlite, Transaction};

pub async fn health_check() -> Result<axum::http::StatusCode, AppError> {
    Ok(axum::http::StatusCode::OK)
}

pub async fn register_item(
    State(pool): State<SqlitePool>,
    Json(payload): Json<Item>,
) -> Result<impl IntoResponse, AppError> {
    query(
        "
        insert into item (
              item_id
            , author_id
            , parent_id
            , created_at
        ) values (?, ?, ?, ?)
        ",
    )
    .bind(payload.item_id)
    .bind(payload.author_id)
    .bind(payload.parent_id)
    .bind(payload.created_at)
    .execute(&pool)
    .await?;

    Ok(axum::http::StatusCode::OK)
}

pub async fn register_vote_event(
    State(pool): State<SqlitePool>,
    Json(payload): Json<VoteEvent>,
) -> Result<impl IntoResponse, AppError> {
    query(
        "
        insert into vote_event (
              vote_event_id
            , item_id
            , user_id
            , vote
            , created_at
        ) values (?, ?, ?, ?, ?)
        ",
    )
    .bind(payload.vote_event_id)
    .bind(payload.item_id)
    .bind(payload.user_id)
    .bind(payload.vote)
    .bind(payload.created_at)
    .execute(&pool)
    .await?;

    Ok(axum::http::StatusCode::OK)
}

pub async fn get_hacker_news_ranking(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<ScoredItem>>, AppError> {
    let mut tx: Transaction<'_, Sqlite> = pool.begin().await?;
    let scored_items = hacker_news::get_ranking(&mut tx).await?;
    tx.commit().await?;

    Ok(Json(scored_items))
}

pub async fn get_ranking_quality_news(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<ScoredItem>>, AppError> {
    let mut tx: Transaction<'_, Sqlite> = pool.begin().await?;
    let scored_items = quality_news::get_ranking(&mut tx).await?;
    tx.commit().await?;

    Ok(Json(scored_items))
}

pub async fn get_ranking_newest(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<ScoredItem>>, AppError> {
    let mut tx: Transaction<'_, Sqlite> = pool.begin().await?;
    let scored_items = newest::get_ranking(&mut tx).await?;
    tx.commit().await?;

    Ok(Json(scored_items))
}
