use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;
use tracing::{error, info};

#[derive(Clone)]
pub struct RedisCache {
    conn: Option<MultiplexedConnection>,
    enabled: bool,
}

impl RedisCache {
    pub async fn new(redis_url: Option<String>) -> Self {
        if let Some(url) = redis_url {
            match redis::Client::open(url.clone()) {
                Ok(client) => match client.get_multiplexed_tokio_connection().await {
                    Ok(conn) => {
                        info!("Redis connected successfully");
                        return Self {
                            conn: Some(conn),
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
            conn: None,
            enabled: false,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get a value from Redis by key
    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        if !self.enabled {
            return None;
        }

        let mut conn = self.conn.as_ref()?.clone();
        match conn.get::<&str, Vec<u8>>(key).await {
            Ok(data) if !data.is_empty() => Some(data),
            _ => None,
        }
    }

    /// Set a value in Redis with expiration (in seconds)
    pub async fn set_ex(&self, key: &str, value: &[u8], seconds: u64) -> bool {
        if !self.enabled {
            return false;
        }

        let Some(conn) = self.conn.as_ref() else {
            return false;
        };
        let mut conn = conn.clone();

        match conn.set_ex::<&str, &[u8], ()>(key, value, seconds).await {
            Ok(_) => true,
            Err(e) => {
                error!("Redis SET_EX error for key '{}': {}", key, e);
                false
            }
        }
    }

    /// Increment a counter and return the new value
    pub async fn incr(&self, key: &str) -> Option<i64> {
        if !self.enabled {
            return None;
        }

        let mut conn = self.conn.as_ref()?.clone();
        match conn.incr::<&str, i64, i64>(key, 1).await {
            Ok(val) => Some(val),
            Err(e) => {
                error!("Redis INCR error for key '{}': {}", key, e);
                None
            }
        }
    }

    /// Check rate limit: returns true if allowed, false if rate limited
    /// Uses fixed window: max_requests per window_seconds
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
        let Some(conn) = self.conn.as_ref() else {
            return true;
        };
        let mut conn = conn.clone();

        // Try to increment the counter
        match conn.incr::<&str, i32, i32>(&key, 1).await {
            Ok(count) => {
                if count == 1 {
                    // First request in window, set expiration
                    let _: redis::RedisResult<()> = conn.expire(&key, window_seconds as i64).await;
                }

                count <= max_requests as i32
            }
            Err(e) => {
                error!("Redis rate limit check error: {}", e);
                true // Allow on error (fail open)
            }
        }
    }

    /// Flush view counters to database periodically
    /// Returns a map of snippet_id -> view_count
    /// Uses GETDEL to atomically retrieve and reset the counter, preventing race conditions
    pub async fn flush_view_counters(&self, prefix: &str) -> Vec<(i64, i64)> {
        if !self.enabled {
            return vec![];
        }

        let Some(conn) = self.conn.as_ref() else {
            return vec![];
        };
        let mut scan_conn = conn.clone();
        let mut getdel_conn = conn.clone();

        let pattern = format!("{}:*", prefix);
        let mut iter: redis::AsyncIter<String> = match scan_conn.scan_match(&pattern).await {
            Ok(iter) => iter,
            Err(e) => {
                error!("Redis SCAN error: {}", e);
                return vec![];
            }
        };

        let mut results = vec![];
        while let Some(key) = iter.next_item().await {
            let snippet_id: i64 = key
                .split(':')
                .nth(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            if snippet_id > 0 {
                // Atomically get and delete the key to prevent lost increments
                let count: i64 = redis::cmd("GETDEL")
                    .arg(&key)
                    .query_async(&mut getdel_conn)
                    .await
                    .unwrap_or(0);

                if count > 0 {
                    results.push((snippet_id, count));
                }
            }
        }

        results
    }
}
