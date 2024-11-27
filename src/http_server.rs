use crate::api;
use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;

pub async fn start_http_server(pool: SqlitePool) -> Result<()> {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/create_post", post(api::create_post))
        .route("/send_vote_event", post(api::send_vote_event))
        .route("/rankings/hn", get(api::get_hacker_news_ranking))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
