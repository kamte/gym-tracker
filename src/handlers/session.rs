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
    session_exercise::SessionExercise,
    workout_session::WorkoutSession,
};
use crate::templates;

#[derive(Deserialize)]
pub struct SessionForm {
    pub name: String,
    pub scheduled_at: String,
    pub notes: String,
    pub status: String,
    #[serde(default)]
    pub exercise_id: Vec<String>,
    #[serde(default)]
    pub planned_sets: Vec<String>,
    #[serde(default)]
    pub planned_reps: Vec<String>,
    #[serde(default)]
    pub planned_weight_kg: Vec<String>,
}

pub struct WorkoutExerciseState {
    pub exercise_id: i64,
    pub exercise_name: String,
    pub muscle_group: String,
    pub planned_sets: i64,
    pub planned_reps: i64,
    pub planned_weight_kg: f64,
    pub suggested_weight: f64,
    pub completed_sets: Vec<CompletedSet>,
}

pub struct CompletedSet {
    pub set_number: i64,
    pub reps_completed: i64,
    pub weight_kg: f64,
    pub set_type: String,
}

#[derive(Deserialize)]
pub struct WorkoutLogForm {
    pub exercise_id: i64,
    pub set_number: i64,
    pub reps_completed: i64,
    pub weight_kg: f64,
    #[serde(default = "default_set_type")]
    pub set_type: String,
    #[serde(default)]
    pub rest: Option<i64>,
}

fn default_set_type() -> String {
    "working".to_string()
}

pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, AppError> {
    let sessions = WorkoutSession::find_all_by_user(&state.pool, auth.user_id).await?;
    let message = params.get("message").cloned();

    let exercise_names_map = SessionExercise::find_exercise_names_for_user(&state.pool, auth.user_id).await?;

    let sessions_with_preview: Vec<templates::SessionWithPreview> = sessions
        .into_iter()
        .map(|session| {
            let names = exercise_names_map.get(&session.id).cloned().unwrap_or_default();
            let exercise_preview = if names.len() <= 3 {
                names.join(", ")
            } else {
                format!("{} +{} more", names[..3].join(", "), names.len() - 3)
            };
            templates::SessionWithPreview {
                session,
                exercise_preview,
            }
        })
        .collect();

    let tmpl = templates::SessionListTemplate {
        username: auth.username,
        active_page: "sessions".to_string(),
        sessions: sessions_with_preview,
        message,
    };
    Ok(Html(tmpl.to_string()))
}

pub async fn detail(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
) -> Result<Html<String>, AppError> {
    let session = WorkoutSession::find_by_id(&state.pool, id, auth.user_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let exercises = SessionExercise::find_by_session(&state.pool, id, auth.user_id).await?;

    let duration_minutes = compute_duration_minutes(&session);

    let total_sets = ExerciseLog::count_by_session(&state.pool, id, auth.user_id).await.unwrap_or(0);
    let total_volume = ExerciseLog::total_volume_by_session(&state.pool, id, auth.user_id).await.unwrap_or(0.0);

    let tmpl = templates::SessionDetailTemplate {
        username: auth.username,
        active_page: "sessions".to_string(),
        session,
        exercises,
        duration_minutes,
        total_sets,
        total_volume,
    };
    Ok(Html(tmpl.to_string()))
}

pub async fn new_form(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Html<String>, AppError> {
    let available_exercises = Exercise::find_all_by_user(&state.pool, auth.user_id).await?;

    let tmpl = templates::SessionFormTemplate {
        username: auth.username,
        active_page: "sessions".to_string(),
        session: None,
        available_exercises,
        session_exercises: vec![],
        error: None,
    };
    Ok(Html(tmpl.to_string()))
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::Form(form): axum::Form<SessionForm>,
) -> Result<axum::response::Response, AppError> {
    if form.name.is_empty() || form.name.len() > 100 {
        let available_exercises = Exercise::find_all_by_user(&state.pool, auth.user_id).await?;
        let tmpl = templates::SessionFormTemplate {
            username: auth.username,
            active_page: "sessions".to_string(),
            session: None,
            available_exercises,
            session_exercises: vec![],
            error: Some("Session name must be between 1 and 100 characters".to_string()),
        };
        return Ok(Html(tmpl.to_string()).into_response());
    }

    if form.scheduled_at.is_empty() {
        let available_exercises = Exercise::find_all_by_user(&state.pool, auth.user_id).await?;
        let tmpl = templates::SessionFormTemplate {
            username: auth.username,
            active_page: "sessions".to_string(),
            session: None,
            available_exercises,
            session_exercises: vec![],
            error: Some("Scheduled date is required".to_string()),
        };
        return Ok(Html(tmpl.to_string()).into_response());
    }

    let status = if ["planned", "completed", "cancelled"].contains(&form.status.as_str()) {
        &form.status
    } else {
        "planned"
    };

    let session_id = WorkoutSession::create(
        &state.pool,
        auth.user_id,
        &form.name,
        &form.scheduled_at,
        &form.notes,
        status,
    )
    .await?;

    // Collect valid exercise IDs owned by this user
    let user_exercises = Exercise::find_all_by_user(&state.pool, auth.user_id).await?;
    let valid_exercise_ids: std::collections::HashSet<i64> = user_exercises.iter().map(|e| e.id).collect();

    for (i, eid_str) in form.exercise_id.iter().enumerate() {
        if let Ok(eid) = eid_str.parse::<i64>() {
            if eid > 0 && valid_exercise_ids.contains(&eid) {
                let sets = form.planned_sets.get(i).and_then(|s| s.parse().ok()).unwrap_or(3);
                let reps = form.planned_reps.get(i).and_then(|s| s.parse().ok()).unwrap_or(10);
                let weight = form.planned_weight_kg.get(i).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                SessionExercise::create(&state.pool, session_id, eid, sets, reps, weight, i as i64).await?;
            }
        }
    }

    Ok(Redirect::to("/sessions?message=Session+created+successfully").into_response())
}

pub async fn edit_form(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
) -> Result<Html<String>, AppError> {
    let session = WorkoutSession::find_by_id(&state.pool, id, auth.user_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let available_exercises = Exercise::find_all_by_user(&state.pool, auth.user_id).await?;
    let session_exercises = SessionExercise::find_by_session(&state.pool, id, auth.user_id).await?;

    let tmpl = templates::SessionFormTemplate {
        username: auth.username,
        active_page: "sessions".to_string(),
        session: Some(session),
        available_exercises,
        session_exercises,
        error: None,
    };
    Ok(Html(tmpl.to_string()))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
    axum::Form(form): axum::Form<SessionForm>,
) -> Result<axum::response::Response, AppError> {
    if form.name.is_empty() || form.name.len() > 100 {
        return Err(AppError::BadRequest("Session name must be between 1 and 100 characters".to_string()));
    }

    let status = if ["planned", "completed", "cancelled"].contains(&form.status.as_str()) {
        &form.status
    } else {
        "planned"
    };

    let updated = WorkoutSession::update(
        &state.pool, id, auth.user_id, &form.name, &form.scheduled_at, &form.notes, status,
    )
    .await?;

    if !updated {
        return Err(AppError::NotFound);
    }

    // Collect valid exercise IDs owned by this user
    let user_exercises = Exercise::find_all_by_user(&state.pool, auth.user_id).await?;
    let valid_exercise_ids: std::collections::HashSet<i64> = user_exercises.iter().map(|e| e.id).collect();

    SessionExercise::delete_by_session(&state.pool, id, auth.user_id).await?;
    for (i, eid_str) in form.exercise_id.iter().enumerate() {
        if let Ok(eid) = eid_str.parse::<i64>() {
            if eid > 0 && valid_exercise_ids.contains(&eid) {
                let sets = form.planned_sets.get(i).and_then(|s| s.parse().ok()).unwrap_or(3);
                let reps = form.planned_reps.get(i).and_then(|s| s.parse().ok()).unwrap_or(10);
                let weight = form.planned_weight_kg.get(i).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                SessionExercise::create(&state.pool, id, eid, sets, reps, weight, i as i64).await?;
            }
        }
    }

    Ok(Redirect::to("/sessions?message=Session+updated+successfully").into_response())
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
) -> Result<Redirect, AppError> {
    WorkoutSession::delete(&state.pool, id, auth.user_id).await?;
    Ok(Redirect::to("/sessions?message=Session+deleted+successfully"))
}

pub async fn active_workout(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, AppError> {
    // Verify session exists and belongs to user
    WorkoutSession::find_by_id(&state.pool, id, auth.user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // Auto-start workout timer on first visit
    WorkoutSession::start_workout(&state.pool, id, auth.user_id).await?;

    let session_exercises = SessionExercise::find_by_session(&state.pool, id, auth.user_id).await?;
    let existing_logs = ExerciseLog::find_by_session(&state.pool, id, auth.user_id).await?;

    let mut exercises: Vec<WorkoutExerciseState> = Vec::new();

    for se in &session_exercises {
        let completed: Vec<CompletedSet> = existing_logs
            .iter()
            .filter(|log| log.exercise_id == se.exercise_id)
            .map(|log| CompletedSet {
                set_number: log.set_number,
                reps_completed: log.reps_completed,
                weight_kg: log.weight_kg,
                set_type: log.set_type.clone(),
            })
            .collect();

        let suggested_weight = if se.planned_weight_kg > 0.0 {
            se.planned_weight_kg
        } else {
            ExerciseLog::find_last_weight_by_exercise_and_user(&state.pool, se.exercise_id, auth.user_id)
                .await?
                .unwrap_or(0.0)
        };

        exercises.push(WorkoutExerciseState {
            exercise_id: se.exercise_id,
            exercise_name: se.exercise_name.clone(),
            muscle_group: se.muscle_group.clone(),
            planned_sets: se.planned_sets,
            planned_reps: se.planned_reps,
            planned_weight_kg: se.planned_weight_kg,
            suggested_weight,
            completed_sets: completed,
        });
    }

    // Reload session to get started_at
    let session = WorkoutSession::find_by_id(&state.pool, id, auth.user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let just_logged = params.get("just_logged").map(|v| v == "1").unwrap_or(false);
    let rest_seconds = params.get("rest").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
    let pr_type = params.get("pr").cloned();

    let tmpl = templates::ActiveWorkoutTemplate {
        username: auth.username,
        active_page: "sessions".to_string(),
        session,
        exercises,
        just_logged,
        rest_seconds,
        pr_type,
    };
    Ok(Html(tmpl.to_string()))
}

pub async fn log_workout_set(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
    axum::Form(form): axum::Form<WorkoutLogForm>,
) -> Result<Redirect, AppError> {
    let _session = WorkoutSession::find_by_id(&state.pool, id, auth.user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // Verify the exercise belongs to the authenticated user
    Exercise::find_by_id(&state.pool, form.exercise_id, auth.user_id)
        .await?
        .ok_or(AppError::BadRequest("Invalid exercise".to_string()))?;

    let log_id = ExerciseLog::create(
        &state.pool,
        auth.user_id,
        form.exercise_id,
        Some(id),
        form.set_number,
        form.reps_completed,
        form.weight_kg,
        None,
        &form.set_type,
        "",
    )
    .await?;

    // Check for PRs
    let broken_prs = PersonalRecord::check_and_update(
        &state.pool, auth.user_id, form.exercise_id,
        log_id, form.weight_kg, form.reps_completed, &form.set_type,
    ).await.unwrap_or_default();

    let rest = form.rest.unwrap_or(90);
    let mut url = format!("/sessions/{}/workout?just_logged=1&rest={}", id, rest);
    if let Some(pr) = broken_prs.first() {
        url.push_str(&format!("&pr={}", pr));
    }

    Ok(Redirect::to(&url))
}

pub async fn start_session(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
) -> Result<Redirect, AppError> {
    WorkoutSession::start_workout(&state.pool, id, auth.user_id).await?;
    Ok(Redirect::to(&format!("/sessions/{}/workout", id)))
}

pub async fn complete_session(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
) -> Result<Redirect, AppError> {
    // Verify session belongs to user
    WorkoutSession::find_by_id(&state.pool, id, auth.user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    WorkoutSession::complete_workout(&state.pool, id, auth.user_id).await?;

    Ok(Redirect::to(&format!("/sessions/{}/summary", id)))
}

pub async fn workout_summary(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
) -> Result<Html<String>, AppError> {
    let session = WorkoutSession::find_by_id(&state.pool, id, auth.user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let total_sets = ExerciseLog::count_by_session(&state.pool, id, auth.user_id).await?;
    let total_volume = ExerciseLog::total_volume_by_session(&state.pool, id, auth.user_id).await?;
    let duration_minutes = compute_duration_minutes(&session);

    let tmpl = templates::WorkoutSummaryTemplate {
        username: auth.username,
        active_page: "sessions".to_string(),
        session,
        total_sets,
        total_volume,
        duration_minutes,
    };
    Ok(Html(tmpl.to_string()))
}

fn compute_duration_minutes(session: &WorkoutSession) -> Option<i64> {
    if let (Some(started), Some(completed)) = (&session.started_at, &session.completed_at) {
        if let (Ok(s), Ok(c)) = (
            chrono::NaiveDateTime::parse_from_str(started, "%Y-%m-%d %H:%M:%S"),
            chrono::NaiveDateTime::parse_from_str(completed, "%Y-%m-%d %H:%M:%S"),
        ) {
            let duration = c.signed_duration_since(s);
            return Some(duration.num_minutes());
        }
    }
    None
}
