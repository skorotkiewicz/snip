use sqlx::{Pool, Sqlite};

#[derive(Clone)]
pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }
    /// Atomically count an attempt in a fixed window, with durable local limits.
    pub async fn check_rate_limit(
        &self,
        scope: &str,
        identifier: &str,
        max_requests: u32,
        window_seconds: u32,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query("DELETE FROM rate_limits WHERE expires_at <= unixepoch()")
            .execute(&self.pool)
            .await?;
        let count: i64 = sqlx::query_scalar(
            "INSERT INTO rate_limits (scope, identifier, requests, expires_at)
             VALUES (?, ?, 1, unixepoch() + ?)
             ON CONFLICT(scope, identifier) DO UPDATE SET
                 requests = CASE WHEN expires_at <= unixepoch() THEN 1 ELSE MIN(requests + 1, ?) END,
                 expires_at = CASE WHEN expires_at <= unixepoch() THEN excluded.expires_at ELSE expires_at END
             RETURNING requests",
        )
        .bind(scope)
        .bind(identifier)
        .bind(i64::from(window_seconds))
        .bind(i64::from(max_requests) + 1)
        .fetch_one(&self.pool)
        .await?;
        Ok(count <= i64::from(max_requests))
    }
}
