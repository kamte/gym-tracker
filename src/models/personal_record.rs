use sqlx::SqlitePool;

#[derive(Debug, sqlx::FromRow)]
pub struct PersonalRecord {
    pub id: i64,
    pub user_id: i64,
    pub exercise_id: i64,
    pub record_type: String,
    pub value: f64,
    pub exercise_log_id: Option<i64>,
    pub achieved_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct PersonalRecordDetail {
    pub id: i64,
    pub user_id: i64,
    pub exercise_id: i64,
    pub exercise_name: String,
    pub muscle_group: String,
    pub record_type: String,
    pub value: f64,
    pub achieved_at: String,
}

impl PersonalRecord {
    pub async fn find_by_exercise(
        pool: &SqlitePool, exercise_id: i64, user_id: i64,
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as::<_, Self>(
            "SELECT id, user_id, exercise_id, record_type, value, exercise_log_id, achieved_at \
             FROM personal_records WHERE exercise_id = ? AND user_id = ? ORDER BY record_type"
        )
        .bind(exercise_id)
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_all_by_user(pool: &SqlitePool, user_id: i64) -> sqlx::Result<Vec<PersonalRecordDetail>> {
        sqlx::query_as::<_, PersonalRecordDetail>(
            "SELECT pr.id, pr.user_id, pr.exercise_id, e.name as exercise_name, e.muscle_group, \
             pr.record_type, pr.value, pr.achieved_at \
             FROM personal_records pr \
             JOIN exercises e ON pr.exercise_id = e.id \
             WHERE pr.user_id = ? \
             ORDER BY e.name, pr.record_type"
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_recent_by_user(pool: &SqlitePool, user_id: i64, limit: i64) -> sqlx::Result<Vec<PersonalRecordDetail>> {
        sqlx::query_as::<_, PersonalRecordDetail>(
            "SELECT pr.id, pr.user_id, pr.exercise_id, e.name as exercise_name, e.muscle_group, \
             pr.record_type, pr.value, pr.achieved_at \
             FROM personal_records pr \
             JOIN exercises e ON pr.exercise_id = e.id \
             WHERE pr.user_id = ? \
             ORDER BY pr.achieved_at DESC LIMIT ?"
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    /// Check if the given log values beat any existing PRs for the exercise.
    /// Returns a list of record types that were broken.
    pub async fn check_and_update(
        pool: &SqlitePool, user_id: i64, exercise_id: i64,
        log_id: i64, weight: f64, reps: i64, set_type: &str,
    ) -> sqlx::Result<Vec<String>> {
        // Don't track PRs for warm-up sets
        if set_type == "warmup" {
            return Ok(vec![]);
        }

        let mut broken = Vec::new();
        let volume = weight * reps as f64;
        let estimated_1rm = weight * (1.0 + reps as f64 / 30.0);

        let records = vec![
            ("max_weight", weight),
            ("max_reps", reps as f64),
            ("max_volume", volume),
            ("estimated_1rm", estimated_1rm),
        ];

        for (record_type, new_value) in records {
            if new_value <= 0.0 {
                continue;
            }

            let existing: Option<(f64,)> = sqlx::query_as(
                "SELECT value FROM personal_records WHERE user_id = ? AND exercise_id = ? AND record_type = ?"
            )
            .bind(user_id)
            .bind(exercise_id)
            .bind(record_type)
            .fetch_optional(pool)
            .await?;

            let is_new_pr = match existing {
                Some((current,)) => new_value > current,
                None => true,
            };

            if is_new_pr {
                sqlx::query(
                    "INSERT INTO personal_records (user_id, exercise_id, record_type, value, exercise_log_id, achieved_at) \
                     VALUES (?, ?, ?, ?, ?, datetime('now')) \
                     ON CONFLICT(user_id, exercise_id, record_type) \
                     DO UPDATE SET value = excluded.value, exercise_log_id = excluded.exercise_log_id, achieved_at = excluded.achieved_at"
                )
                .bind(user_id)
                .bind(exercise_id)
                .bind(record_type)
                .bind(new_value)
                .bind(log_id)
                .execute(pool)
                .await?;
                broken.push(record_type.to_string());
            }
        }

        Ok(broken)
    }
}
