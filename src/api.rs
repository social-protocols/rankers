use crate::error::AppError;
use crate::model::{HnStats, Item, Observation, Score, ScoredItem, VoteEvent};
use crate::util::now_utc_millis;
use anyhow::Result;
use axum::{extract::State, response::IntoResponse, Json};
use itertools::Itertools;
use sqlx::{query, query_scalar, sqlite::SqlitePool};
use sqlx::{Sqlite, Transaction};

pub async fn health_check() -> Result<axum::http::StatusCode, AppError> {
    Ok(axum::http::StatusCode::OK)
}

pub async fn create_item(
    State(pool): State<SqlitePool>,
    Json(payload): Json<Item>,
) -> Result<impl IntoResponse, AppError> {
    query(
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
    .await?;

    Ok(axum::http::StatusCode::OK)
}

pub async fn send_vote_event(
    State(pool): State<SqlitePool>,
    Json(payload): Json<VoteEvent>,
) -> Result<impl IntoResponse, AppError> {
    query(
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
    .await?;

    Ok(axum::http::StatusCode::OK)
}

// TODO: handle unvotes and revotes
pub async fn get_hacker_news_ranking(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<ScoredItem>>, AppError> {
    let sample_time = now_utc_millis();

    let current_stats: Vec<HnStats> = sqlx::query_as::<_, HnStats>(
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
          , uc.upvotes
        from newest_items ni
        join upvote_counts uc
        on ni.item_id = uc.item_id
        ",
    )
    .fetch_all(&pool)
    .await?;

    let stats_observations: Vec<Observation<HnStats>> = current_stats
        .into_iter()
        .map(|stat| Observation {
            sample_time,
            data: HnStats {
                item_id: stat.item_id,
                submission_time: stat.submission_time,
                upvotes: stat.upvotes,
            },
        })
        .collect();

    let scored_items: Vec<ScoredItem> = stats_observations
        .into_iter()
        .map(|item| ScoredItem {
            item_id: item.data.item_id,
            score: item.score(),
        })
        .collect();

    Ok(Json(scored_items))
}

pub async fn get_ranking_quality_news(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<ScoredItem>>, AppError> {
    let mut tx: Transaction<'_, Sqlite> = pool.begin().await?;

    let latest_sample_time: i64 = query_scalar("select max(sample_time) from stats_history")
        .fetch_one(&mut *tx)
        .await?;

    let current_stats =
        crate::upvote_rate::get_items_with_stats(&mut tx, latest_sample_time).await?;

    let scored_items: Vec<ScoredItem> = current_stats
        .into_iter()
        .sorted_by(|a, b| a.score().partial_cmp(&b.score()).unwrap().reverse())
        .map(|item| ScoredItem {
            item_id: item.data.item_id,
            score: item.score(),
        })
        .collect();

    tx.commit().await?;

    Ok(Json(scored_items))
}
