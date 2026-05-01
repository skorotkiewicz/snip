use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

#[derive(Clone)]
pub struct RedisCache {
    conn: Arc<Mutex<Option<MultiplexedConnection>>>,
    enabled: bool,
}

impl RedisCache {
    pub async fn new(redis_url: Option<String>) -> Self {
        let _enabled = redis_url.is_some();

        if let Some(url) = redis_url {
            match redis::Client::open(url.clone()) {
                Ok(client) => match client.get_multiplexed_tokio_connection().await {
                    Ok(conn) => {
                        info!("Redis connected successfully");
                        return Self {
                            conn: Arc::new(Mutex::new(Some(conn))),
                            enabled: true,
                        };
                    }
                    Err(e) => {
                        error!("Failed to connect to Redis: {}", e);
                    }
                },
                Err(e) => {
                    error!("Invalid Redis URL '{}': {}", url, e);
                }
            }
        }

        info!("Redis disabled or not configured");
        Self {
            conn: Arc::new(Mutex::new(None)),
            enabled: false,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Increment a counter and return the new value
    pub async fn incr(&self, key: &str) -> Option<i64> {
        if !self.enabled {
            return None;
        }

        let mut guard = self.conn.lock().await;
        if let Some(ref mut conn) = *guard {
            match conn.incr::<&str, i64, i64>(key, 1).await {
                Ok(val) => return Some(val),
                Err(e) => {
                    error!("Redis INCR error for key '{}': {}", key, e);
                }
            }
        }
        None
    }

    /// Check rate limit: returns true if allowed, false if rate limited
    /// Uses sliding window: max_requests per window_seconds
    pub async fn check_rate_limit(
        &self,
        key_prefix: &str,
        identifier: &str,
        max_requests: u32,
        window_seconds: u64,
    ) -> bool {
        if !self.enabled {
            return true; // Allow if Redis is disabled
        }

        let key = format!("{}:{}", key_prefix, identifier);
        let mut guard = self.conn.lock().await;

        if let Some(ref mut conn) = *guard {
            // Try to increment the counter
            match conn.incr::<&str, i32, i32>(&key, 1).await {
                Ok(count) => {
                    if count == 1 {
                        // First request in window, set expiration
                        let _: redis::RedisResult<()> =
                            conn.expire(&key, window_seconds as i64).await;
                    }

                    if count > max_requests as i32 {
                        return false; // Rate limited
                    }
                    return true; // Allowed
                }
                Err(e) => {
                    error!("Redis rate limit check error: {}", e);
                    return true; // Allow on error (fail open)
                }
            }
        }

        true
    }

    /// Flush view counters to database periodically
    /// Returns a map of snippet_id -> view_count
    pub async fn flush_view_counters(&self, prefix: &str) -> Vec<(i64, i64)> {
        if !self.enabled {
            return vec![];
        }

        // Collect keys first
        let keys: Vec<String> = {
            let mut guard = self.conn.lock().await;
            let mut keys = vec![];

            if let Some(ref mut conn) = *guard {
                let pattern = format!("{}:*", prefix);
                let mut iter: redis::AsyncIter<String> = match conn.scan_match(&pattern).await {
                    Ok(iter) => iter,
                    Err(e) => {
                        error!("Redis SCAN error: {}", e);
                        return vec![];
                    }
                };

                while let Some(key) = iter.next_item().await {
                    keys.push(key);
                }
            }
            keys
        };

        // Process keys and collect results
        let mut results = vec![];
        for key in keys {
            let snippet_id: i64 = key
                .split(':')
                .nth(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            if snippet_id > 0 {
                let mut guard = self.conn.lock().await;
                if let Some(ref mut conn) = *guard
                    && let Ok(count) = conn.get::<&str, i64>(&key).await
                    && count > 0
                {
                    results.push((snippet_id, count));
                    // Reset counter after reading
                    let _: redis::RedisResult<()> = conn.del(&key).await;
                }
            }
        }

        results
    }
}
