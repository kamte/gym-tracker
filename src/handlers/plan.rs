use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Redirect},
};
use chrono::Datelike;

use crate::error::AppError;
use crate::handlers::AppState;
use crate::middleware::auth::AuthUser;
use crate::models::{
    exercise::Exercise,
    session_exercise::SessionExercise,
    workout_session::WorkoutSession,
};
use crate::templates;

pub struct PlanExercise {
    pub name: &'static str,
    pub sets: i64,
    pub reps: i64,
    pub weight_kg: f64,
}

pub struct PlanDay {
    pub name: &'static str,
    pub exercises: Vec<PlanExercise>,
}

pub struct WorkoutPlan {
    pub id: u32,
    pub name: &'static str,
    pub difficulty: &'static str,
    pub description: &'static str,
    pub tip: &'static str,
    pub days: Vec<PlanDay>,
}

fn get_plans() -> Vec<WorkoutPlan> {
    vec![
        WorkoutPlan {
            id: 1,
            name: "StrongLifts 5x5",
            difficulty: "Beginner",
            description: "Best for: Complete beginners. Only 3 exercises per day, focus on learning compound movements with linear weight progression. Add 2.5kg upper body / 5kg lower body each session.",
            tip: "Start with empty bar (20kg) on all lifts except Deadlift (40kg) and Row (30kg). Focus on form.",
            days: vec![
                PlanDay {
                    name: "Day A",
                    exercises: vec![
                        PlanExercise { name: "Barbell Squat", sets: 5, reps: 5, weight_kg: 20.0 },
                        PlanExercise { name: "Bench Press", sets: 5, reps: 5, weight_kg: 20.0 },
                        PlanExercise { name: "Barbell Row", sets: 5, reps: 5, weight_kg: 30.0 },
                    ],
                },
                PlanDay {
                    name: "Day B",
                    exercises: vec![
                        PlanExercise { name: "Barbell Squat", sets: 5, reps: 5, weight_kg: 20.0 },
                        PlanExercise { name: "Overhead Press", sets: 5, reps: 5, weight_kg: 20.0 },
                        PlanExercise { name: "Deadlift", sets: 1, reps: 5, weight_kg: 40.0 },
                    ],
                },
                PlanDay {
                    name: "Day C",
                    exercises: vec![
                        PlanExercise { name: "Barbell Squat", sets: 5, reps: 5, weight_kg: 20.0 },
                        PlanExercise { name: "Bench Press", sets: 5, reps: 5, weight_kg: 20.0 },
                        PlanExercise { name: "Barbell Row", sets: 5, reps: 5, weight_kg: 30.0 },
                    ],
                },
            ],
        },
        WorkoutPlan {
            id: 2,
            name: "Push / Pull / Legs",
            difficulty: "Intermediate",
            description: "Best for: Lifters with 2-3 months of experience who want more volume per muscle group. Each day targets specific movement patterns for balanced development.",
            tip: "When you can complete all sets and reps with good form, add 2.5kg next session.",
            days: vec![
                PlanDay {
                    name: "Day A \u{2014} Push",
                    exercises: vec![
                        PlanExercise { name: "Bench Press", sets: 4, reps: 8, weight_kg: 0.0 },
                        PlanExercise { name: "Overhead Press", sets: 3, reps: 10, weight_kg: 0.0 },
                        PlanExercise { name: "Incline Dumbbell Press", sets: 3, reps: 10, weight_kg: 0.0 },
                        PlanExercise { name: "Lateral Raise", sets: 3, reps: 15, weight_kg: 0.0 },
                        PlanExercise { name: "Tricep Pushdown", sets: 3, reps: 12, weight_kg: 0.0 },
                        PlanExercise { name: "Dip", sets: 3, reps: 8, weight_kg: 0.0 },
                    ],
                },
                PlanDay {
                    name: "Day B \u{2014} Pull",
                    exercises: vec![
                        PlanExercise { name: "Deadlift", sets: 3, reps: 5, weight_kg: 0.0 },
                        PlanExercise { name: "Barbell Row", sets: 4, reps: 8, weight_kg: 0.0 },
                        PlanExercise { name: "Lat Pulldown", sets: 3, reps: 10, weight_kg: 0.0 },
                        PlanExercise { name: "Face Pull", sets: 3, reps: 15, weight_kg: 0.0 },
                        PlanExercise { name: "Barbell Curl", sets: 3, reps: 10, weight_kg: 0.0 },
                        PlanExercise { name: "Hammer Curl", sets: 3, reps: 10, weight_kg: 0.0 },
                    ],
                },
                PlanDay {
                    name: "Day C \u{2014} Legs",
                    exercises: vec![
                        PlanExercise { name: "Barbell Squat", sets: 4, reps: 8, weight_kg: 0.0 },
                        PlanExercise { name: "Leg Press", sets: 3, reps: 12, weight_kg: 0.0 },
                        PlanExercise { name: "Romanian Deadlift", sets: 3, reps: 10, weight_kg: 0.0 },
                        PlanExercise { name: "Leg Curl", sets: 3, reps: 12, weight_kg: 0.0 },
                        PlanExercise { name: "Lunges", sets: 3, reps: 10, weight_kg: 0.0 },
                        PlanExercise { name: "Seated Calf Raise", sets: 3, reps: 15, weight_kg: 0.0 },
                    ],
                },
            ],
        },
        WorkoutPlan {
            id: 3,
            name: "Full Body 3x",
            difficulty: "Versatile",
            description: "Best for: Anyone who wants balanced training every session. Great for busy schedules \u{2014} miss a day and you've still hit everything. Varies intensity across days (heavy/moderate/light).",
            tip: "Heavy day = challenging weight, long rests (3min). Light day = moderate weight, shorter rests (60-90s). Increase weight when all reps feel controlled.",
            days: vec![
                PlanDay {
                    name: "Day A \u{2014} Heavy",
                    exercises: vec![
                        PlanExercise { name: "Barbell Squat", sets: 4, reps: 5, weight_kg: 0.0 },
                        PlanExercise { name: "Bench Press", sets: 4, reps: 5, weight_kg: 0.0 },
                        PlanExercise { name: "Barbell Row", sets: 4, reps: 5, weight_kg: 0.0 },
                        PlanExercise { name: "Face Pull", sets: 3, reps: 15, weight_kg: 0.0 },
                        PlanExercise { name: "Plank", sets: 3, reps: 30, weight_kg: 0.0 },
                    ],
                },
                PlanDay {
                    name: "Day B \u{2014} Moderate",
                    exercises: vec![
                        PlanExercise { name: "Deadlift", sets: 3, reps: 5, weight_kg: 0.0 },
                        PlanExercise { name: "Overhead Press", sets: 3, reps: 8, weight_kg: 0.0 },
                        PlanExercise { name: "Lat Pulldown", sets: 3, reps: 10, weight_kg: 0.0 },
                        PlanExercise { name: "Lunges", sets: 3, reps: 10, weight_kg: 0.0 },
                        PlanExercise { name: "Barbell Curl", sets: 3, reps: 10, weight_kg: 0.0 },
                        PlanExercise { name: "Cable Crunch", sets: 3, reps: 15, weight_kg: 0.0 },
                    ],
                },
                PlanDay {
                    name: "Day C \u{2014} Light/Volume",
                    exercises: vec![
                        PlanExercise { name: "Leg Press", sets: 3, reps: 12, weight_kg: 0.0 },
                        PlanExercise { name: "Incline Dumbbell Press", sets: 3, reps: 12, weight_kg: 0.0 },
                        PlanExercise { name: "Dumbbell Row", sets: 3, reps: 12, weight_kg: 0.0 },
                        PlanExercise { name: "Lateral Raise", sets: 3, reps: 15, weight_kg: 0.0 },
                        PlanExercise { name: "Tricep Pushdown", sets: 3, reps: 12, weight_kg: 0.0 },
                        PlanExercise { name: "Hammer Curl", sets: 3, reps: 12, weight_kg: 0.0 },
                    ],
                },
            ],
        },
        WorkoutPlan {
            id: 4,
            name: "Knee-Friendly Push / Pull / Legs",
            difficulty: "Beginner",
            description: "Modified PPL for knee injuries. Avoids squats and lunges, replacing them with machine-based quad work (leg press, leg extension) and hip-hinge movements (deadlift, Romanian deadlift) that are easier on the knees.",
            tip: "Use controlled range of motion on all machine exercises. Consult a physiotherapist to confirm these movements are appropriate for your specific condition.",
            days: vec![
                PlanDay {
                    name: "Day A \u{2014} Push",
                    exercises: vec![
                        PlanExercise { name: "Bench Press", sets: 4, reps: 8, weight_kg: 0.0 },
                        PlanExercise { name: "Overhead Press", sets: 3, reps: 10, weight_kg: 0.0 },
                        PlanExercise { name: "Incline Dumbbell Press", sets: 3, reps: 10, weight_kg: 0.0 },
                        PlanExercise { name: "Lateral Raise", sets: 3, reps: 15, weight_kg: 0.0 },
                        PlanExercise { name: "Tricep Pushdown", sets: 3, reps: 12, weight_kg: 0.0 },
                        PlanExercise { name: "Dip", sets: 3, reps: 8, weight_kg: 0.0 },
                    ],
                },
                PlanDay {
                    name: "Day B \u{2014} Pull",
                    exercises: vec![
                        PlanExercise { name: "Deadlift", sets: 3, reps: 5, weight_kg: 0.0 },
                        PlanExercise { name: "Barbell Row", sets: 4, reps: 8, weight_kg: 0.0 },
                        PlanExercise { name: "Lat Pulldown", sets: 3, reps: 10, weight_kg: 0.0 },
                        PlanExercise { name: "Face Pull", sets: 3, reps: 15, weight_kg: 0.0 },
                        PlanExercise { name: "Barbell Curl", sets: 3, reps: 10, weight_kg: 0.0 },
                        PlanExercise { name: "Hammer Curl", sets: 3, reps: 10, weight_kg: 0.0 },
                    ],
                },
                PlanDay {
                    name: "Day C \u{2014} Legs (Knee-Safe)",
                    exercises: vec![
                        PlanExercise { name: "Leg Press", sets: 3, reps: 12, weight_kg: 0.0 },
                        PlanExercise { name: "Romanian Deadlift", sets: 3, reps: 10, weight_kg: 0.0 },
                        PlanExercise { name: "Leg Extension", sets: 3, reps: 12, weight_kg: 0.0 },
                        PlanExercise { name: "Leg Curl", sets: 3, reps: 12, weight_kg: 0.0 },
                        PlanExercise { name: "Seated Calf Raise", sets: 3, reps: 15, weight_kg: 0.0 },
                        PlanExercise { name: "Plank", sets: 3, reps: 30, weight_kg: 0.0 },
                    ],
                },
            ],
        },
        WorkoutPlan {
            id: 5,
            name: "Knee-Friendly Upper / Lower",
            difficulty: "Beginner",
            description: "Upper/lower split designed for knee injuries. Two upper-body days (push and pull focus) plus a lower-body day built around hip-hinge and machine movements that avoid deep knee flexion under load.",
            tip: "Start light on lower-body day and focus on posterior chain (hamstrings, glutes). Increase weight only when movement feels comfortable and controlled.",
            days: vec![
                PlanDay {
                    name: "Day A \u{2014} Upper Push",
                    exercises: vec![
                        PlanExercise { name: "Bench Press", sets: 4, reps: 8, weight_kg: 0.0 },
                        PlanExercise { name: "Overhead Press", sets: 3, reps: 10, weight_kg: 0.0 },
                        PlanExercise { name: "Incline Dumbbell Press", sets: 3, reps: 10, weight_kg: 0.0 },
                        PlanExercise { name: "Lateral Raise", sets: 3, reps: 15, weight_kg: 0.0 },
                        PlanExercise { name: "Tricep Pushdown", sets: 3, reps: 12, weight_kg: 0.0 },
                        PlanExercise { name: "Overhead Tricep Extension", sets: 3, reps: 10, weight_kg: 0.0 },
                    ],
                },
                PlanDay {
                    name: "Day B \u{2014} Upper Pull",
                    exercises: vec![
                        PlanExercise { name: "Barbell Row", sets: 4, reps: 8, weight_kg: 0.0 },
                        PlanExercise { name: "Lat Pulldown", sets: 3, reps: 10, weight_kg: 0.0 },
                        PlanExercise { name: "Dumbbell Row", sets: 3, reps: 12, weight_kg: 0.0 },
                        PlanExercise { name: "Face Pull", sets: 3, reps: 15, weight_kg: 0.0 },
                        PlanExercise { name: "Barbell Curl", sets: 3, reps: 10, weight_kg: 0.0 },
                        PlanExercise { name: "Hammer Curl", sets: 3, reps: 10, weight_kg: 0.0 },
                    ],
                },
                PlanDay {
                    name: "Day C \u{2014} Lower Body + Core",
                    exercises: vec![
                        PlanExercise { name: "Deadlift", sets: 3, reps: 5, weight_kg: 0.0 },
                        PlanExercise { name: "Leg Press", sets: 3, reps: 12, weight_kg: 0.0 },
                        PlanExercise { name: "Romanian Deadlift", sets: 3, reps: 10, weight_kg: 0.0 },
                        PlanExercise { name: "Leg Curl", sets: 3, reps: 12, weight_kg: 0.0 },
                        PlanExercise { name: "Seated Calf Raise", sets: 3, reps: 15, weight_kg: 0.0 },
                        PlanExercise { name: "Cable Crunch", sets: 3, reps: 15, weight_kg: 0.0 },
                    ],
                },
            ],
        },
        WorkoutPlan {
            id: 6,
            name: "Knee-Friendly Full Body",
            difficulty: "Versatile",
            description: "Full-body sessions with hip-hinge emphasis. Every session trains upper and lower body while protecting knees \u{2014} no squats or lunges. Uses a heavy/moderate/light structure across the three days.",
            tip: "Heavy day = lower reps, longer rests (3min). Light day = higher reps, shorter rests (60-90s). All lower-body work uses machines or hip-hinge patterns.",
            days: vec![
                PlanDay {
                    name: "Day A \u{2014} Heavy Compound",
                    exercises: vec![
                        PlanExercise { name: "Deadlift", sets: 3, reps: 5, weight_kg: 0.0 },
                        PlanExercise { name: "Bench Press", sets: 4, reps: 5, weight_kg: 0.0 },
                        PlanExercise { name: "Barbell Row", sets: 4, reps: 5, weight_kg: 0.0 },
                        PlanExercise { name: "Leg Curl", sets: 3, reps: 12, weight_kg: 0.0 },
                        PlanExercise { name: "Face Pull", sets: 3, reps: 15, weight_kg: 0.0 },
                        PlanExercise { name: "Plank", sets: 3, reps: 30, weight_kg: 0.0 },
                    ],
                },
                PlanDay {
                    name: "Day B \u{2014} Moderate",
                    exercises: vec![
                        PlanExercise { name: "Leg Press", sets: 3, reps: 12, weight_kg: 0.0 },
                        PlanExercise { name: "Overhead Press", sets: 3, reps: 8, weight_kg: 0.0 },
                        PlanExercise { name: "Lat Pulldown", sets: 3, reps: 10, weight_kg: 0.0 },
                        PlanExercise { name: "Leg Extension", sets: 3, reps: 12, weight_kg: 0.0 },
                        PlanExercise { name: "Barbell Curl", sets: 3, reps: 10, weight_kg: 0.0 },
                        PlanExercise { name: "Cable Crunch", sets: 3, reps: 15, weight_kg: 0.0 },
                    ],
                },
                PlanDay {
                    name: "Day C \u{2014} Light / Volume",
                    exercises: vec![
                        PlanExercise { name: "Romanian Deadlift", sets: 3, reps: 12, weight_kg: 0.0 },
                        PlanExercise { name: "Incline Dumbbell Press", sets: 3, reps: 12, weight_kg: 0.0 },
                        PlanExercise { name: "Dumbbell Row", sets: 3, reps: 12, weight_kg: 0.0 },
                        PlanExercise { name: "Seated Calf Raise", sets: 3, reps: 15, weight_kg: 0.0 },
                        PlanExercise { name: "Lateral Raise", sets: 3, reps: 15, weight_kg: 0.0 },
                        PlanExercise { name: "Hammer Curl", sets: 3, reps: 12, weight_kg: 0.0 },
                    ],
                },
            ],
        },
    ]
}

pub async fn list(
    auth: AuthUser,
) -> Result<Html<String>, AppError> {
    let plans = get_plans();
    let tmpl = templates::PlanListTemplate {
        username: auth.username,
        active_page: "plans".to_string(),
        plans,
    };
    Ok(Html(tmpl.to_string()))
}

pub async fn use_plan(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(plan_id): Path<u32>,
) -> Result<axum::response::Response, AppError> {
    let plans = get_plans();
    let plan = plans.into_iter().find(|p| p.id == plan_id);

    let plan = match plan {
        Some(p) => p,
        None => return Err(AppError::NotFound),
    };

    // Find next Mon/Wed/Fri from today
    let today = chrono::Local::now().date_naive();
    let weekday = today.weekday();

    let days_to_monday = match weekday {
        chrono::Weekday::Mon => 7, // next monday
        chrono::Weekday::Tue => 6,
        chrono::Weekday::Wed => 5,
        chrono::Weekday::Thu => 4,
        chrono::Weekday::Fri => 3,
        chrono::Weekday::Sat => 2,
        chrono::Weekday::Sun => 1,
    };

    let next_monday = today + chrono::Duration::days(days_to_monday);
    let next_wednesday = next_monday + chrono::Duration::days(2);
    let next_friday = next_monday + chrono::Duration::days(4);

    let schedule_dates = [next_monday, next_wednesday, next_friday];

    for (i, day) in plan.days.iter().enumerate() {
        let date = schedule_dates.get(i).copied().unwrap_or(next_friday);
        let scheduled_at = format!("{}T09:00", date);

        let session_name = format!("{} - {}", plan.name, day.name);
        let session_id = WorkoutSession::create(
            &state.pool,
            auth.user_id,
            &session_name,
            &scheduled_at,
            "",
            "planned",
        )
        .await?;

        for (j, exercise) in day.exercises.iter().enumerate() {
            if let Some(ex) = Exercise::find_by_name_and_user(&state.pool, exercise.name, auth.user_id).await? {
                SessionExercise::create(
                    &state.pool,
                    session_id,
                    ex.id,
                    exercise.sets,
                    exercise.reps,
                    exercise.weight_kg,
                    j as i64,
                )
                .await?;
            }
        }
    }

    let message = format!(
        "{}+plan+created!+3+sessions+scheduled+for+Mon/Wed/Fri.",
        plan.name.replace(' ', "+")
    );
    Ok(Redirect::to(&format!("/sessions?message={message}")).into_response())
}
