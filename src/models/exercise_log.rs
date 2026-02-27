use sqlx::SqlitePool;

#[derive(Debug, sqlx::FromRow)]
pub struct ExerciseLog {
    pub id: i64,
    pub user_id: i64,
    pub exercise_id: i64,
    pub workout_session_id: Option<i64>,
    pub set_number: i64,
    pub reps_completed: i64,
    pub weight_kg: f64,
    pub rpe: Option<f64>,
    pub performed_at: String,
    pub set_type: String,
    pub notes: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ExerciseLogDetail {
    pub id: i64,
    pub user_id: i64,
    pub exercise_id: i64,
    pub exercise_name: String,
    pub muscle_group: String,
    pub workout_session_id: Option<i64>,
    pub session_name: Option<String>,
    pub set_number: i64,
    pub reps_completed: i64,
    pub weight_kg: f64,
    pub rpe: Option<f64>,
    pub performed_at: String,
    pub set_type: String,
    pub notes: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ProgressDataPoint {
    pub date: String,
    pub max_weight: f64,
    pub total_volume: f64,
    pub max_estimated_1rm: f64,
}

impl ExerciseLog {
    pub async fn find_all_by_user(pool: &SqlitePool, user_id: i64) -> sqlx::Result<Vec<ExerciseLogDetail>> {
        sqlx::query_as::<_, ExerciseLogDetail>(
            "SELECT el.id, el.user_id, el.exercise_id, e.name as exercise_name, e.muscle_group, \
             el.workout_session_id, ws.name as session_name, \
             el.set_number, el.reps_completed, el.weight_kg, el.rpe, el.performed_at, \
             el.set_type, el.notes \
             FROM exercise_logs el \
             JOIN exercises e ON el.exercise_id = e.id \
             LEFT JOIN workout_sessions ws ON el.workout_session_id = ws.id \
             WHERE el.user_id = ? ORDER BY el.performed_at DESC"
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: i64, user_id: i64) -> sqlx::Result<Option<ExerciseLogDetail>> {
        sqlx::query_as::<_, ExerciseLogDetail>(
            "SELECT el.id, el.user_id, el.exercise_id, e.name as exercise_name, e.muscle_group, \
             el.workout_session_id, ws.name as session_name, \
             el.set_number, el.reps_completed, el.weight_kg, el.rpe, el.performed_at, \
             el.set_type, el.notes \
             FROM exercise_logs el \
             JOIN exercises e ON el.exercise_id = e.id \
             LEFT JOIN workout_sessions ws ON el.workout_session_id = ws.id \
             WHERE el.id = ? AND el.user_id = ?"
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_recent_by_user(pool: &SqlitePool, user_id: i64, limit: i64) -> sqlx::Result<Vec<ExerciseLogDetail>> {
        sqlx::query_as::<_, ExerciseLogDetail>(
            "SELECT el.id, el.user_id, el.exercise_id, e.name as exercise_name, e.muscle_group, \
             el.workout_session_id, ws.name as session_name, \
             el.set_number, el.reps_completed, el.weight_kg, el.rpe, el.performed_at, \
             el.set_type, el.notes \
             FROM exercise_logs el \
             JOIN exercises e ON el.exercise_id = e.id \
             LEFT JOIN workout_sessions ws ON el.workout_session_id = ws.id \
             WHERE el.user_id = ? ORDER BY el.performed_at DESC LIMIT ?"
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    pub async fn create(
        pool: &SqlitePool, user_id: i64, exercise_id: i64, session_id: Option<i64>,
        set_number: i64, reps: i64, weight: f64, rpe: Option<f64>,
        set_type: &str, notes: &str,
    ) -> sqlx::Result<i64> {
        let result = sqlx::query(
            "INSERT INTO exercise_logs (user_id, exercise_id, workout_session_id, set_number, reps_completed, weight_kg, rpe, set_type, notes) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(exercise_id)
        .bind(session_id)
        .bind(set_number)
        .bind(reps)
        .bind(weight)
        .bind(rpe)
        .bind(set_type)
        .bind(notes)
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn delete(pool: &SqlitePool, id: i64, user_id: i64) -> sqlx::Result<bool> {
        let result = sqlx::query("DELETE FROM exercise_logs WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn count_by_user(pool: &SqlitePool, user_id: i64) -> sqlx::Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM exercise_logs WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
        Ok(row.0)
    }

    pub async fn find_by_exercise_and_user(
        pool: &SqlitePool, exercise_id: i64, user_id: i64, limit: i64
    ) -> sqlx::Result<Vec<ExerciseLogDetail>> {
        sqlx::query_as::<_, ExerciseLogDetail>(
            "SELECT el.id, el.user_id, el.exercise_id, e.name as exercise_name, e.muscle_group, \
             el.workout_session_id, ws.name as session_name, \
             el.set_number, el.reps_completed, el.weight_kg, el.rpe, el.performed_at, \
             el.set_type, el.notes \
             FROM exercise_logs el \
             JOIN exercises e ON el.exercise_id = e.id \
             LEFT JOIN workout_sessions ws ON el.workout_session_id = ws.id \
             WHERE el.exercise_id = ? AND el.user_id = ? \
             ORDER BY el.performed_at DESC LIMIT ?"
        )
        .bind(exercise_id)
        .bind(user_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    pub async fn find_last_weight_by_exercise_and_user(
        pool: &SqlitePool, exercise_id: i64, user_id: i64
    ) -> sqlx::Result<Option<f64>> {
        sqlx::query_scalar::<_, f64>(
            "SELECT weight_kg FROM exercise_logs WHERE exercise_id = ? AND user_id = ? ORDER BY performed_at DESC LIMIT 1"
        )
        .bind(exercise_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_session(
        pool: &SqlitePool, session_id: i64, user_id: i64
    ) -> sqlx::Result<Vec<ExerciseLog>> {
        sqlx::query_as::<_, ExerciseLog>(
            "SELECT id, user_id, exercise_id, workout_session_id, set_number, reps_completed, weight_kg, rpe, performed_at, set_type, notes \
             FROM exercise_logs WHERE workout_session_id = ? AND user_id = ? ORDER BY exercise_id, set_number"
        )
        .bind(session_id)
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_progress_by_exercise(
        pool: &SqlitePool, exercise_id: i64, user_id: i64
    ) -> sqlx::Result<Vec<ProgressDataPoint>> {
        sqlx::query_as::<_, ProgressDataPoint>(
            "SELECT date(performed_at) as date, \
             MAX(weight_kg) as max_weight, \
             SUM(weight_kg * reps_completed) as total_volume, \
             MAX(weight_kg * (1.0 + reps_completed / 30.0)) as max_estimated_1rm \
             FROM exercise_logs \
             WHERE exercise_id = ? AND user_id = ? AND set_type != 'warmup' \
             GROUP BY date(performed_at) \
             ORDER BY date(performed_at) ASC"
        )
        .bind(exercise_id)
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    pub async fn count_by_session(pool: &SqlitePool, session_id: i64, user_id: i64) -> sqlx::Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM exercise_logs WHERE workout_session_id = ? AND user_id = ?"
        )
        .bind(session_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;
        Ok(row.0)
    }

    pub async fn total_volume_by_session(pool: &SqlitePool, session_id: i64, user_id: i64) -> sqlx::Result<f64> {
        let row: (f64,) = sqlx::query_as(
            "SELECT COALESCE(SUM(weight_kg * reps_completed), 0.0) FROM exercise_logs WHERE workout_session_id = ? AND user_id = ? AND set_type != 'warmup'"
        )
        .bind(session_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;
        Ok(row.0)
    }

    pub async fn find_for_csv_export(pool: &SqlitePool, user_id: i64) -> sqlx::Result<Vec<ExerciseLogDetail>> {
        sqlx::query_as::<_, ExerciseLogDetail>(
            "SELECT el.id, el.user_id, el.exercise_id, e.name as exercise_name, e.muscle_group, \
             el.workout_session_id, ws.name as session_name, \
             el.set_number, el.reps_completed, el.weight_kg, el.rpe, el.performed_at, \
             el.set_type, el.notes \
             FROM exercise_logs el \
             JOIN exercises e ON el.exercise_id = e.id \
             LEFT JOIN workout_sessions ws ON el.workout_session_id = ws.id \
             WHERE el.user_id = ? ORDER BY el.performed_at ASC"
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }
}
