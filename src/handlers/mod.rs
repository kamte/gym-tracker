pub mod auth;
pub mod calendar;
pub mod dashboard;
pub mod exercise;
pub mod export;
pub mod log;
pub mod plan;
pub mod records;
pub mod session;

use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub jwt_secret: String,
    pub cookie_secure: bool,
}
