use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found")]
    NotFound,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Unauthorized => Redirect::to("/login").into_response(),
            AppError::NotFound => {
                (StatusCode::NOT_FOUND, Html("<h1>404 - Not Found</h1><p>The page you're looking for doesn't exist.</p><a href=\"/\">Go home</a>")).into_response()
            }
            AppError::BadRequest(msg) => {
                // Simple HTML-escape for error messages
                let escaped = msg.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;");
                (StatusCode::BAD_REQUEST, Html(format!("<h1>400 - Bad Request</h1><p>{escaped}</p><a href=\"javascript:history.back()\">Go back</a>"))).into_response()
            }
            AppError::Database(e) => {
                tracing::error!("Database error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, Html("<h1>500 - Internal Server Error</h1><p>Something went wrong.</p><a href=\"/\">Go home</a>")).into_response()
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {msg}");
                (StatusCode::INTERNAL_SERVER_ERROR, Html("<h1>500 - Internal Server Error</h1><p>Something went wrong.</p><a href=\"/\">Go home</a>")).into_response()
            }
        }
    }
}
