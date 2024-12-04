use crate::error::AppError;
use crate::model;
use crate::model::Score;
use crate::util::now_millis;
use anyhow::Result;
use axum::{extract::State, response::IntoResponse, Json};
use sqlx::{query, sqlite::SqlitePool};

pub async fn health_check() -> Result<axum::http::StatusCode> {
    Ok(axum::http::StatusCode::OK)
}

pub async fn create_item(
    State(pool): State<SqlitePool>,
    Json(payload): Json<model::Item>,
) -> impl IntoResponse {
    if let Err(_) = query(
        "
        insert into item (
              item_id
            , parent_id
            , created_at
        ) values (?, ?, ?)
        ",
    )
    .bind(payload.item_id)
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
            , item_id
            , vote
            , created_at
        ) values (?, ?, ?, ?)
        ",
    )
    .bind(payload.vote_event_id)
    .bind(payload.item_id)
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
) -> Result<Json<Vec<model::ScoredItem>>, AppError> {
    let sample_time = now_millis();

    let rows: Vec<model::HnStatsObservation> = sqlx::query_as::<_, model::HnStatsObservation>(
        "
        with newest_items as (
            select *
            from item
            order by created_at desc
            limit 1500
        )
        , upvote_counts as (
          select
              item_id
            , count(*) as upvotes
          from vote_event
          where vote = 1
          group by item_id
        )
        select
            ni.item_id
          , ni.created_at as submission_time
          , ? as sample_time
          , uc.upvotes
        from newest_items ni
        join upvote_counts uc
        on ni.item_id = uc.item_id
        ",
    )
    .bind(sample_time)
    .fetch_all(&pool)
    .await?;

    let scored_items: Vec<model::ScoredItem> = rows
        .into_iter()
        .map(|item| model::ScoredItem {
            item_id: item.item_id,
            score: item.score(),
        })
        .collect();

    Ok(Json(scored_items))
}
