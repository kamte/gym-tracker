use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse, Redirect},
};
use serde::Deserialize;
use std::collections::HashMap;

use crate::error::AppError;
use crate::handlers::AppState;
use crate::middleware::auth::AuthUser;
use crate::models::{
    exercise::Exercise,
    exercise_log::ExerciseLog,
    personal_record::PersonalRecord,
    workout_session::WorkoutSession,
};
use crate::templates;

#[derive(Deserialize)]
pub struct LogForm {
    pub exercise_id: i64,
    pub workout_session_id: Option<String>,
    pub set_number: i64,
    pub reps_completed: i64,
    pub weight_kg: f64,
    pub rpe: Option<String>,
    #[serde(default = "default_set_type")]
    pub set_type: String,
    #[serde(default)]
    pub notes: String,
}

fn default_set_type() -> String {
    "working".to_string()
}

pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, AppError> {
    let logs = ExerciseLog::find_all_by_user(&state.pool, auth.user_id).await?;
    let message = params.get("message").cloned();

    let tmpl = templates::LogListTemplate {
        username: auth.username,
        active_page: "logs".to_string(),
        logs,
        message,
    };
    Ok(Html(tmpl.to_string()))
}

pub async fn detail(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
) -> Result<Html<String>, AppError> {
    let log = ExerciseLog::find_by_id(&state.pool, id, auth.user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let tmpl = templates::LogDetailTemplate {
        username: auth.username,
        active_page: "logs".to_string(),
        log,
    };
    Ok(Html(tmpl.to_string()))
}

pub async fn new_form(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Html<String>, AppError> {
    let exercises = Exercise::find_all_by_user(&state.pool, auth.user_id).await?;
    let sessions = WorkoutSession::find_all_by_user(&state.pool, auth.user_id).await?;

    let tmpl = templates::LogFormTemplate {
        username: auth.username,
        active_page: "logs".to_string(),
        exercises,
        sessions,
        error: None,
    };
    Ok(Html(tmpl.to_string()))
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::Form(form): axum::Form<LogForm>,
) -> Result<axum::response::Response, AppError> {
    if form.set_number < 1 || form.set_number > 100 {
        let exercises = Exercise::find_all_by_user(&state.pool, auth.user_id).await?;
        let sessions = WorkoutSession::find_all_by_user(&state.pool, auth.user_id).await?;
        let tmpl = templates::LogFormTemplate {
            username: auth.username,
            active_page: "logs".to_string(),
            exercises,
            sessions,
            error: Some("Set number must be between 1 and 100".to_string()),
        };
        return Ok(Html(tmpl.to_string()).into_response());
    }

    if form.reps_completed < 0 || form.reps_completed > 1000 {
        let exercises = Exercise::find_all_by_user(&state.pool, auth.user_id).await?;
        let sessions = WorkoutSession::find_all_by_user(&state.pool, auth.user_id).await?;
        let tmpl = templates::LogFormTemplate {
            username: auth.username,
            active_page: "logs".to_string(),
            exercises,
            sessions,
            error: Some("Reps must be between 0 and 1000".to_string()),
        };
        return Ok(Html(tmpl.to_string()).into_response());
    }

    let session_id = form
        .workout_session_id
        .as_ref()
        .and_then(|s| if s.is_empty() { None } else { s.parse::<i64>().ok() });

    let rpe = form
        .rpe
        .as_ref()
        .and_then(|s| if s.is_empty() { None } else { s.parse::<f64>().ok() });

    let set_type = if ["warmup", "working", "drop", "failure"].contains(&form.set_type.as_str()) {
        &form.set_type
    } else {
        "working"
    };

    // Verify the exercise belongs to the authenticated user
    Exercise::find_by_id(&state.pool, form.exercise_id, auth.user_id)
        .await?
        .ok_or(AppError::BadRequest("Invalid exercise".to_string()))?;

    let log_id = ExerciseLog::create(
        &state.pool,
        auth.user_id,
        form.exercise_id,
        session_id,
        form.set_number,
        form.reps_completed,
        form.weight_kg,
        rpe,
        set_type,
        &form.notes,
    )
    .await?;

    // Check for PRs
    let _broken_prs = PersonalRecord::check_and_update(
        &state.pool, auth.user_id, form.exercise_id,
        log_id, form.weight_kg, form.reps_completed, set_type,
    ).await.unwrap_or_default();

    Ok(Redirect::to("/logs?message=Log+entry+created+successfully").into_response())
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
) -> Result<Redirect, AppError> {
    ExerciseLog::delete(&state.pool, id, auth.user_id).await?;
    Ok(Redirect::to("/logs?message=Log+entry+deleted+successfully"))
}
