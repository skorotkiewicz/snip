use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
};
use bcrypt::{DEFAULT_COST, hash, verify};
use uuid::Uuid;

use crate::{AppState, models::*};

pub async fn register_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<CreateUserResponse>, (StatusCode, String)> {
    let pool = state.db.pool();

    // Rate limiting: 5 registrations per hour per IP
    // Using username as identifier since we don't have direct IP access in this context
    let allowed = state
        .redis
        .check_rate_limit("register", &req.username, 5, 3600)
        .await;

    if !allowed {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded: 5 registrations per hour".to_string(),
        ));
    }

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

    // Rate limiting: 10 snippets per minute per user
    let allowed = state
        .redis
        .check_rate_limit("snippet_create", &user_id.to_string(), 10, 60)
        .await;

    if !allowed {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded: 10 snippets per minute".to_string(),
        ));
    }

    if req.content.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Content cannot be empty".to_string(),
        ));
    }

    if req.content.len() > 5000 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Content exceeds maximum length of 5000 characters".to_string(),
        ));
    }

    if req.description.as_ref().map(|d| d.len()).unwrap_or(0) > 255 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Description exceeds maximum length of 255 characters".to_string(),
        ));
    }

    // Validate language
    let validated_lang = CreateSnippetRequest::validate_language(&req.language)
        .ok_or((
            StatusCode::BAD_REQUEST,
            "Invalid language. Valid: plaintext, bash, c, cpp, csharp, css, go, html, java, javascript, json, kotlin, lua, markdown, php, python, ruby, rust, scala, shell, sql, swift, typescript, yaml, zig".to_string(),
        ))?;

    let result: Result<(i64, chrono::DateTime<chrono::Utc>), sqlx::Error> = sqlx::query_as(
        r#"
        INSERT INTO snippets (user_id, content, description, language)
        VALUES (?1, ?2, ?3, ?4)
        RETURNING id, created_at
        "#,
    )
    .bind(user_id)
    .bind(&req.content)
    .bind(&req.description)
    .bind(&validated_lang)
    .fetch_one(pool)
    .await;

    match result {
        Ok((id, created_at)) => Ok(Json(CreateSnippetResponse {
            id,
            content: req.content,
            description: req.description,
            language: Some(validated_lang),
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
            s.language,
            s.created_at,
            s.views,
            u.username as author,
            (SELECT COUNT(*) FROM stars WHERE snippet_id = s.id) as stars
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

pub async fn list_user_snippets(
    State(state): State<AppState>,
    axum::extract::Path(username): axum::extract::Path<String>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<ListSnippetsResponse>, (StatusCode, String)> {
    let pool = state.db.pool();

    let page = query.page.max(1);
    let limit = query.limit.clamp(1, 100);
    let offset = (page - 1) * limit;

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM snippets s
        JOIN users u ON s.user_id = u.id
        WHERE u.username = ?1
        "#,
    )
    .bind(&username)
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
            s.language,
            s.created_at,
            s.views,
            u.username as author,
            (SELECT COUNT(*) FROM stars WHERE snippet_id = s.id) as stars
        FROM snippets s
        JOIN users u ON s.user_id = u.id
        WHERE u.username = ?1
        ORDER BY s.created_at DESC
        LIMIT ?2 OFFSET ?3
        "#,
    )
    .bind(&username)
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

pub async fn get_snippet(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<Json<SnippetWithAuthor>, (StatusCode, String)> {
    let pool = state.db.pool();

    // Increment views using Redis if available, otherwise fallback to database
    let redis_count = if state.redis.is_enabled() {
        let key = format!("views:{}", id);
        state.redis.incr(&key).await
    } else {
        // Fallback: increment directly in database
        sqlx::query("UPDATE snippets SET views = views + 1 WHERE id = ?1")
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                )
            })?;
        None
    };

    let snippet: Option<SnippetWithAuthor> = sqlx::query_as(
        r#"
        SELECT
            s.id,
            s.content,
            s.description,
            s.language,
            s.created_at,
            s.views,
            u.username as author,
            (SELECT COUNT(*) FROM stars WHERE snippet_id = s.id) as stars
        FROM snippets s
        JOIN users u ON s.user_id = u.id
        WHERE s.id = ?1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    match snippet {
        Some(mut s) => {
            // If we have a Redis count, add it to the database count
            if let Some(redis_views) = redis_count {
                s.views += redis_views;
            }
            Ok(Json(s))
        }
        None => Err((StatusCode::NOT_FOUND, "Snippet not found".to_string())),
    }
}

pub async fn get_raw_snippet(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<axum::response::Response, (StatusCode, String)> {
    let pool = state.db.pool();

    // Increment views using Redis if available, otherwise fallback to database
    if state.redis.is_enabled() {
        let key = format!("views:{}", id);
        let _: Option<i64> = state.redis.incr(&key).await;
    } else {
        // Fallback: increment directly in database
        sqlx::query("UPDATE snippets SET views = views + 1 WHERE id = ?1")
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                )
            })?;
    }

    let content: Option<(String,)> = sqlx::query_as("SELECT content FROM snippets WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    match content {
        Some((content,)) => {
            let response = axum::response::Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain; charset=utf-8")
                .body(axum::body::Body::from(content))
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Response error: {}", e),
                    )
                })?;
            Ok(response)
        }
        None => Err((StatusCode::NOT_FOUND, "Snippet not found".to_string())),
    }
}

pub async fn search_snippets(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<ListSnippetsResponse>, (StatusCode, String)> {
    let pool = state.db.pool();

    let page = query.page.max(1);
    let limit = query.limit.clamp(1, 100);
    let offset = (page - 1) * limit;

    // Build the WHERE clause based on search parameters
    let mut conditions = Vec::new();
    let mut params: Vec<String> = Vec::new();

    if let Some(search_term) = &query.q
        && !search_term.is_empty()
    {
        let pattern = format!("%{}%", search_term);
        conditions.push("(s.content LIKE ? OR s.description LIKE ?)".to_string());
        params.push(pattern.clone());
        params.push(pattern);
    }

    if let Some(lang) = &query.lang
        && !lang.is_empty()
        && lang != "all"
    {
        conditions.push("s.language = ?".to_string());
        params.push(lang.to_lowercase());
    }

    let where_clause = if conditions.is_empty() {
        ""
    } else {
        &format!("WHERE {}", conditions.join(" AND "))
    };

    // Build count query
    let count_sql = format!(
        "SELECT COUNT(*) FROM snippets s JOIN users u ON s.user_id = u.id {}",
        where_clause
    );

    let mut count_query = sqlx::query_scalar(&count_sql);
    for param in &params {
        count_query = count_query.bind(param);
    }

    let total: i64 = count_query.fetch_one(pool).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    // Build data query
    let data_sql = format!(
        r#"
        SELECT
            s.id,
            s.content,
            s.description,
            s.language,
            s.created_at,
            s.views,
            u.username as author,
            (SELECT COUNT(*) FROM stars WHERE snippet_id = s.id) as stars
        FROM snippets s
        JOIN users u ON s.user_id = u.id
        {}
        ORDER BY s.created_at DESC
        LIMIT ? OFFSET ?
        "#,
        where_clause
    );

    let mut data_query = sqlx::query_as::<_, SnippetWithAuthor>(&data_sql);
    for param in &params {
        data_query = data_query.bind(param);
    }
    let rows: Vec<SnippetWithAuthor> = data_query
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

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    let pool = state.db.pool();

    // Rate limiting: 10 login attempts per minute per username
    let allowed = state
        .redis
        .check_rate_limit("login", &req.username, 10, 60)
        .await;

    if !allowed {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded: 10 login attempts per minute".to_string(),
        ));
    }

    let user: Option<(String, String)> =
        sqlx::query_as("SELECT username, password_hash FROM users WHERE username = ?1")
            .bind(&req.username)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                )
            })?;

    let (username, password_hash) = user.ok_or((
        StatusCode::UNAUTHORIZED,
        "Invalid username or password".to_string(),
    ))?;

    let valid = verify(&req.password, &password_hash).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Password verification error: {}", e),
        )
    })?;

    if !valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Invalid username or password".to_string(),
        ));
    }

    let api_key: String = sqlx::query_scalar("SELECT api_key FROM users WHERE username = ?1")
        .bind(&username)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    Ok(Json(LoginResponse { username, api_key }))
}

pub async fn revoke_api_key(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<RevokeKeyResponse>, (StatusCode, String)> {
    let pool = state.db.pool();

    let old_api_key = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Missing X-API-Key header".to_string(),
        ))?;

    let user: Option<(String, String)> =
        sqlx::query_as("SELECT username, api_key FROM users WHERE api_key = ?1")
            .bind(old_api_key)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                )
            })?;

    let (username, old_key) =
        user.ok_or((StatusCode::UNAUTHORIZED, "Invalid API key".to_string()))?;

    let new_api_key = Uuid::new_v4().to_string();

    sqlx::query("UPDATE users SET api_key = ?1 WHERE username = ?2")
        .bind(&new_api_key)
        .bind(&username)
        .execute(pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    Ok(Json(RevokeKeyResponse {
        username,
        old_api_key: old_key,
        new_api_key,
    }))
}

pub async fn change_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<ChangePasswordResponse>, (StatusCode, String)> {
    let pool = state.db.pool();

    let api_key = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Missing X-API-Key header".to_string(),
        ))?;

    // Get user info
    let user: Option<(String, String)> =
        sqlx::query_as("SELECT username, password_hash FROM users WHERE api_key = ?1")
            .bind(api_key)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                )
            })?;

    let (username, password_hash) =
        user.ok_or((StatusCode::UNAUTHORIZED, "Invalid API key".to_string()))?;

    // Verify old password
    let valid = verify(&req.old_password, &password_hash).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Password verification error: {}", e),
        )
    })?;

    if !valid {
        return Err((StatusCode::UNAUTHORIZED, "Invalid old password".to_string()));
    }

    // Validate new password
    if req.new_password.len() < 6 {
        return Err((
            StatusCode::BAD_REQUEST,
            "New password must be at least 6 characters".to_string(),
        ));
    }

    // Hash new password
    let new_password_hash = hash(&req.new_password, DEFAULT_COST).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Hash error: {}", e),
        )
    })?;

    // Update password
    sqlx::query("UPDATE users SET password_hash = ?1 WHERE username = ?2")
        .bind(&new_password_hash)
        .bind(&username)
        .execute(pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    Ok(Json(ChangePasswordResponse {
        username,
        message: "Password changed successfully".to_string(),
    }))
}

pub async fn delete_snippet(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<StatusCode, (StatusCode, String)> {
    let pool = state.db.pool();

    let api_key = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Missing X-API-Key header".to_string(),
        ))?;

    // Get user id from API key
    let user: Option<(i64,)> = sqlx::query_as("SELECT id FROM users WHERE api_key = ?1")
        .bind(api_key)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    let (user_id,) = user.ok_or((StatusCode::UNAUTHORIZED, "Invalid API key".to_string()))?;

    // Check if snippet exists and belongs to user
    let snippet: Option<(i64,)> = sqlx::query_as("SELECT user_id FROM snippets WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    if let Some((snippet_user_id,)) = snippet {
        if snippet_user_id != user_id {
            return Err((
                StatusCode::FORBIDDEN,
                "Can only delete your own snippets".to_string(),
            ));
        }
    } else {
        return Err((StatusCode::NOT_FOUND, "Snippet not found".to_string()));
    }

    // Delete the snippet
    sqlx::query("DELETE FROM snippets WHERE id = ?1 AND user_id = ?2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn star_snippet(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<Json<StarResponse>, (StatusCode, String)> {
    let pool = state.db.pool();

    let api_key = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Missing X-API-Key header".to_string(),
        ))?;

    let user: Option<(i64,)> = sqlx::query_as("SELECT id FROM users WHERE api_key = ?1")
        .bind(api_key)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    let (user_id,) = user.ok_or((StatusCode::UNAUTHORIZED, "Invalid API key".to_string()))?;

    // Check if snippet exists
    let snippet_exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM snippets WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    snippet_exists.ok_or((StatusCode::NOT_FOUND, "Snippet not found".to_string()))?;

    // Add star
    let result = sqlx::query(
        "INSERT INTO stars (user_id, snippet_id) VALUES (?1, ?2) ON CONFLICT DO NOTHING",
    )
    .bind(user_id)
    .bind(id)
    .execute(pool)
    .await;

    let _was_added = match result {
        Ok(res) => res.rows_affected() > 0,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            ));
        }
    };

    // Get total stars
    let total_stars: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM stars WHERE snippet_id = ?1")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    Ok(Json(StarResponse {
        snippet_id: id,
        starred: true,
        total_stars,
    }))
}

pub async fn unstar_snippet(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<Json<StarResponse>, (StatusCode, String)> {
    let pool = state.db.pool();

    let api_key = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Missing X-API-Key header".to_string(),
        ))?;

    let user: Option<(i64,)> = sqlx::query_as("SELECT id FROM users WHERE api_key = ?1")
        .bind(api_key)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    let (user_id,) = user.ok_or((StatusCode::UNAUTHORIZED, "Invalid API key".to_string()))?;

    // Remove star
    sqlx::query("DELETE FROM stars WHERE user_id = ?1 AND snippet_id = ?2")
        .bind(user_id)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    // Get total stars
    let total_stars: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM stars WHERE snippet_id = ?1")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    Ok(Json(StarResponse {
        snippet_id: id,
        starred: false,
        total_stars,
    }))
}

pub async fn get_star_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<Json<StarStatusResponse>, (StatusCode, String)> {
    let pool = state.db.pool();

    let api_key_opt = headers.get("x-api-key").and_then(|v| v.to_str().ok());

    let mut user_starred = false;

    // Check if user has starred (if API key provided)
    if let Some(api_key) = api_key_opt {
        let starred: Option<(i64,)> = sqlx::query_as(
            "SELECT 1 FROM stars s JOIN users u ON s.user_id = u.id WHERE u.api_key = ?1 AND s.snippet_id = ?2"
        )
        .bind(api_key)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;
        user_starred = starred.is_some();
    }

    // Get total stars
    let total_stars: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM stars WHERE snippet_id = ?1")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    Ok(Json(StarStatusResponse {
        snippet_id: id,
        starred: user_starred,
        total_stars,
    }))
}
