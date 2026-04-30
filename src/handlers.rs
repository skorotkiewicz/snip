use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
};
use bcrypt::{DEFAULT_COST, hash};
use uuid::Uuid;

use crate::{AppState, models::*};

pub async fn register_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<CreateUserResponse>, (StatusCode, String)> {
    let pool = state.db.pool();

    if req.username.len() < 3 || req.username.len() > 32 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Username must be 3-32 characters".to_string(),
        ));
    }

    if req.password.len() < 6 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Password must be at least 6 characters".to_string(),
        ));
    }

    let password_hash = hash(&req.password, DEFAULT_COST).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Hash error: {}", e),
        )
    })?;

    let api_key = Uuid::new_v4().to_string();

    let result: Result<i64, sqlx::Error> = sqlx::query_scalar(
        r#"
        INSERT INTO users (username, password_hash, api_key)
        VALUES (?1, ?2, ?3)
        RETURNING id
        "#,
    )
    .bind(&req.username)
    .bind(&password_hash)
    .bind(&api_key)
    .fetch_one(pool)
    .await;

    match result {
        Ok(id) => Ok(Json(CreateUserResponse {
            id,
            username: req.username,
            api_key,
        })),
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
            Err((StatusCode::CONFLICT, "Username already exists".to_string()))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )),
    }
}

pub async fn create_snippet(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateSnippetRequest>,
) -> Result<Json<CreateSnippetResponse>, (StatusCode, String)> {
    let pool = state.db.pool();

    let api_key = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Missing X-API-Key header".to_string(),
        ))?;

    let user_row: Option<(i64,)> = sqlx::query_as("SELECT id FROM users WHERE api_key = ?1")
        .bind(api_key)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    let user_id = user_row
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid API key".to_string()))?
        .0;

    if req.content.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Content cannot be empty".to_string(),
        ));
    }

    let result: Result<(i64, chrono::DateTime<chrono::Utc>), sqlx::Error> = sqlx::query_as(
        r#"
        INSERT INTO snippets (user_id, content, description)
        VALUES (?1, ?2, ?3)
        RETURNING id, created_at
        "#,
    )
    .bind(user_id)
    .bind(&req.content)
    .bind(&req.description)
    .fetch_one(pool)
    .await;

    match result {
        Ok((id, created_at)) => Ok(Json(CreateSnippetResponse {
            id,
            content: req.content,
            description: req.description,
            created_at,
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )),
    }
}

pub async fn list_snippets(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<ListSnippetsResponse>, (StatusCode, String)> {
    let pool = state.db.pool();

    let page = query.page.max(1);
    let limit = query.limit.clamp(1, 100);
    let offset = (page - 1) * limit;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM snippets")
        .fetch_one(pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    let rows: Vec<SnippetWithAuthor> = sqlx::query_as(
        r#"
        SELECT 
            s.id,
            s.content,
            s.description,
            s.created_at,
            u.username as author
        FROM snippets s
        JOIN users u ON s.user_id = u.id
        ORDER BY s.created_at DESC
        LIMIT ?1 OFFSET ?2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(Json(ListSnippetsResponse {
        snippets: rows,
        total,
        page,
        limit,
    }))
}
