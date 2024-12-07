use crate::api;
use crate::common::error::AppError;
use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;
use tower_http::trace::TraceLayer;

pub async fn start_http_server(pool: SqlitePool) -> Result<(), AppError> {
    let app = Router::new()
        .route("/health_check", get(api::health_check))
        .route("/items", post(api::register_item))
        .route("/vote_events", post(api::register_vote_event))
        .route("/rankings/hn", get(api::get_hacker_news_ranking))
        .route("/rankings/qn", get(api::get_ranking_quality_news))
        .route("/rankings/newest", get(api::get_ranking_newest))
        .layer(TraceLayer::new_for_http())
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
