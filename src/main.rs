use axum::{
    routing::{get, post},
    response::{Json, IntoResponse},
    Router,
    extract::State,
};
use serde::Deserialize;
use dotenv::dotenv;
use std::env;
use sqlx::{sqlite::SqlitePool, query};
use anyhow::Result;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to create pool.");

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/from_db", get(from_db))
        .route("/create_post", post(create_post))
        .route("/send_vote_event", post(send_vote_event))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn from_db(State(pool): State<SqlitePool>) -> impl IntoResponse {
    let row: (i32,) =
        sqlx::query_as("select 42")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch row");

    format!("Result: {}", row.0)
}

#[derive(Deserialize)]
struct VoteEvent {
    vote_event_id: i32,
    post_id: i32,
    vote: i32,
    vote_event_time: i32,
}

#[derive(Deserialize)]
struct NewsAggregatorPost {
    post_id: i32,
    parent_id: i32,
    content: String,
    created_at: i32,
}

async fn create_post(
    State(pool): State<SqlitePool>,
    Json(payload): Json<NewsAggregatorPost>,
) -> impl IntoResponse {
    if let Err(_) = query("insert into post (post_id, content, created_at) values (?, ?, ?)")
        .bind(&payload.post_id)
        .bind(payload.content)
        .bind(payload.created_at)
        .execute(&pool)
        .await
    {
        return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(axum::http::StatusCode::OK)
}

async fn send_vote_event(
    State(pool): State<SqlitePool>,
    Json(payload): Json<VoteEvent>,
) -> Result<impl IntoResponse, axum::http::StatusCode> {
    if let Err(_) = query("insert into vote_event (vote_event_id, post_id, vote, vote_event_time) values (?, ?, ?, ?)")
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

async fn news_aggregator_ranking() {}

