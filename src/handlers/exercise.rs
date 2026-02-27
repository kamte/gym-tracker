use axum::{
    extract::{Path, Query, State},
    response::{Html, Redirect},
};
use serde::Deserialize;
use std::collections::HashMap;

use crate::error::AppError;
use crate::handlers::AppState;
use crate::middleware::auth::AuthUser;
use crate::models::exercise::Exercise;
use crate::models::exercise_log::ExerciseLog;
use crate::models::personal_record::PersonalRecord;
use crate::templates;

#[derive(Deserialize)]
pub struct ExerciseForm {
    pub name: String,
    pub description: String,
    pub muscle_group: String,
    pub equipment: String,
    pub difficulty: String,
    pub instructions: String,
    pub tips: String,
}

pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, AppError> {
    let exercises = Exercise::find_all_by_user(&state.pool, auth.user_id).await?;
    let message = params.get("message").cloned();

    let tmpl = templates::ExerciseListTemplate {
        username: auth.username,
        active_page: "exercises".to_string(),
        exercises,
        message,
    };
    Ok(Html(tmpl.to_string()))
}

pub async fn detail(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
) -> Result<Html<String>, AppError> {
    let exercise = Exercise::find_by_id(&state.pool, id, auth.user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let logs = ExerciseLog::find_by_exercise_and_user(&state.pool, id, auth.user_id, 20).await?;
    let prs = PersonalRecord::find_by_exercise(&state.pool, id, auth.user_id).await?;
    let progress_data = ExerciseLog::find_progress_by_exercise(&state.pool, id, auth.user_id).await?;

    // Serialize progress data as JSON for chart
    let progress_json = serde_json::to_string(&progress_data.iter().map(|p| {
        serde_json::json!({
            "date": p.date,
            "max_weight": p.max_weight,
            "total_volume": p.total_volume,
            "max_estimated_1rm": p.max_estimated_1rm
        })
    }).collect::<Vec<_>>()).unwrap_or_else(|_| "[]".to_string());

    let instructions_lines: Vec<String> = exercise.instructions.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect();
    let tips_lines: Vec<String> = exercise.tips.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect();

    let tmpl = templates::ExerciseDetailTemplate {
        username: auth.username,
        active_page: "exercises".to_string(),
        exercise,
        instructions_lines,
        tips_lines,
        logs,
        prs,
        progress_json,
    };
    Ok(Html(tmpl.to_string()))
}

pub async fn new_form(auth: AuthUser) -> Result<Html<String>, AppError> {
    let tmpl = templates::ExerciseFormTemplate {
        username: auth.username,
        active_page: "exercises".to_string(),
        exercise: None,
        error: None,
    };
    Ok(Html(tmpl.to_string()))
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::Form(form): axum::Form<ExerciseForm>,
) -> Result<axum::response::Response, AppError> {
    use axum::response::IntoResponse;

    if form.name.is_empty() || form.name.len() > 100 {
        let tmpl = templates::ExerciseFormTemplate {
            username: auth.username,
            active_page: "exercises".to_string(),
            exercise: None,
            error: Some("Exercise name must be between 1 and 100 characters".to_string()),
        };
        return Ok(Html(tmpl.to_string()).into_response());
    }

    Exercise::create(
        &state.pool, auth.user_id, &form.name, &form.description, &form.muscle_group,
        &form.equipment, &form.difficulty, &form.instructions, &form.tips,
    ).await?;
    Ok(Redirect::to("/exercises?message=Exercise+created+successfully").into_response())
}

pub async fn edit_form(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
) -> Result<Html<String>, AppError> {
    let exercise = Exercise::find_by_id(&state.pool, id, auth.user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let tmpl = templates::ExerciseFormTemplate {
        username: auth.username,
        active_page: "exercises".to_string(),
        exercise: Some(exercise),
        error: None,
    };
    Ok(Html(tmpl.to_string()))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
    axum::Form(form): axum::Form<ExerciseForm>,
) -> Result<axum::response::Response, AppError> {
    use axum::response::IntoResponse;

    if form.name.is_empty() || form.name.len() > 100 {
        let exercise = Exercise::find_by_id(&state.pool, id, auth.user_id).await?.ok_or(AppError::NotFound)?;
        let tmpl = templates::ExerciseFormTemplate {
            username: auth.username,
            active_page: "exercises".to_string(),
            exercise: Some(exercise),
            error: Some("Exercise name must be between 1 and 100 characters".to_string()),
        };
        return Ok(Html(tmpl.to_string()).into_response());
    }

    let updated = Exercise::update(
        &state.pool, id, auth.user_id, &form.name, &form.description, &form.muscle_group,
        &form.equipment, &form.difficulty, &form.instructions, &form.tips,
    ).await?;
    if !updated {
        return Err(AppError::NotFound);
    }
    Ok(Redirect::to("/exercises?message=Exercise+updated+successfully").into_response())
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
) -> Result<Redirect, AppError> {
    Exercise::delete(&state.pool, id, auth.user_id).await?;
    Ok(Redirect::to("/exercises?message=Exercise+deleted+successfully"))
}

pub async fn seed(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Redirect, AppError> {
    Exercise::seed_or_update_defaults(&state.pool, auth.user_id).await?;
    Ok(Redirect::to("/exercises?message=Default+exercises+loaded+successfully"))
}
