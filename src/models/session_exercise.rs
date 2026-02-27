use std::collections::HashMap;
use sqlx::SqlitePool;

#[derive(Debug, sqlx::FromRow)]
pub struct SessionExercise {
    pub id: i64,
    pub workout_session_id: i64,
    pub exercise_id: i64,
    pub planned_sets: i64,
    pub planned_reps: i64,
    pub planned_weight_kg: f64,
    pub sort_order: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct SessionExerciseDetail {
    pub id: i64,
    pub workout_session_id: i64,
    pub exercise_id: i64,
    pub exercise_name: String,
    pub muscle_group: String,
    pub planned_sets: i64,
    pub planned_reps: i64,
    pub planned_weight_kg: f64,
    pub sort_order: i64,
}

impl SessionExercise {
    pub async fn find_by_session(pool: &SqlitePool, session_id: i64, user_id: i64) -> sqlx::Result<Vec<SessionExerciseDetail>> {
        sqlx::query_as::<_, SessionExerciseDetail>(
            "SELECT se.id, se.workout_session_id, se.exercise_id, e.name as exercise_name, e.muscle_group, se.planned_sets, se.planned_reps, se.planned_weight_kg, se.sort_order \
             FROM session_exercises se \
             JOIN exercises e ON se.exercise_id = e.id \
             JOIN workout_sessions ws ON se.workout_session_id = ws.id \
             WHERE se.workout_session_id = ? AND ws.user_id = ? \
             ORDER BY se.sort_order"
        )
        .bind(session_id)
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    pub async fn create(pool: &SqlitePool, session_id: i64, exercise_id: i64, sets: i64, reps: i64, weight: f64, sort_order: i64) -> sqlx::Result<i64> {
        let result = sqlx::query(
            "INSERT INTO session_exercises (workout_session_id, exercise_id, planned_sets, planned_reps, planned_weight_kg, sort_order) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(session_id)
        .bind(exercise_id)
        .bind(sets)
        .bind(reps)
        .bind(weight)
        .bind(sort_order)
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn delete_by_session(pool: &SqlitePool, session_id: i64, user_id: i64) -> sqlx::Result<()> {
        sqlx::query(
            "DELETE FROM session_exercises WHERE workout_session_id IN \
             (SELECT id FROM workout_sessions WHERE id = ? AND user_id = ?)"
        )
            .bind(session_id)
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn find_exercise_names_for_user(pool: &SqlitePool, user_id: i64) -> sqlx::Result<HashMap<i64, Vec<String>>> {
        let rows = sqlx::query_as::<_, (i64, String)>(
            "SELECT se.workout_session_id, e.name \
             FROM session_exercises se \
             JOIN exercises e ON se.exercise_id = e.id \
             JOIN workout_sessions ws ON se.workout_session_id = ws.id \
             WHERE ws.user_id = ? \
             ORDER BY se.workout_session_id, se.sort_order"
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let mut map: HashMap<i64, Vec<String>> = HashMap::new();
        for (session_id, name) in rows {
            map.entry(session_id).or_default().push(name);
        }
        Ok(map)
    }
}
