// --------------------------------------------------------------------
// INLINED FROM: https://github.com/social-protocols/prototype-1/blob/main/src/error.rs
// --------------------------------------------------------------------

use axum::http::StatusCode;
use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

// https://github.com/tokio-rs/axum/blob/main/examples/anyhow-error-response/src/main.rs
pub struct AppError(pub anyhow::Error);

// Tell axum how to convert `AppError` into a response.
// https://github.com/tokio-rs/axum/discussions/713
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError(inner) => {
                tracing::debug!("stacktrace: {}", inner.backtrace());
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        AppError(err.into())
    }
}
