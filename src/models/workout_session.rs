use sqlx::SqlitePool;

#[derive(Debug, sqlx::FromRow)]
pub struct WorkoutSession {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub scheduled_at: String,
    pub notes: String,
    pub status: String,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

impl WorkoutSession {
    pub async fn find_all_by_user(pool: &SqlitePool, user_id: i64) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as::<_, Self>(
            "SELECT id, user_id, name, scheduled_at, notes, status, created_at, started_at, completed_at FROM workout_sessions WHERE user_id = ? ORDER BY scheduled_at DESC"
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: i64, user_id: i64) -> sqlx::Result<Option<Self>> {
        sqlx::query_as::<_, Self>(
            "SELECT id, user_id, name, scheduled_at, notes, status, created_at, started_at, completed_at FROM workout_sessions WHERE id = ? AND user_id = ?"
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(pool: &SqlitePool, user_id: i64, name: &str, scheduled_at: &str, notes: &str, status: &str) -> sqlx::Result<i64> {
        let result = sqlx::query(
            "INSERT INTO workout_sessions (user_id, name, scheduled_at, notes, status) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(name)
        .bind(scheduled_at)
        .bind(notes)
        .bind(status)
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn update(pool: &SqlitePool, id: i64, user_id: i64, name: &str, scheduled_at: &str, notes: &str, status: &str) -> sqlx::Result<bool> {
        let result = sqlx::query(
            "UPDATE workout_sessions SET name = ?, scheduled_at = ?, notes = ?, status = ? WHERE id = ? AND user_id = ?"
        )
        .bind(name)
        .bind(scheduled_at)
        .bind(notes)
        .bind(status)
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete(pool: &SqlitePool, id: i64, user_id: i64) -> sqlx::Result<bool> {
        sqlx::query("UPDATE exercise_logs SET workout_session_id = NULL WHERE workout_session_id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;

        let result = sqlx::query("DELETE FROM workout_sessions WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn find_upcoming_by_user(pool: &SqlitePool, user_id: i64, limit: i64) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as::<_, Self>(
            "SELECT id, user_id, name, scheduled_at, notes, status, created_at, started_at, completed_at FROM workout_sessions WHERE user_id = ? AND status = 'planned' AND scheduled_at >= datetime('now') ORDER BY scheduled_at ASC LIMIT ?"
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    pub async fn count_by_user(pool: &SqlitePool, user_id: i64) -> sqlx::Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM workout_sessions WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
        Ok(row.0)
    }

    pub async fn find_today_by_user(pool: &SqlitePool, user_id: i64) -> sqlx::Result<Option<Self>> {
        sqlx::query_as::<_, Self>(
            "SELECT id, user_id, name, scheduled_at, notes, status, created_at, started_at, completed_at FROM workout_sessions \
             WHERE user_id = ? AND status = 'planned' AND date(scheduled_at) = date('now') \
             ORDER BY scheduled_at ASC LIMIT 1"
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn start_workout(pool: &SqlitePool, id: i64, user_id: i64) -> sqlx::Result<bool> {
        let result = sqlx::query(
            "UPDATE workout_sessions SET started_at = datetime('now') WHERE id = ? AND user_id = ? AND started_at IS NULL"
        )
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn complete_workout(pool: &SqlitePool, id: i64, user_id: i64) -> sqlx::Result<bool> {
        let result = sqlx::query(
            "UPDATE workout_sessions SET status = 'completed', completed_at = datetime('now') WHERE id = ? AND user_id = ?"
        )
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn find_completed_in_month(pool: &SqlitePool, user_id: i64, year: i32, month: u32) -> sqlx::Result<Vec<Self>> {
        let start = format!("{:04}-{:02}-01", year, month);
        let end = if month == 12 {
            format!("{:04}-01-01", year + 1)
        } else {
            format!("{:04}-{:02}-01", year, month + 1)
        };
        sqlx::query_as::<_, Self>(
            "SELECT id, user_id, name, scheduled_at, notes, status, created_at, started_at, completed_at \
             FROM workout_sessions \
             WHERE user_id = ? AND status = 'completed' \
             AND (completed_at >= ? OR scheduled_at >= ?) \
             AND (completed_at < ? OR scheduled_at < ?) \
             ORDER BY COALESCE(completed_at, scheduled_at) ASC"
        )
        .bind(user_id)
        .bind(&start)
        .bind(&start)
        .bind(&end)
        .bind(&end)
        .fetch_all(pool)
        .await
    }

    pub async fn find_completed_by_user(pool: &SqlitePool, user_id: i64) -> sqlx::Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM workout_sessions WHERE user_id = ? AND status = 'completed'"
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;
        Ok(row.0)
    }
}
