use axum::{
    Json,
    body::Body,
    extract::{FromRequestParts, Path, Query, State},
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use bcrypt::{DEFAULT_COST, hash, verify};
use uuid::Uuid;

use crate::{AppState, models::*};

// ==========================================
// Error Handling
// ==========================================

#[derive(Debug)]
pub struct AppError(pub StatusCode, pub String);

impl AppError {
    pub fn new(status: StatusCode, msg: impl Into<String>) -> Self {
        Self(status, msg.into())
    }
    pub fn internal(msg: impl Into<String>) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, msg.into())
    }
    pub fn bad_request(msg: &str) -> Self {
        Self(StatusCode::BAD_REQUEST, msg.to_string())
    }
    pub fn unauthorized(msg: &str) -> Self {
        Self(StatusCode::UNAUTHORIZED, msg.to_string())
    }
    pub fn not_found(msg: &str) -> Self {
        Self(StatusCode::NOT_FOUND, msg.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.0, self.1).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::internal(e.to_string())
    }
}

impl From<bcrypt::BcryptError> for AppError {
    fn from(e: bcrypt::BcryptError) -> Self {
        AppError::internal(e.to_string())
    }
}

// ==========================================
// Extractors
// ==========================================

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: i64,
    pub username: String,
    pub api_key: String,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let api_key = parts
            .headers
            .get("x-api-key")
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::unauthorized("Missing X-API-Key header"))?;

        let pool = state.db.pool();
        let user: Option<(i64, String, String)> =
            sqlx::query_as("SELECT id, username, api_key FROM users WHERE api_key = ?1")
                .bind(api_key)
                .fetch_optional(pool)
                .await?;

        user.map(|(id, username, api_key)| AuthUser {
            id,
            username,
            api_key,
        })
        .ok_or(AppError::unauthorized("Invalid API key"))
    }
}

#[derive(Debug)]
pub struct OptionalAuthUser(pub Option<AuthUser>);

impl FromRequestParts<AppState> for OptionalAuthUser {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(OptionalAuthUser(
            AuthUser::from_request_parts(parts, state).await.ok(),
        ))
    }
}

// ==========================================
// Helpers & Constants
// ==========================================

const SNIPPET_BASE_SQL: &str = r#"
    SELECT
        s.id, s.content, s.description, s.language, s.created_at, s.views,
        u.username as author,
        (SELECT COUNT(*) FROM stars WHERE snippet_id = s.id) as stars,
        s.forks, s.forked_from
    FROM snippets s
    JOIN users u ON s.user_id = u.id
"#;

async fn increment_views(state: &AppState, id: i64) -> Result<Option<i64>, AppError> {
    let pool = state.db.pool();
    if state.redis.is_enabled() {
        let key = format!("views:{}", id);
        Ok(state.redis.incr(&key).await)
    } else {
        sqlx::query("UPDATE snippets SET views = views + 1 WHERE id = ?1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(None)
    }
}

async fn get_total_stars(pool: &sqlx::SqlitePool, snippet_id: i64) -> Result<i64, AppError> {
    sqlx::query_scalar("SELECT COUNT(*) FROM stars WHERE snippet_id = ?1")
        .bind(snippet_id)
        .fetch_one(pool)
        .await
        .map_err(Into::into)
}

// ==========================================
// Handlers
// ==========================================

pub async fn register_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<CreateUserResponse>, AppError> {
    let pool = state.db.pool();

    if !state
        .redis
        .check_rate_limit("register", &req.username, 5, 3600)
        .await
    {
        return Err(AppError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded: 5 registrations per hour",
        ));
    }

    if req.username.len() < 3 || req.username.len() > 32 {
        return Err(AppError::bad_request("Username must be 3-32 characters"));
    }
    if req.password.len() < 6 {
        return Err(AppError::bad_request(
            "Password must be at least 6 characters",
        ));
    }

    let password_hash = hash(&req.password, DEFAULT_COST)?;
    let api_key = Uuid::new_v4().to_string();

    let result = sqlx::query_scalar(
        r#"INSERT INTO users (username, password_hash, api_key) VALUES (?1, ?2, ?3) RETURNING id"#,
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
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => Err(AppError::new(
            StatusCode::CONFLICT,
            "Username already exists",
        )),
        Err(e) => Err(AppError::from(e)),
    }
}

pub async fn create_snippet(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateSnippetRequest>,
) -> Result<Json<CreateSnippetResponse>, AppError> {
    let pool = state.db.pool();

    if !state
        .redis
        .check_rate_limit("snippet_create", &user.id.to_string(), 10, 60)
        .await
    {
        return Err(AppError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded: 10 snippets per minute",
        ));
    }

    if req.content.is_empty() {
        return Err(AppError::bad_request("Content cannot be empty"));
    }
    if req.content.len() > 5000 {
        return Err(AppError::bad_request(
            "Content exceeds maximum length of 5000 characters",
        ));
    }
    if req.description.as_ref().map(|d| d.len()).unwrap_or(0) > 255 {
        return Err(AppError::bad_request(
            "Description exceeds maximum length of 255 characters",
        ));
    }

    let validated_lang = CreateSnippetRequest::validate_language(&req.language).ok_or_else(|| {
        AppError::bad_request("Invalid language. Valid: plaintext, bash, c, cpp, csharp, css, go, html, java, javascript, json, kotlin, lua, markdown, php, python, ruby, rust, scala, shell, sql, swift, typescript, yaml, zig")
    })?;

    let (id, created_at): (i64, chrono::DateTime<chrono::Utc>) = sqlx::query_as(
        r#"INSERT INTO snippets (user_id, content, description, language) VALUES (?1, ?2, ?3, ?4) RETURNING id, created_at"#,
    )
    .bind(user.id)
    .bind(&req.content)
    .bind(&req.description)
    .bind(&validated_lang)
    .fetch_one(pool)
    .await?;

    Ok(Json(CreateSnippetResponse {
        id,
        content: req.content,
        description: req.description,
        language: Some(validated_lang),
        created_at,
    }))
}

pub async fn list_snippets(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<ListSnippetsResponse>, AppError> {
    let pool = state.db.pool();
    let page = query.page.max(1);
    let limit = query.limit.clamp(1, 100);
    let offset = (page - 1) * limit;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM snippets")
        .fetch_one(pool)
        .await?;

    let rows: Vec<SnippetWithAuthor> = sqlx::query_as(&format!(
        "{} ORDER BY s.created_at DESC LIMIT ?1 OFFSET ?2",
        SNIPPET_BASE_SQL
    ))
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(Json(ListSnippetsResponse {
        snippets: rows,
        total,
        page,
        limit,
    }))
}

pub async fn list_user_snippets(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<ListSnippetsResponse>, AppError> {
    let pool = state.db.pool();
    let page = query.page.max(1);
    let limit = query.limit.clamp(1, 100);
    let offset = (page - 1) * limit;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM snippets s JOIN users u ON s.user_id = u.id WHERE u.username = ?1",
    )
    .bind(&username)
    .fetch_one(pool)
    .await?;

    let rows: Vec<SnippetWithAuthor> = sqlx::query_as(&format!(
        "{} WHERE u.username = ?1 ORDER BY s.created_at DESC LIMIT ?2 OFFSET ?3",
        SNIPPET_BASE_SQL
    ))
    .bind(&username)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(Json(ListSnippetsResponse {
        snippets: rows,
        total,
        page,
        limit,
    }))
}

pub async fn get_snippet(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<SnippetWithAuthor>, AppError> {
    let pool = state.db.pool();
    let redis_count = increment_views(&state, id).await?;

    let mut snippet: SnippetWithAuthor =
        sqlx::query_as(&format!("{} WHERE s.id = ?1", SNIPPET_BASE_SQL))
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or(AppError::not_found("Snippet not found"))?;

    if let Some(redis_views) = redis_count {
        snippet.views += redis_views;
    }

    Ok(Json(snippet))
}

pub async fn get_raw_snippet(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    let pool = state.db.pool();
    increment_views(&state, id).await?;

    let content: Option<(String,)> = sqlx::query_as("SELECT content FROM snippets WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    match content {
        Some((content,)) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(Body::from(content))
            .map_err(|e| AppError::internal(e.to_string()))?),
        None => Err(AppError::not_found("Snippet not found")),
    }
}

pub async fn search_snippets(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<ListSnippetsResponse>, AppError> {
    let pool = state.db.pool();
    let page = query.page.max(1);
    let limit = query.limit.clamp(1, 100);
    let offset = (page - 1) * limit;

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
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let count_sql = format!(
        "SELECT COUNT(*) FROM snippets s JOIN users u ON s.user_id = u.id {}",
        where_clause
    );
    let mut count_query = sqlx::query_scalar(&count_sql);
    for param in &params {
        count_query = count_query.bind(param);
    }
    let total: i64 = count_query.fetch_one(pool).await?;

    let data_sql = format!(
        "{} {} ORDER BY s.created_at DESC LIMIT ? OFFSET ?",
        SNIPPET_BASE_SQL, where_clause
    );
    let mut data_query = sqlx::query_as::<_, SnippetWithAuthor>(&data_sql);
    for param in &params {
        data_query = data_query.bind(param);
    }
    let rows: Vec<SnippetWithAuthor> = data_query.bind(limit).bind(offset).fetch_all(pool).await?;

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
) -> Result<Json<LoginResponse>, AppError> {
    let pool = state.db.pool();

    if !state
        .redis
        .check_rate_limit("login", &req.username, 10, 60)
        .await
    {
        return Err(AppError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded: 10 login attempts per minute",
        ));
    }

    // Combine user fetch and API key retrieval into one query
    let user: Option<(String, String, String)> =
        sqlx::query_as("SELECT username, password_hash, api_key FROM users WHERE username = ?1")
            .bind(&req.username)
            .fetch_optional(pool)
            .await?;

    let (username, password_hash, api_key) =
        user.ok_or(AppError::unauthorized("Invalid username or password"))?;

    let valid = verify(&req.password, &password_hash)?;
    if !valid {
        return Err(AppError::unauthorized("Invalid username or password"));
    }

    Ok(Json(LoginResponse { username, api_key }))
}

pub async fn revoke_api_key(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<RevokeKeyResponse>, AppError> {
    let pool = state.db.pool();
    let new_api_key = Uuid::new_v4().to_string();

    sqlx::query("UPDATE users SET api_key = ?1 WHERE id = ?2")
        .bind(&new_api_key)
        .bind(user.id)
        .execute(pool)
        .await?;

    Ok(Json(RevokeKeyResponse {
        username: user.username,
        old_api_key: user.api_key,
        new_api_key,
    }))
}

pub async fn change_password(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<ChangePasswordResponse>, AppError> {
    let pool = state.db.pool();

    let (password_hash,): (String,) =
        sqlx::query_as("SELECT password_hash FROM users WHERE id = ?1")
            .bind(user.id)
            .fetch_one(pool)
            .await?;

    let valid = verify(&req.old_password, &password_hash)?;
    if !valid {
        return Err(AppError::unauthorized("Invalid old password"));
    }
    if req.new_password.len() < 6 {
        return Err(AppError::bad_request(
            "New password must be at least 6 characters",
        ));
    }

    let new_password_hash = hash(&req.new_password, DEFAULT_COST)?;
    sqlx::query("UPDATE users SET password_hash = ?1 WHERE id = ?2")
        .bind(&new_password_hash)
        .bind(user.id)
        .execute(pool)
        .await?;

    Ok(Json(ChangePasswordResponse {
        username: user.username,
        message: "Password changed successfully".to_string(),
    }))
}

pub async fn delete_snippet(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let pool = state.db.pool();

    let result = sqlx::query("DELETE FROM snippets WHERE id = ?1 AND user_id = ?2")
        .bind(id)
        .bind(user.id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM snippets WHERE id = ?1)")
                .bind(id)
                .fetch_one(pool)
                .await?;

        return if exists {
            Err(AppError::new(
                StatusCode::FORBIDDEN,
                "Can only delete your own snippets",
            ))
        } else {
            Err(AppError::not_found("Snippet not found"))
        };
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn star_snippet(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<Json<StarResponse>, AppError> {
    let pool = state.db.pool();

    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM snippets WHERE id = ?1)")
        .bind(id)
        .fetch_one(pool)
        .await?;
    if !exists {
        return Err(AppError::not_found("Snippet not found"));
    }

    sqlx::query("INSERT INTO stars (user_id, snippet_id) VALUES (?1, ?2) ON CONFLICT DO NOTHING")
        .bind(user.id)
        .bind(id)
        .execute(pool)
        .await?;

    let total_stars = get_total_stars(pool, id).await?;
    Ok(Json(StarResponse {
        snippet_id: id,
        starred: true,
        total_stars,
    }))
}

pub async fn unstar_snippet(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<Json<StarResponse>, AppError> {
    let pool = state.db.pool();

    sqlx::query("DELETE FROM stars WHERE user_id = ?1 AND snippet_id = ?2")
        .bind(user.id)
        .bind(id)
        .execute(pool)
        .await?;

    let total_stars = get_total_stars(pool, id).await?;
    Ok(Json(StarResponse {
        snippet_id: id,
        starred: false,
        total_stars,
    }))
}

pub async fn get_star_status(
    State(state): State<AppState>,
    opt_user: OptionalAuthUser,
    Path(id): Path<i64>,
) -> Result<Json<StarStatusResponse>, AppError> {
    let pool = state.db.pool();

    let user_starred = if let Some(u) = opt_user.0 {
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM stars WHERE user_id = ?1 AND snippet_id = ?2)",
        )
        .bind(u.id)
        .bind(id)
        .fetch_one(pool)
        .await?
    } else {
        false
    };

    let total_stars = get_total_stars(pool, id).await?;
    Ok(Json(StarStatusResponse {
        snippet_id: id,
        starred: user_starred,
        total_stars,
    }))
}

pub async fn fork_snippet(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<Json<ForkResponse>, AppError> {
    let pool = state.db.pool();

    if !state
        .redis
        .check_rate_limit("fork", &user.id.to_string(), 10, 60)
        .await
    {
        return Err(AppError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded: 10 forks per minute",
        ));
    }

    let snippet: Option<(String, Option<String>, Option<String>, i64)> = sqlx::query_as(
        "SELECT content, description, language, user_id FROM snippets WHERE id = ?1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let (content, description, language, owner_id) =
        snippet.ok_or(AppError::not_found("Snippet not found"))?;

    if owner_id == user.id {
        return Err(AppError::bad_request("Cannot fork your own snippet"));
    }

    let (new_id,): (i64,) = sqlx::query_as(
        r#"INSERT INTO snippets (user_id, content, description, language, forked_from)
        VALUES (?1, ?2, ?3, ?4, ?5) RETURNING id"#,
    )
    .bind(user.id)
    .bind(&content)
    .bind(&description)
    .bind(&language)
    .bind(id)
    .fetch_one(pool)
    .await?;

    sqlx::query("UPDATE snippets SET forks = forks + 1 WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;

    let total_forks: i64 = sqlx::query_scalar("SELECT forks FROM snippets WHERE id = ?1")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(Json(ForkResponse {
        snippet_id: id,
        forked_id: new_id,
        total_forks,
    }))
}
