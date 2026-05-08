use std::time::Duration;

use crate::{config, db::Database, redis_cache::RedisCache};

/// Spawn a background task that periodically flushes view counters from Redis to the database.
pub fn spawn_view_counter_flush(db: Database, redis: RedisCache) {
    if !redis.is_enabled() {
        return;
    }

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(
            config::rate_limit::VIEW_COUNTER_FLUSH_INTERVAL_SECS,
        ));
        loop {
            interval.tick().await;
            let counters = redis.flush_view_counters("views").await;
            if !counters.is_empty() {
                tracing::info!("Flushing {} view counters to database", counters.len());
                for (snippet_id, count) in counters {
                    if let Err(e) =
                        sqlx::query("UPDATE snippets SET views = views + ?1 WHERE id = ?2")
                            .bind(count)
                            .bind(snippet_id)
                            .execute(db.pool())
                            .await
                    {
                        tracing::error!("Failed to update views for snippet {}: {}", snippet_id, e);
                    }
                }
            }
        }
    });
}
