use axum::extract::State;
use axum::response::Html;

use crate::error::AppError;
use crate::handlers::AppState;
use crate::middleware::auth::AuthUser;
use crate::models::personal_record::PersonalRecord;
use crate::templates;

pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Html<String>, AppError> {
    let records = PersonalRecord::find_all_by_user(&state.pool, auth.user_id).await?;

    // Group records by exercise name
    let mut grouped: Vec<(String, String, Vec<templates::RecordEntry>)> = Vec::new();
    let mut current_exercise = String::new();

    for r in &records {
        if r.exercise_name != current_exercise {
            current_exercise = r.exercise_name.clone();
            grouped.push((r.exercise_name.clone(), r.muscle_group.clone(), Vec::new()));
        }
        if let Some(group) = grouped.last_mut() {
            group.2.push(templates::RecordEntry {
                record_type: format_record_type(&r.record_type),
                value: format_record_value(&r.record_type, r.value),
                achieved_at: r.achieved_at.clone(),
            });
        }
    }

    let tmpl = templates::RecordsListTemplate {
        username: auth.username,
        active_page: "records".to_string(),
        grouped_records: grouped,
    };
    Ok(Html(tmpl.to_string()))
}

fn format_record_type(t: &str) -> String {
    match t {
        "max_weight" => "Max Weight".to_string(),
        "max_reps" => "Max Reps".to_string(),
        "max_volume" => "Max Volume".to_string(),
        "estimated_1rm" => "Est. 1RM".to_string(),
        _ => t.to_string(),
    }
}

fn format_record_value(record_type: &str, value: f64) -> String {
    match record_type {
        "max_weight" | "estimated_1rm" => format!("{:.1} kg", value),
        "max_reps" => format!("{}", value as i64),
        "max_volume" => format!("{:.0} kg", value),
        _ => format!("{:.1}", value),
    }
}
