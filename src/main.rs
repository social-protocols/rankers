use axum::{
    routing::get,
    response::{Json, IntoResponse},
    Router,
    extract::Extension,
};
use serde_json::{Value, json};
use dotenv::dotenv;
use std::env;
use sqlx::sqlite::SqlitePool;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to create pool.");

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/halo", get(halo))
        .route("/json", get(json))
        .route("/from_db", get(from_db))
        .layer(Extension(pool));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn halo() -> &'static str {
    "Halo i bims widder lol!"
}

async fn json() -> Json<Value> {
    Json(json!({ "data": 42 }))
}

async fn from_db(Extension(pool): Extension<SqlitePool>) -> impl IntoResponse {
    let row: (i32,) =
        sqlx::query_as("SELECT 42")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch row");

    format!("Result: {}", row.0)
}
