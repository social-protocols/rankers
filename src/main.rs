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
    vote: i32,
}

async fn send_vote_event(
    State(pool): State<SqlitePool>,
    Json(payload): Json<VoteEvent>,
) -> Result<impl IntoResponse, axum::http::StatusCode> {
    if let Err(_) = query("insert into vote_event (vote_event_id, vote) values (?, ?)")
        .bind(&payload.vote_event_id)
        .bind(payload.vote)
        .execute(&pool)
        .await
    {
        return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(axum::http::StatusCode::OK)
}
