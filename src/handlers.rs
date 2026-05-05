use axum::{
    Json,
    body::Body,
    extract::{FromRequestParts, Path, Query, State},
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use bcrypt::{hash, verify};
use sqlx::{QueryBuilder, Sqlite};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::{AppState, models::*};
use snip::config;

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
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self(StatusCode::BAD_REQUEST, msg.into())
    }
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self(StatusCode::UNAUTHORIZED, msg.into())
    }
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self(StatusCode::NOT_FOUND, msg.into())
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
        s.id, s.content, s.description, s.language, CAST(s.created_at AS TEXT) AS created_at, s.views,
        u.username as author,
        (SELECT COUNT(*) FROM stars WHERE snippet_id = s.id) as stars,
        s.forks, s.forked_from,
        (SELECT COUNT(*) FROM comments WHERE snippet_id = s.id) as comments
    FROM snippets s
    JOIN users u ON s.user_id = u.id
"#;

#[derive(Debug, sqlx::FromRow)]
struct CommentRow {
    id: i64,
    parent_id: Option<i64>,
    content: String,
    created_at: String,
    author: String,
    author_id: i64,
    snippet_owner_id: i64,
    likes: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Pagination {
    page: i64,
    limit: i64,
    offset: i64,
}

impl Pagination {
    fn new(page: i64, limit: i64) -> Self {
        let page = page.max(1);
        let limit = limit.clamp(1, 100);
        let offset = (page - 1) * limit;

        Self {
            page,
            limit,
            offset,
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct SearchFilters {
    where_clause: String,
    params: Vec<String>,
}

impl SearchFilters {
    fn from_query(query: &SearchQuery) -> Self {
        let mut conditions = Vec::new();
        let mut params = Vec::new();

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

        Self {
            where_clause,
            params,
        }
    }
}

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

async fn get_starred_snippet_ids(
    pool: &sqlx::SqlitePool,
    user_id: Option<i64>,
    snippet_ids: &[i64],
) -> Result<HashSet<i64>, AppError> {
    let Some(user_id) = user_id else {
        return Ok(HashSet::new());
    };
    if snippet_ids.is_empty() {
        return Ok(HashSet::new());
    }

    let mut query = QueryBuilder::<Sqlite>::new("SELECT snippet_id FROM stars WHERE user_id = ");
    query.push_bind(user_id);
    query.push(" AND snippet_id IN (");

    let mut separated = query.separated(", ");
    for snippet_id in snippet_ids {
        separated.push_bind(snippet_id);
    }
    separated.push_unseparated(")");

    let rows: Vec<(i64,)> = query.build_query_as().fetch_all(pool).await?;
    Ok(rows.into_iter().map(|(snippet_id,)| snippet_id).collect())
}

fn build_snippet_responses(
    rows: Vec<SnippetWithAuthor>,
    starred_ids: &HashSet<i64>,
) -> Vec<SnippetResponse> {
    rows.into_iter()
        .map(|row| SnippetResponse {
            starred: starred_ids.contains(&row.id),
            id: row.id,
            content: row.content,
            description: row.description,
            language: row.language,
            created_at: row.created_at,
            author: row.author,
            views: row.views,
            stars: row.stars,
            forks: row.forks,
            forked_from: row.forked_from,
            comments: row.comments,
        })
        .collect()
}

async fn get_liked_comment_ids(
    pool: &sqlx::SqlitePool,
    user_id: Option<i64>,
    comment_ids: &[i64],
) -> Result<HashSet<i64>, AppError> {
    let Some(user_id) = user_id else {
        return Ok(HashSet::new());
    };
    if comment_ids.is_empty() {
        return Ok(HashSet::new());
    }

    let mut query =
        QueryBuilder::<Sqlite>::new("SELECT comment_id FROM comment_likes WHERE user_id = ");
    query.push_bind(user_id);
    query.push(" AND comment_id IN (");

    let mut separated = query.separated(", ");
    for comment_id in comment_ids {
        separated.push_bind(comment_id);
    }
    separated.push_unseparated(")");

    let rows: Vec<(i64,)> = query.build_query_as().fetch_all(pool).await?;
    Ok(rows.into_iter().map(|(comment_id,)| comment_id).collect())
}

fn can_delete_comment(
    current_user_id: Option<i64>,
    comment_author_id: i64,
    snippet_owner_id: i64,
) -> bool {
    matches!(
        current_user_id,
        Some(user_id) if user_id == comment_author_id || user_id == snippet_owner_id
    )
}

fn build_comment_responses(
    rows: Vec<CommentRow>,
    liked_comment_ids: &HashSet<i64>,
    current_user_id: Option<i64>,
) -> Vec<CommentResponse> {
    let mut by_parent: HashMap<Option<i64>, Vec<CommentResponse>> = HashMap::new();

    for row in rows {
        by_parent
            .entry(row.parent_id)
            .or_default()
            .push(CommentResponse {
                id: row.id,
                parent_id: row.parent_id,
                content: row.content,
                created_at: row.created_at,
                author: row.author,
                likes: row.likes,
                liked: liked_comment_ids.contains(&row.id),
                can_delete: can_delete_comment(
                    current_user_id,
                    row.author_id,
                    row.snippet_owner_id,
                ),
                children: Vec::new(),
            });
    }

    fn nest_comments(
        parent_id: Option<i64>,
        by_parent: &mut HashMap<Option<i64>, Vec<CommentResponse>>,
    ) -> Vec<CommentResponse> {
        by_parent
            .remove(&parent_id)
            .unwrap_or_default()
            .into_iter()
            .map(|mut comment| {
                comment.children = nest_comments(Some(comment.id), by_parent);
                comment
            })
            .collect()
    }

    nest_comments(None, &mut by_parent)
}

async fn snippet_exists(pool: &sqlx::SqlitePool, snippet_id: i64) -> Result<bool, AppError> {
    let exists: i64 = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM snippets WHERE id = ?)")
        .bind(snippet_id)
        .fetch_one(pool)
        .await?;
    Ok(exists != 0)
}

async fn comment_exists(pool: &sqlx::SqlitePool, comment_id: i64) -> Result<bool, AppError> {
    let exists: i64 = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM comments WHERE id = ?)")
        .bind(comment_id)
        .fetch_one(pool)
        .await?;
    Ok(exists != 0)
}

async fn get_total_comment_likes(
    pool: &sqlx::SqlitePool,
    comment_id: i64,
) -> Result<i64, AppError> {
    sqlx::query_scalar("SELECT COUNT(*) FROM comment_likes WHERE comment_id = ?")
        .bind(comment_id)
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
        .check_rate_limit(
            "register",
            &req.username,
            config::rate_limit::REGISTER_MAX_REQUESTS,
            config::rate_limit::REGISTRATION_WINDOW_SECS,
        )
        .await
    {
        return Err(AppError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded: 5 registrations per hour",
        ));
    }

    if req.username.len() < config::limits::MIN_USERNAME_LENGTH
        || req.username.len() > config::limits::MAX_USERNAME_LENGTH
    {
        return Err(AppError::bad_request("Username must be 3-32 characters"));
    }
    if req.password.len() < config::limits::MIN_PASSWORD_LENGTH {
        return Err(AppError::bad_request(
            "Password must be at least 6 characters",
        ));
    }

    let password_hash = hash(&req.password, config::DEFAULT_COST)?;
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
        .check_rate_limit(
            "snippet_create",
            &user.id.to_string(),
            config::rate_limit::SNIPPET_CREATE_MAX_REQUESTS,
            config::rate_limit::DEFAULT_WINDOW_SECS,
        )
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
    if req.content.len() > config::limits::MAX_CONTENT_LENGTH {
        return Err(AppError::bad_request(
            "Content exceeds maximum length of 5000 characters",
        ));
    }
    if req.description.as_ref().map(|d| d.len()).unwrap_or(0)
        > config::limits::MAX_DESCRIPTION_LENGTH
    {
        return Err(AppError::bad_request(
            "Description exceeds maximum length of 255 characters",
        ));
    }

    let validated_lang =
        CreateSnippetRequest::validate_language(&req.language).ok_or_else(|| {
            let langs = config::SUPPORTED_LANGUAGES.join(", ");
            AppError::bad_request(format!("Invalid language. Valid: {}", langs))
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
        created_at: created_at.to_string(),
    }))
}

pub async fn list_snippets(
    State(state): State<AppState>,
    opt_user: OptionalAuthUser,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<ListSnippetsResponse>, AppError> {
    let pool = state.db.pool();
    let pagination = Pagination::new(query.page, query.limit);

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM snippets")
        .fetch_one(pool)
        .await?;

    let rows: Vec<SnippetWithAuthor> = sqlx::query_as(&format!(
        "{} ORDER BY s.created_at DESC LIMIT ?1 OFFSET ?2",
        SNIPPET_BASE_SQL
    ))
    .bind(pagination.limit)
    .bind(pagination.offset)
    .fetch_all(pool)
    .await?;
    let snippet_ids = rows.iter().map(|row| row.id).collect::<Vec<_>>();
    let starred_ids =
        get_starred_snippet_ids(pool, opt_user.0.map(|user| user.id), &snippet_ids).await?;

    Ok(Json(ListSnippetsResponse {
        snippets: build_snippet_responses(rows, &starred_ids),
        total,
        page: pagination.page,
        limit: pagination.limit,
    }))
}

pub async fn list_user_snippets(
    State(state): State<AppState>,
    opt_user: OptionalAuthUser,
    Path(username): Path<String>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<ListSnippetsResponse>, AppError> {
    let pool = state.db.pool();
    let pagination = Pagination::new(query.page, query.limit);

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
    .bind(pagination.limit)
    .bind(pagination.offset)
    .fetch_all(pool)
    .await?;
    let snippet_ids = rows.iter().map(|row| row.id).collect::<Vec<_>>();
    let starred_ids =
        get_starred_snippet_ids(pool, opt_user.0.map(|user| user.id), &snippet_ids).await?;

    Ok(Json(ListSnippetsResponse {
        snippets: build_snippet_responses(rows, &starred_ids),
        total,
        page: pagination.page,
        limit: pagination.limit,
    }))
}

pub async fn get_snippet(
    State(state): State<AppState>,
    opt_user: OptionalAuthUser,
    Path(id): Path<i64>,
) -> Result<Json<SnippetResponse>, AppError> {
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

    let starred_ids =
        get_starred_snippet_ids(pool, opt_user.0.map(|user| user.id), &[snippet.id]).await?;
    let response = build_snippet_responses(vec![snippet], &starred_ids)
        .into_iter()
        .next()
        .ok_or_else(|| AppError::internal("Failed to build snippet response"))?;

    Ok(Json(response))
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
    opt_user: OptionalAuthUser,
    Query(query): Query<SearchQuery>,
) -> Result<Json<ListSnippetsResponse>, AppError> {
    let pool = state.db.pool();
    let pagination = Pagination::new(query.page, query.limit);
    let filters = SearchFilters::from_query(&query);

    let count_sql = format!(
        "SELECT COUNT(*) FROM snippets s JOIN users u ON s.user_id = u.id {}",
        filters.where_clause
    );
    let mut count_query = sqlx::query_scalar(&count_sql);
    for param in &filters.params {
        count_query = count_query.bind(param);
    }
    let total: i64 = count_query.fetch_one(pool).await?;

    let data_sql = format!(
        "{} {} ORDER BY s.created_at DESC LIMIT ? OFFSET ?",
        SNIPPET_BASE_SQL, filters.where_clause
    );
    let mut data_query = sqlx::query_as::<_, SnippetWithAuthor>(&data_sql);
    for param in &filters.params {
        data_query = data_query.bind(param);
    }
    let rows: Vec<SnippetWithAuthor> = data_query
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(pool)
        .await?;
    let snippet_ids = rows.iter().map(|row| row.id).collect::<Vec<_>>();
    let starred_ids =
        get_starred_snippet_ids(pool, opt_user.0.map(|user| user.id), &snippet_ids).await?;

    Ok(Json(ListSnippetsResponse {
        snippets: build_snippet_responses(rows, &starred_ids),
        total,
        page: pagination.page,
        limit: pagination.limit,
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let pool = state.db.pool();

    if !state
        .redis
        .check_rate_limit(
            "login",
            &req.username,
            config::rate_limit::LOGIN_MAX_REQUESTS,
            config::rate_limit::DEFAULT_WINDOW_SECS,
        )
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
    if req.new_password.len() < config::limits::MIN_PASSWORD_LENGTH {
        return Err(AppError::bad_request(
            "New password must be at least 6 characters",
        ));
    }

    let new_password_hash = hash(&req.new_password, config::DEFAULT_COST)?;
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
        let exists = snippet_exists(pool, id).await?;

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

    if !snippet_exists(pool, id).await? {
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
        .check_rate_limit(
            "fork",
            &user.id.to_string(),
            config::rate_limit::FORK_MAX_REQUESTS,
            config::rate_limit::DEFAULT_WINDOW_SECS,
        )
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

pub async fn list_comments(
    State(state): State<AppState>,
    opt_user: OptionalAuthUser,
    Path(snippet_id): Path<i64>,
) -> Result<Json<ListCommentsResponse>, AppError> {
    let pool = state.db.pool();
    let current_user_id = opt_user.0.as_ref().map(|user| user.id);

    if !snippet_exists(pool, snippet_id).await? {
        return Err(AppError::not_found("Snippet not found"));
    }

    let rows: Vec<CommentRow> = sqlx::query_as(
        r#"
        SELECT
            c.id,
            c.parent_id,
            c.content,
            CAST(c.created_at AS TEXT) AS created_at,
            u.username AS author,
            c.user_id AS author_id,
            s.user_id AS snippet_owner_id,
            (SELECT COUNT(*) FROM comment_likes cl WHERE cl.comment_id = c.id) AS likes
        FROM comments c
        JOIN users u ON c.user_id = u.id
        JOIN snippets s ON c.snippet_id = s.id
        WHERE c.snippet_id = ?
        ORDER BY c.created_at ASC, c.id ASC
        "#,
    )
    .bind(snippet_id)
    .fetch_all(pool)
    .await?;

    let comment_ids = rows.iter().map(|row| row.id).collect::<Vec<_>>();
    let liked_comment_ids = get_liked_comment_ids(pool, current_user_id, &comment_ids).await?;

    Ok(Json(ListCommentsResponse {
        comments: build_comment_responses(rows, &liked_comment_ids, current_user_id),
    }))
}

pub async fn create_comment(
    State(state): State<AppState>,
    user: AuthUser,
    Path(snippet_id): Path<i64>,
    Json(req): Json<CreateCommentRequest>,
) -> Result<Json<CreateCommentResponse>, AppError> {
    let pool = state.db.pool();
    let content = req.content.trim();

    if !state
        .redis
        .check_rate_limit(
            "comment_create",
            &user.id.to_string(),
            config::rate_limit::COMMENT_CREATE_MAX_REQUESTS,
            config::rate_limit::DEFAULT_WINDOW_SECS,
        )
        .await
    {
        return Err(AppError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded: 30 comments per minute",
        ));
    }

    if content.is_empty() {
        return Err(AppError::bad_request("Comment cannot be empty"));
    }
    if content.len() > config::limits::MAX_COMMENT_LENGTH {
        return Err(AppError::bad_request(
            "Comment exceeds maximum length of 1000 characters",
        ));
    }
    if !snippet_exists(pool, snippet_id).await? {
        return Err(AppError::not_found("Snippet not found"));
    }

    if let Some(parent_id) = req.parent_id {
        let parent_snippet_id: Option<i64> =
            sqlx::query_scalar("SELECT snippet_id FROM comments WHERE id = ?")
                .bind(parent_id)
                .fetch_optional(pool)
                .await?;

        match parent_snippet_id {
            Some(id) if id == snippet_id => {}
            Some(_) => {
                return Err(AppError::bad_request(
                    "Parent comment belongs to a different snippet",
                ));
            }
            None => return Err(AppError::not_found("Parent comment not found")),
        }
    }

    let (id, created_at): (i64, String) = sqlx::query_as(
        "INSERT INTO comments (snippet_id, user_id, parent_id, content) VALUES (?, ?, ?, ?) RETURNING id, CAST(created_at AS TEXT)",
    )
        .bind(snippet_id)
        .bind(user.id)
        .bind(req.parent_id)
        .bind(content)
        .fetch_one(pool)
        .await?;

    Ok(Json(CreateCommentResponse {
        id,
        snippet_id,
        parent_id: req.parent_id,
        created_at,
    }))
}

pub async fn delete_comment(
    State(state): State<AppState>,
    user: AuthUser,
    Path(comment_id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let pool = state.db.pool();

    let owner_info: Option<(i64, i64)> = sqlx::query_as(
        r#"
        SELECT c.user_id, s.user_id
        FROM comments c
        JOIN snippets s ON c.snippet_id = s.id
        WHERE c.id = ?
        "#,
    )
    .bind(comment_id)
    .fetch_optional(pool)
    .await?;

    let Some((comment_author_id, snippet_owner_id)) = owner_info else {
        return Err(AppError::not_found("Comment not found"));
    };

    if !can_delete_comment(Some(user.id), comment_author_id, snippet_owner_id) {
        return Err(AppError::new(
            StatusCode::FORBIDDEN,
            "Can only delete your own comments or comments on your snippets",
        ));
    }

    sqlx::query("DELETE FROM comments WHERE id = ?")
        .bind(comment_id)
        .execute(pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn like_comment(
    State(state): State<AppState>,
    user: AuthUser,
    Path(comment_id): Path<i64>,
) -> Result<Json<CommentLikeResponse>, AppError> {
    let pool = state.db.pool();

    if !comment_exists(pool, comment_id).await? {
        return Err(AppError::not_found("Comment not found"));
    }

    sqlx::query(
        "INSERT INTO comment_likes (user_id, comment_id) VALUES (?, ?) ON CONFLICT DO NOTHING",
    )
    .bind(user.id)
    .bind(comment_id)
    .execute(pool)
    .await?;

    let total_likes = get_total_comment_likes(pool, comment_id).await?;
    Ok(Json(CommentLikeResponse {
        comment_id,
        liked: true,
        total_likes,
    }))
}

pub async fn unlike_comment(
    State(state): State<AppState>,
    user: AuthUser,
    Path(comment_id): Path<i64>,
) -> Result<Json<CommentLikeResponse>, AppError> {
    let pool = state.db.pool();

    sqlx::query("DELETE FROM comment_likes WHERE user_id = ? AND comment_id = ?")
        .bind(user.id)
        .bind(comment_id)
        .execute(pool)
        .await?;

    let total_likes = get_total_comment_likes(pool, comment_id).await?;
    Ok(Json(CommentLikeResponse {
        comment_id,
        liked: false,
        total_likes,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn pagination_normalizes_page_and_limit() {
        assert_eq!(
            Pagination::new(0, 500),
            Pagination {
                page: 1,
                limit: 100,
                offset: 0,
            }
        );
        assert_eq!(
            Pagination::new(3, -5),
            Pagination {
                page: 3,
                limit: 1,
                offset: 2,
            }
        );
    }

    #[test]
    fn search_filters_empty_when_no_query_provided() {
        let query = SearchQuery {
            page: 1,
            limit: 10,
            q: None,
            lang: None,
        };

        assert_eq!(SearchFilters::from_query(&query), SearchFilters::default());
    }

    #[test]
    fn search_filters_include_content_and_language_conditions() {
        let query = SearchQuery {
            page: 1,
            limit: 10,
            q: Some("main".to_string()),
            lang: Some("Rust".to_string()),
        };

        assert_eq!(
            SearchFilters::from_query(&query),
            SearchFilters {
                where_clause: "WHERE (s.content LIKE ? OR s.description LIKE ?) AND s.language = ?"
                    .to_string(),
                params: vec![
                    "%main%".to_string(),
                    "%main%".to_string(),
                    "rust".to_string(),
                ],
            }
        );
    }

    #[test]
    fn build_snippet_responses_marks_matching_snippets_as_starred() {
        let created_at = "2026-05-04T00:00:00Z".to_string();
        let rows = vec![
            SnippetWithAuthor {
                id: 10,
                content: "fn main() {}".to_string(),
                description: Some("first".to_string()),
                language: Some("rust".to_string()),
                created_at: created_at.clone(),
                author: "alice".to_string(),
                views: 1,
                stars: 3,
                forks: 0,
                forked_from: None,
                comments: 5,
            },
            SnippetWithAuthor {
                id: 11,
                content: "puts 1".to_string(),
                description: Some("second".to_string()),
                language: Some("ruby".to_string()),
                created_at,
                author: "bob".to_string(),
                views: 2,
                stars: 4,
                forks: 1,
                forked_from: Some(9),
                comments: 3,
            },
        ];
        let starred_ids = HashSet::from([11]);

        let responses = build_snippet_responses(rows, &starred_ids);

        assert_eq!(responses.len(), 2);
        assert!(!responses[0].starred);
        assert!(responses[1].starred);
        assert_eq!(responses[1].forked_from, Some(9));
    }

    #[test]
    fn build_snippet_responses_defaults_to_unstarred_without_matches() {
        let rows = vec![SnippetWithAuthor {
            id: 42,
            content: "echo hi".to_string(),
            description: None,
            language: Some("bash".to_string()),
            created_at: "2026-05-04T00:00:00Z".to_string(),
            author: "carol".to_string(),
            views: 0,
            stars: 0,
            forks: 0,
            forked_from: None,
            comments: 0,
        }];

        let responses = build_snippet_responses(rows, &HashSet::new());

        assert_eq!(responses.len(), 1);
        assert!(!responses[0].starred);
        assert_eq!(responses[0].author, "carol");
    }

    #[test]
    fn can_delete_comment_allows_author_or_snippet_owner() {
        assert!(can_delete_comment(Some(7), 7, 9));
        assert!(can_delete_comment(Some(9), 7, 9));
        assert!(!can_delete_comment(Some(3), 7, 9));
        assert!(!can_delete_comment(None, 7, 9));
    }

    #[test]
    fn build_comment_responses_nests_children_and_marks_likes() {
        let created_at = "2026-05-04T00:00:00Z".to_string();
        let rows = vec![
            CommentRow {
                id: 1,
                parent_id: None,
                content: "root".to_string(),
                created_at: created_at.clone(),
                author: "alice".to_string(),
                author_id: 10,
                snippet_owner_id: 99,
                likes: 2,
            },
            CommentRow {
                id: 2,
                parent_id: Some(1),
                content: "child".to_string(),
                created_at: created_at.clone(),
                author: "bob".to_string(),
                author_id: 20,
                snippet_owner_id: 99,
                likes: 1,
            },
            CommentRow {
                id: 3,
                parent_id: Some(2),
                content: "grandchild".to_string(),
                created_at,
                author: "carol".to_string(),
                author_id: 30,
                snippet_owner_id: 99,
                likes: 0,
            },
        ];

        let liked_ids = HashSet::from([2]);
        let comments = build_comment_responses(rows, &liked_ids, Some(99));

        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].id, 1);
        assert_eq!(comments[0].children.len(), 1);
        assert_eq!(comments[0].children[0].id, 2);
        assert!(comments[0].children[0].liked);
        assert!(comments[0].children[0].can_delete);
        assert_eq!(comments[0].children[0].children.len(), 1);
        assert_eq!(comments[0].children[0].children[0].id, 3);
    }
}
