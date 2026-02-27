mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod models;
mod templates;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{routing::get, Router};
use axum::http::HeaderValue;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "gym=debug,tower_http=debug".parse().unwrap()),
        )
        .init();

    dotenvy::dotenv().ok();

    let config = config::Config::from_env();
    let pool = db::create_pool(&config.database_url).await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    tracing::info!("Migrations applied successfully");

    let app_state = handlers::AppState {
        pool,
        jwt_secret: config.jwt_secret,
        cookie_secure: config.cookie_secure,
    };

    // Rate-limited auth routes
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(2)
        .burst_size(5)
        .finish()
        .unwrap();

    let governor_conf = Arc::new(governor_conf);

    let auth_routes = Router::new()
        .route("/register", get(handlers::auth::register_form).post(handlers::auth::register))
        .route("/login", get(handlers::auth::login_form).post(handlers::auth::login))
        .layer(GovernorLayer::new(governor_conf));

    let app = Router::new()
        .merge(auth_routes)
        .route("/logout", axum::routing::post(handlers::auth::logout))
        // Dashboard
        .route("/", get(handlers::dashboard::index))
        // Exercises
        .route("/exercises", get(handlers::exercise::list))
        .route("/exercises/new", get(handlers::exercise::new_form).post(handlers::exercise::create))
        .route("/exercises/seed", axum::routing::post(handlers::exercise::seed))
        .route("/exercises/{id}", get(handlers::exercise::detail).post(handlers::exercise::delete))
        .route("/exercises/{id}/edit", get(handlers::exercise::edit_form).post(handlers::exercise::update))
        // Plans
        .route("/plans", get(handlers::plan::list))
        .route("/plans/{id}/use", axum::routing::post(handlers::plan::use_plan))
        // Sessions
        .route("/sessions", get(handlers::session::list))
        .route("/sessions/new", get(handlers::session::new_form).post(handlers::session::create))
        .route("/sessions/{id}", get(handlers::session::detail).post(handlers::session::delete))
        .route("/sessions/{id}/edit", get(handlers::session::edit_form).post(handlers::session::update))
        .route("/sessions/{id}/workout", get(handlers::session::active_workout))
        .route("/sessions/{id}/workout/log", axum::routing::post(handlers::session::log_workout_set))
        .route("/sessions/{id}/start", axum::routing::post(handlers::session::start_session))
        .route("/sessions/{id}/complete", axum::routing::post(handlers::session::complete_session))
        .route("/sessions/{id}/summary", get(handlers::session::workout_summary))
        // Logs
        .route("/logs", get(handlers::log::list))
        .route("/logs/new", get(handlers::log::new_form).post(handlers::log::create))
        .route("/logs/{id}", get(handlers::log::detail).post(handlers::log::delete))
        // Personal Records
        .route("/records", get(handlers::records::list))
        // Calendar
        .route("/calendar", get(handlers::calendar::index))
        // Export
        .route("/export", get(handlers::export::index))
        .route("/export/logs.csv", get(handlers::export::export_logs_csv))
        .route("/export/sessions.csv", get(handlers::export::export_sessions_csv))
        // Static files
        .nest_service("/static", ServeDir::new("static"))
        .with_state(app_state)
        // Security headers
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ));

    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("Server listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
