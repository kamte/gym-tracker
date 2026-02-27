use sqlx::SqlitePool;

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: String,
}

impl User {
    pub async fn find_by_username(pool: &SqlitePool, username: &str) -> sqlx::Result<Option<Self>> {
        sqlx::query_as::<_, Self>("SELECT id, username, email, password_hash, created_at FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: i64) -> sqlx::Result<Option<Self>> {
        sqlx::query_as::<_, Self>("SELECT id, username, email, password_hash, created_at FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn create(pool: &SqlitePool, username: &str, email: &str, password_hash: &str) -> sqlx::Result<i64> {
        let result = sqlx::query("INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)")
            .bind(username)
            .bind(email)
            .bind(password_hash)
            .execute(pool)
            .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn exists_by_username_or_email(pool: &SqlitePool, username: &str, email: &str) -> sqlx::Result<bool> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE username = ? OR email = ?")
            .bind(username)
            .bind(email)
            .fetch_one(pool)
            .await?;
        Ok(row.0 > 0)
    }
}
