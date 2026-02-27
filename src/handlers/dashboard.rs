use axum::extract::State;
use axum::response::Html;

use crate::error::AppError;
use crate::handlers::AppState;
use crate::middleware::auth::AuthUser;
use crate::models::{exercise::Exercise, exercise_log::ExerciseLog, workout_session::WorkoutSession, session_exercise::SessionExercise, personal_record::PersonalRecord};
use crate::templates;

pub async fn index(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Html<String>, AppError> {
    let recent_logs = ExerciseLog::find_recent_by_user(&state.pool, auth.user_id, 10).await?;
    let upcoming_sessions = WorkoutSession::find_upcoming_by_user(&state.pool, auth.user_id, 5).await?;
    let exercise_count = Exercise::count_by_user(&state.pool, auth.user_id).await?;
    let session_count = WorkoutSession::count_by_user(&state.pool, auth.user_id).await?;
    let log_count = ExerciseLog::count_by_user(&state.pool, auth.user_id).await?;
    let recent_prs = PersonalRecord::find_recent_by_user(&state.pool, auth.user_id, 5).await?;

    let today_session = WorkoutSession::find_today_by_user(&state.pool, auth.user_id).await?;
    let today_exercises = if let Some(ref session) = today_session {
        SessionExercise::find_by_session(&state.pool, session.id, auth.user_id).await?
    } else {
        vec![]
    };

    let tmpl = templates::DashboardTemplate {
        username: auth.username,
        active_page: "dashboard".to_string(),
        recent_logs,
        upcoming_sessions,
        exercise_count,
        session_count,
        log_count,
        today_session,
        today_exercises,
        recent_prs,
    };

    Ok(Html(tmpl.to_string()))
}
