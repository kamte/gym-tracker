use axum::extract::State;
use axum::response::{Html, IntoResponse, Response};
use axum::http::{header, StatusCode};

use crate::error::AppError;
use crate::handlers::AppState;
use crate::middleware::auth::AuthUser;
use crate::models::exercise_log::ExerciseLog;
use crate::models::workout_session::WorkoutSession;
use crate::templates;

pub async fn index(auth: AuthUser) -> Result<Html<String>, AppError> {
    let tmpl = templates::ExportTemplate {
        username: auth.username,
        active_page: "export".to_string(),
    };
    Ok(Html(tmpl.to_string()))
}

pub async fn export_logs_csv(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Response, AppError> {
    let logs = ExerciseLog::find_for_csv_export(&state.pool, auth.user_id).await?;

    let mut csv = String::from("Date,Exercise,Muscle Group,Session,Set,Weight (kg),Reps,RPE,Set Type,Notes\n");
    for log in &logs {
        let session = log.session_name.as_deref().unwrap_or("");
        let rpe = log.rpe.map(|r| format!("{}", r)).unwrap_or_default();
        // Escape CSV fields that might contain commas or quotes
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{}\n",
            csv_escape(&log.performed_at),
            csv_escape(&log.exercise_name),
            csv_escape(&log.muscle_group),
            csv_escape(session),
            log.set_number,
            log.weight_kg,
            log.reps_completed,
            rpe,
            csv_escape(&log.set_type),
            csv_escape(&log.notes),
        ));
    }

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"gym_logs.csv\""),
        ],
        csv,
    ).into_response())
}

pub async fn export_sessions_csv(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Response, AppError> {
    let sessions = WorkoutSession::find_all_by_user(&state.pool, auth.user_id).await?;

    let mut csv = String::from("Name,Scheduled At,Status,Started At,Completed At,Notes\n");
    for s in &sessions {
        csv.push_str(&format!(
            "{},{},{},{},{},{}\n",
            csv_escape(&s.name),
            csv_escape(&s.scheduled_at),
            csv_escape(&s.status),
            csv_escape(s.started_at.as_deref().unwrap_or("")),
            csv_escape(s.completed_at.as_deref().unwrap_or("")),
            csv_escape(&s.notes),
        ));
    }

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"gym_sessions.csv\""),
        ],
        csv,
    ).into_response())
}

fn csv_escape(s: &str) -> String {
    // Neutralize Excel formula injection by prefixing with a tab
    let s = if s.starts_with('=') || s.starts_with('+') || s.starts_with('-') || s.starts_with('@') {
        format!("\t{}", s)
    } else {
        s.to_string()
    };
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s
    }
}
