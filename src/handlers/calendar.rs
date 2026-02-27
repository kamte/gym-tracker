use axum::{
    extract::{Query, State},
    response::Html,
};
use serde::Deserialize;

use crate::error::AppError;
use crate::handlers::AppState;
use crate::middleware::auth::AuthUser;
use crate::models::workout_session::WorkoutSession;
use crate::templates;

#[derive(Deserialize)]
pub struct CalendarQuery {
    pub month: Option<String>,
}

pub struct CalendarDay {
    pub day: u32,
    pub has_workout: bool,
    pub is_today: bool,
    pub session_names: Vec<String>,
}

pub async fn index(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<CalendarQuery>,
) -> Result<Html<String>, AppError> {
    let now = chrono::Local::now().naive_local();
    let (year, month) = if let Some(ref m) = query.month {
        parse_year_month(m).unwrap_or((now.year(), now.month()))
    } else {
        (now.year(), now.month())
    };

    let sessions = WorkoutSession::find_completed_in_month(&state.pool, auth.user_id, year, month).await?;

    // Build workout days set
    let mut workout_days: std::collections::HashMap<u32, Vec<String>> = std::collections::HashMap::new();
    for s in &sessions {
        let date_str = s.completed_at.as_deref().unwrap_or(&s.scheduled_at);
        if let Some(day) = extract_day(date_str) {
            workout_days.entry(day).or_default().push(s.name.clone());
        }
    }

    // Calculate calendar grid
    let first_day_of_month = chrono::NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or(AppError::BadRequest("Invalid date".to_string()))?;
    let days_in_month = days_in_month(year, month);
    let start_weekday = first_day_of_month.weekday().num_days_from_monday(); // 0=Mon, 6=Sun

    let today = now.date();
    let today_day = if today.year() == year && today.month() == month {
        Some(today.day())
    } else {
        None
    };

    let mut calendar_days: Vec<Option<CalendarDay>> = Vec::new();
    // Empty cells before first day
    for _ in 0..start_weekday {
        calendar_days.push(None);
    }
    for d in 1..=days_in_month {
        let names = workout_days.get(&d).cloned().unwrap_or_default();
        calendar_days.push(Some(CalendarDay {
            day: d,
            has_workout: !names.is_empty(),
            is_today: today_day == Some(d),
            session_names: names,
        }));
    }

    // Prev/next month
    let (prev_year, prev_month) = if month == 1 { (year - 1, 12u32) } else { (year, month - 1) };
    let (next_year, next_month) = if month == 12 { (year + 1, 1u32) } else { (year, month + 1) };

    let month_name = month_name(month);
    let weekly_count = sessions.len();

    let tmpl = templates::CalendarTemplate {
        username: auth.username,
        active_page: "calendar".to_string(),
        year,
        month,
        month_name: month_name.to_string(),
        calendar_days,
        prev_month: format!("{:04}-{:02}", prev_year, prev_month),
        next_month: format!("{:04}-{:02}", next_year, next_month),
        workout_count: weekly_count,
    };
    Ok(Html(tmpl.to_string()))
}

fn parse_year_month(s: &str) -> Option<(i32, u32)> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() == 2 {
        let y = parts[0].parse().ok()?;
        let m = parts[1].parse().ok()?;
        if (1..=12).contains(&m) {
            return Some((y, m));
        }
    }
    None
}

fn extract_day(date_str: &str) -> Option<u32> {
    // Parse "YYYY-MM-DD" or "YYYY-MM-DD HH:MM:SS"
    let date_part = date_str.split(' ').next()?;
    let parts: Vec<&str> = date_part.split('-').collect();
    if parts.len() == 3 {
        return parts[2].parse().ok();
    }
    None
}

fn days_in_month(year: i32, month: u32) -> u32 {
    if month == 12 {
        chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .unwrap()
    .pred_opt()
    .unwrap()
    .day()
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January", 2 => "February", 3 => "March", 4 => "April",
        5 => "May", 6 => "June", 7 => "July", 8 => "August",
        9 => "September", 10 => "October", 11 => "November", 12 => "December",
        _ => "Unknown",
    }
}

use chrono::Datelike;
