use askama::Template;
use crate::handlers::calendar::CalendarDay;
use crate::handlers::plan::WorkoutPlan;
use crate::handlers::session::WorkoutExerciseState;
use crate::models::{exercise::Exercise, workout_session::WorkoutSession, session_exercise::SessionExerciseDetail, exercise_log::ExerciseLogDetail, personal_record::{PersonalRecord, PersonalRecordDetail}};

// Auth templates
#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub error: Option<String>,
    pub message: Option<String>,
}

#[derive(Template)]
#[template(path = "register.html")]
pub struct RegisterTemplate {
    pub error: Option<String>,
}

// Dashboard
#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub username: String,
    pub active_page: String,
    pub recent_logs: Vec<ExerciseLogDetail>,
    pub upcoming_sessions: Vec<WorkoutSession>,
    pub exercise_count: i64,
    pub session_count: i64,
    pub log_count: i64,
    pub today_session: Option<WorkoutSession>,
    pub today_exercises: Vec<SessionExerciseDetail>,
    pub recent_prs: Vec<PersonalRecordDetail>,
}

// Exercise templates
#[derive(Template)]
#[template(path = "exercises/list.html")]
pub struct ExerciseListTemplate {
    pub username: String,
    pub active_page: String,
    pub exercises: Vec<Exercise>,
    pub message: Option<String>,
}

#[derive(Template)]
#[template(path = "exercises/detail.html")]
pub struct ExerciseDetailTemplate {
    pub username: String,
    pub active_page: String,
    pub exercise: Exercise,
    pub instructions_lines: Vec<String>,
    pub tips_lines: Vec<String>,
    pub logs: Vec<ExerciseLogDetail>,
    pub prs: Vec<PersonalRecord>,
    pub progress_json: String,
}

#[derive(Template)]
#[template(path = "exercises/form.html")]
pub struct ExerciseFormTemplate {
    pub username: String,
    pub active_page: String,
    pub exercise: Option<Exercise>,
    pub error: Option<String>,
}

// Session templates
pub struct SessionWithPreview {
    pub session: WorkoutSession,
    pub exercise_preview: String,
}

#[derive(Template)]
#[template(path = "sessions/list.html")]
pub struct SessionListTemplate {
    pub username: String,
    pub active_page: String,
    pub sessions: Vec<SessionWithPreview>,
    pub message: Option<String>,
}

#[derive(Template)]
#[template(path = "sessions/detail.html")]
pub struct SessionDetailTemplate {
    pub username: String,
    pub active_page: String,
    pub session: WorkoutSession,
    pub exercises: Vec<SessionExerciseDetail>,
    pub duration_minutes: Option<i64>,
    pub total_sets: i64,
    pub total_volume: f64,
}

#[derive(Template)]
#[template(path = "sessions/form.html")]
pub struct SessionFormTemplate {
    pub username: String,
    pub active_page: String,
    pub session: Option<WorkoutSession>,
    pub available_exercises: Vec<Exercise>,
    pub session_exercises: Vec<SessionExerciseDetail>,
    pub error: Option<String>,
}

#[derive(Template)]
#[template(path = "sessions/workout.html")]
#[allow(dead_code)]
pub struct ActiveWorkoutTemplate {
    pub username: String,
    pub active_page: String,
    pub session: WorkoutSession,
    pub exercises: Vec<WorkoutExerciseState>,
    pub just_logged: bool,
    pub rest_seconds: i64,
    pub pr_type: Option<String>,
}

#[derive(Template)]
#[template(path = "sessions/summary.html")]
pub struct WorkoutSummaryTemplate {
    pub username: String,
    pub active_page: String,
    pub session: WorkoutSession,
    pub total_sets: i64,
    pub total_volume: f64,
    pub duration_minutes: Option<i64>,
}

// Plan templates
#[derive(Template)]
#[template(path = "plans/list.html")]
pub struct PlanListTemplate {
    pub username: String,
    pub active_page: String,
    pub plans: Vec<WorkoutPlan>,
}

// Log templates
#[derive(Template)]
#[template(path = "logs/list.html")]
pub struct LogListTemplate {
    pub username: String,
    pub active_page: String,
    pub logs: Vec<ExerciseLogDetail>,
    pub message: Option<String>,
}

#[derive(Template)]
#[template(path = "logs/detail.html")]
pub struct LogDetailTemplate {
    pub username: String,
    pub active_page: String,
    pub log: ExerciseLogDetail,
}

#[derive(Template)]
#[template(path = "logs/form.html")]
pub struct LogFormTemplate {
    pub username: String,
    pub active_page: String,
    pub exercises: Vec<Exercise>,
    pub sessions: Vec<WorkoutSession>,
    pub error: Option<String>,
}

// Records templates
pub struct RecordEntry {
    pub record_type: String,
    pub value: String,
    pub achieved_at: String,
}

#[derive(Template)]
#[template(path = "records/list.html")]
pub struct RecordsListTemplate {
    pub username: String,
    pub active_page: String,
    pub grouped_records: Vec<(String, String, Vec<RecordEntry>)>,
}

// Calendar template
#[derive(Template)]
#[template(path = "calendar.html")]
#[allow(dead_code)]
pub struct CalendarTemplate {
    pub username: String,
    pub active_page: String,
    pub year: i32,
    pub month: u32,
    pub month_name: String,
    pub calendar_days: Vec<Option<CalendarDay>>,
    pub prev_month: String,
    pub next_month: String,
    pub workout_count: usize,
}

// Export template
#[derive(Template)]
#[template(path = "export.html")]
pub struct ExportTemplate {
    pub username: String,
    pub active_page: String,
}
