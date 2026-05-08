use axum::{
    body::{Body, to_bytes},
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

const CACHE_EXPIRATION_SECONDS: u64 = 300; // 5 minutes

#[derive(Serialize, Deserialize)]
struct CachedResponse {
    status: u16,
    content_type: String,
    body: Vec<u8>,
}

/// Cache middleware that stores successful GET responses in Redis.
/// Only caches safe, idempotent GET requests that return 2xx status codes.
pub async fn cache_middleware(State(state): State<AppState>, req: Request, next: Next) -> Response {
    // Only cache GET requests
    if req.method() != axum::http::Method::GET {
        return next.run(req).await;
    }

    // Skip if Redis is not enabled
    if !state.redis.is_enabled() {
        return next.run(req).await;
    }

    let cache_key = format!("cache:{}", req.uri());

    // Try to get from cache
    if let Some(cached_data) = state.redis.get(&cache_key).await {
        tracing::debug!("Cache hit for {}", req.uri());
        match serde_json::from_slice::<CachedResponse>(&cached_data) {
            Ok(cached) => rebuild_response(cached, "HIT"),
            Err(e) => {
                tracing::warn!("Corrupt cache entry for {}: {}", cache_key, e);
                // Let the corrupt entry expire naturally; serve a fresh response
                execute_and_cache(&state, cache_key, req, next).await
            }
        }
    } else {
        execute_and_cache(&state, cache_key, req, next).await
    }
}

async fn execute_and_cache(
    state: &AppState,
    cache_key: String,
    req: Request,
    next: Next,
) -> Response {
    tracing::debug!("Cache miss for {}", req.uri());

    let response = next.run(req).await;

    // Only cache successful responses
    if !response.status().is_success() {
        return response;
    }

    let (parts, body) = response.into_parts();
    match to_bytes(body, usize::MAX).await {
        Ok(bytes) => {
            let body_data = bytes.to_vec();
            let content_type = parts
                .headers
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("application/json")
                .to_string();

            let cached = CachedResponse {
                status: parts.status.as_u16(),
                content_type: content_type.clone(),
                body: body_data.clone(),
            };

            if let Ok(serialized) = serde_json::to_vec(&cached) {
                state
                    .redis
                    .set_ex(&cache_key, &serialized, CACHE_EXPIRATION_SECONDS)
                    .await;
                tracing::debug!("Cached {} ({} bytes)", cache_key, body_data.len());
            }

            Response::builder()
                .status(parts.status)
                .header("X-Cache", "MISS")
                .header("Content-Type", &content_type)
                .body(Body::from(body_data))
                .unwrap_or_else(|e| {
                    tracing::error!("Failed to reconstruct response: {}", e);
                    internal_error("Cache error")
                })
        }
        Err(e) => {
            tracing::error!("Failed to read response body: {}", e);
            internal_error("Failed to process response")
        }
    }
}

fn rebuild_response(cached: CachedResponse, cache_status: &'static str) -> Response {
    Response::builder()
        .status(StatusCode::from_u16(cached.status).unwrap_or(StatusCode::OK))
        .header("X-Cache", cache_status)
        .header("Content-Type", &cached.content_type)
        .body(Body::from(cached.body))
        .unwrap_or_else(|_| internal_error("Cache reconstruction error"))
}

fn internal_error(msg: &'static str) -> Response {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(msg))
        .unwrap()
}
