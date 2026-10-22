use axum::{
    Json,
    body::Body,
    extract::{ConnectInfo, Path, Query, State},
    http::StatusCode,
    response::Response,
};
use bcrypt::{non_truncating_hash, verify};
use sqlx::{QueryBuilder, Sqlite};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::{
    AppState,
    app_error::AppError,
    auth_user::{AuthUser, OptionalAuthUser},
    models::{
        ChangePasswordRequest, ChangePasswordResponse, CommentLikeResponse, CommentResponse,
        CreateCommentRequest, CreateCommentResponse, CreateSnippetRequest, CreateSnippetResponse,
        CreateUserRequest, CreateUserResponse, ForkResponse, ListCommentsResponse,
        ListSnippetsResponse, LoginRequest, LoginResponse, PaginationQuery, RevokeKeyResponse,
        SearchQuery, SnippetResponse, SnippetWithAuthor, StarResponse, StarStatusResponse,
    },
};
use snip::config;
use std::net::SocketAddr;

// ==========================================

const SNIPPET_BASE_SQL: &str = r#"
    SELECT
        s.id, s.content, s.description, s.language,
        strftime('%Y-%m-%dT%H:%M:%SZ', s.created_at) AS created_at, s.views,
        u.username AS author,
        (SELECT COUNT(*) FROM stars WHERE snippet_id = s.id) AS stars,
        (SELECT COUNT(*) FROM snippets WHERE forked_from = s.id) AS forks,
        s.forked_from,
        (SELECT COUNT(*) FROM comments WHERE snippet_id = s.id) AS comments
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
        let offset = (page - 1).saturating_mul(limit);

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

fn validate_password(password: &str) -> Result<(), AppError> {
    if password.chars().count() < config::MIN_PASSWORD_LENGTH {
        return Err(AppError::bad_request(
            "Password must be at least 6 characters",
        ));
    }
    if password.len() > config::MAX_PASSWORD_BYTES {
        return Err(AppError::bad_request(
            "Password must be at most 72 UTF-8 bytes",
        ));
    }
    Ok(())
}

async fn hash_password(password: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || non_truncating_hash(password, config::DEFAULT_COST))
        .await
        .map_err(|error| AppError::internal(error.to_string()))?
        .map_err(Into::into)
}

async fn verify_password(password: String, password_hash: String) -> Result<bool, AppError> {
    if password.len() > config::MAX_PASSWORD_BYTES {
        return Ok(false);
    }
    tokio::task::spawn_blocking(move || verify(password, &password_hash))
        .await
        .map_err(|error| AppError::internal(error.to_string()))?
        .map_err(Into::into)
}

async fn increment_views(state: &AppState, id: i64) -> Result<i64, AppError> {
    sqlx::query_scalar("UPDATE snippets SET views = views + 1 WHERE id = ? RETURNING views")
        .bind(id)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| AppError::not_found("Snippet not found"))
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
    snippet_id: i64,
) -> Result<HashSet<i64>, AppError> {
    let Some(user_id) = user_id else {
        return Ok(HashSet::new());
    };
    let rows: Vec<i64> = sqlx::query_scalar(
        "SELECT cl.comment_id FROM comment_likes cl
         JOIN comments c ON c.id = cl.comment_id WHERE cl.user_id = ? AND c.snippet_id = ?",
    )
    .bind(user_id)
    .bind(snippet_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().collect())
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
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<CreateUserResponse>, AppError> {
    let pool = state.db.pool();

    if !state
        .db
        .check_rate_limit(
            "register",
            &peer.ip().to_string(),
            config::REGISTER_MAX_REQUESTS,
            config::REGISTRATION_WINDOW_SECS,
        )
        .await?
    {
        return Err(AppError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded: 5 registrations per hour",
        ));
    }

    if req.username.len() < config::MIN_USERNAME_LENGTH
        || req.username.len() > config::MAX_USERNAME_LENGTH
    {
        return Err(AppError::bad_request("Username must be 3-32 characters"));
    }
    if !req
        .username
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(AppError::bad_request(
            "Username must contain only ASCII letters, digits, underscores, or hyphens",
        ));
    }
    validate_password(&req.password)?;
    let password_hash = hash_password(req.password).await?;
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
        .db
        .check_rate_limit(
            "snippet_create",
            &user.id.to_string(),
            config::SNIPPET_CREATE_MAX_REQUESTS,
            config::DEFAULT_WINDOW_SECS,
        )
        .await?
    {
        return Err(AppError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded: 10 snippets per minute",
        ));
    }

    if req.content.is_empty() {
        return Err(AppError::bad_request("Content cannot be empty"));
    }
    if req.content.chars().count() > config::MAX_CONTENT_LENGTH {
        return Err(AppError::bad_request(
            "Content exceeds maximum length of 5000 characters",
        ));
    }
    if req
        .description
        .as_ref()
        .map(|d| d.chars().count())
        .unwrap_or(0)
        > config::MAX_DESCRIPTION_LENGTH
    {
        return Err(AppError::bad_request(
            "Description exceeds maximum length of 255 characters",
        ));
    }

    let validated_lang = CreateSnippetRequest::validate_language(req.language.as_deref())
        .ok_or_else(|| {
            let langs = config::SUPPORTED_LANGUAGES.join(", ");
            AppError::bad_request(format!("Invalid language. Valid: {}", langs))
        })?;

    let (id, created_at): (i64, String) = sqlx::query_as(
        r#"INSERT INTO snippets (user_id, content, description, language) VALUES (?1, ?2, ?3, ?4) RETURNING id, strftime('%Y-%m-%dT%H:%M:%SZ', created_at)"#,
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
    opt_user: OptionalAuthUser,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<ListSnippetsResponse>, AppError> {
    let pool = state.db.pool();
    let pagination = Pagination::new(query.page, query.limit);

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM snippets")
        .fetch_one(pool)
        .await?;

    let rows: Vec<SnippetWithAuthor> = sqlx::query_as(&format!(
        "{} ORDER BY s.created_at DESC, s.id DESC LIMIT ?1 OFFSET ?2",
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
        "{} WHERE u.username = ?1 ORDER BY s.created_at DESC, s.id DESC LIMIT ?2 OFFSET ?3",
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

    let mut snippet: SnippetWithAuthor =
        sqlx::query_as(&format!("{} WHERE s.id = ?1", SNIPPET_BASE_SQL))
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or(AppError::not_found("Snippet not found"))?;

    snippet.views = increment_views(&state, id).await?;

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

    let content: Option<(String,)> = sqlx::query_as("SELECT content FROM snippets WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    match content {
        Some((content,)) => {
            increment_views(&state, id).await?;
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain; charset=utf-8")
                .body(Body::from(content))
                .map_err(|e| AppError::internal(e.to_string()))?)
        }
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
        "{} {} ORDER BY s.created_at DESC, s.id DESC LIMIT ? OFFSET ?",
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
        .db
        .check_rate_limit(
            "login",
            &req.username,
            config::LOGIN_MAX_REQUESTS,
            config::DEFAULT_WINDOW_SECS,
        )
        .await?
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

    let valid = verify_password(req.password, password_hash).await?;
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

    if !state
        .db
        .check_rate_limit(
            "password_change",
            &user.id.to_string(),
            config::LOGIN_MAX_REQUESTS,
            config::DEFAULT_WINDOW_SECS,
        )
        .await?
    {
        return Err(AppError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many password change attempts",
        ));
    }
    let (password_hash,): (String,) =
        sqlx::query_as("SELECT password_hash FROM users WHERE id = ?1")
            .bind(user.id)
            .fetch_one(pool)
            .await?;

    let valid = verify_password(req.old_password, password_hash).await?;
    if !valid {
        return Err(AppError::unauthorized("Invalid old password"));
    }
    validate_password(&req.new_password)?;
    let new_password_hash = hash_password(req.new_password).await?;
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

    if !snippet_exists(pool, id).await? {
        return Err(AppError::not_found("Snippet not found"));
    }

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

    if !snippet_exists(pool, id).await? {
        return Err(AppError::not_found("Snippet not found"));
    }

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
        .db
        .check_rate_limit(
            "fork",
            &user.id.to_string(),
            config::FORK_MAX_REQUESTS,
            config::DEFAULT_WINDOW_SECS,
        )
        .await?
    {
        return Err(AppError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded: 10 forks per minute",
        ));
    }

    let new_id: Option<i64> = sqlx::query_scalar(
        "INSERT INTO snippets (user_id, content, description, language, forked_from)
         SELECT ?, content, description, language, id FROM snippets WHERE id = ? AND user_id != ?
         RETURNING id",
    )
    .bind(user.id)
    .bind(id)
    .bind(user.id)
    .fetch_optional(pool)
    .await?;
    let Some(new_id) = new_id else {
        return Err(if snippet_exists(pool, id).await? {
            AppError::bad_request("Cannot fork your own snippet")
        } else {
            AppError::not_found("Snippet not found")
        });
    };
    let total_forks = sqlx::query_scalar("SELECT COUNT(*) FROM snippets WHERE forked_from = ?")
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
            strftime('%Y-%m-%dT%H:%M:%SZ', c.created_at) AS created_at,
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

    let liked_comment_ids = get_liked_comment_ids(pool, current_user_id, snippet_id).await?;

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
        .db
        .check_rate_limit(
            "comment_create",
            &user.id.to_string(),
            config::COMMENT_CREATE_MAX_REQUESTS,
            config::DEFAULT_WINDOW_SECS,
        )
        .await?
    {
        return Err(AppError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded: 30 comments per minute",
        ));
    }

    if content.is_empty() {
        return Err(AppError::bad_request("Comment cannot be empty"));
    }
    if content.chars().count() > config::MAX_COMMENT_LENGTH {
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

    if let Some(parent_id) = req.parent_id {
        let depth: i64 = sqlx::query_scalar(
            "WITH RECURSIVE ancestors(id, parent_id, depth) AS (
                 SELECT id, parent_id, 1 FROM comments WHERE id = ?
                 UNION ALL
                 SELECT c.id, c.parent_id, a.depth + 1 FROM comments c
                 JOIN ancestors a ON c.id = a.parent_id WHERE a.depth < ?
             ) SELECT COALESCE(MAX(depth), 0) FROM ancestors",
        )
        .bind(parent_id)
        .bind(config::MAX_COMMENT_DEPTH)
        .fetch_one(pool)
        .await?;
        if depth >= config::MAX_COMMENT_DEPTH {
            return Err(AppError::bad_request(
                "Comments support at most 8 nested levels",
            ));
        }
    }

    let (id, created_at): (i64, String) = sqlx::query_as(
        "INSERT INTO comments (snippet_id, user_id, parent_id, content) VALUES (?, ?, ?, ?) RETURNING id, strftime('%Y-%m-%dT%H:%M:%SZ', created_at)",
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

    if !comment_exists(pool, comment_id).await? {
        return Err(AppError::not_found("Comment not found"));
    }

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
    use crate::database::Database;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::collections::HashSet;

    async fn test_state() -> AppState {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("open isolated test database");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate isolated test database");

        AppState {
            db: Database::new(pool),
        }
    }

    fn test_user() -> AuthUser {
        AuthUser {
            id: 1,
            username: "alice".to_string(),
            api_key: "key".to_string(),
        }
    }

    #[tokio::test]
    async fn missing_star_and_like_targets_return_not_found() {
        let state = test_state().await;
        let user = test_user();

        let err = unstar_snippet(State(state.clone()), user.clone(), Path(404))
            .await
            .expect_err("missing target must return an error");
        assert_eq!(err.status, StatusCode::NOT_FOUND);

        let err = get_star_status(State(state.clone()), OptionalAuthUser(None), Path(404))
            .await
            .expect_err("missing target must return an error");
        assert_eq!(err.status, StatusCode::NOT_FOUND);

        let err = unlike_comment(State(state), user, Path(404))
            .await
            .expect_err("missing target must return an error");
        assert_eq!(err.status, StatusCode::NOT_FOUND);
    }

    async fn seed_snippet(state: &AppState) {
        sqlx::query("INSERT INTO users (id, username, password_hash, api_key) VALUES (1, 'alice', '', 'key'), (2, 'bob', '', 'bob-key')")
            .execute(state.db.pool()).await.expect("insert test users");
        sqlx::query("INSERT INTO snippets (id, user_id, content, language) VALUES (1, 1, 'hello', 'plaintext')")
            .execute(state.db.pool()).await.expect("insert test snippet");
    }

    #[test]
    fn pagination_accepts_largest_page_without_overflow() {
        let pagination = Pagination::new(i64::MAX, 100);
        assert!(
            pagination.offset >= 0,
            "offset must stay in SQLite's integer range"
        );
    }

    #[tokio::test]
    async fn deleting_a_fork_updates_the_original_count() {
        let state = test_state().await;
        seed_snippet(&state).await;
        let bob = AuthUser {
            id: 2,
            username: "bob".into(),
            api_key: "bob-key".into(),
        };
        let Json(fork) = fork_snippet(State(state.clone()), bob.clone(), Path(1))
            .await
            .expect("fork succeeds");
        assert_eq!(fork.total_forks, 1);
        delete_snippet(State(state.clone()), bob, Path(fork.forked_id))
            .await
            .expect("delete fork");
        let Json(original) = get_snippet(State(state), OptionalAuthUser(None), Path(1))
            .await
            .expect("read original");
        assert_eq!(original.forks, 0, "deleted forks must not count");
    }

    #[tokio::test]
    async fn snippet_timestamps_are_rfc3339() {
        let state = test_state().await;
        seed_snippet(&state).await;
        let Json(snippet) = get_snippet(State(state), OptionalAuthUser(None), Path(1))
            .await
            .expect("read snippet");
        assert!(
            chrono::DateTime::parse_from_rfc3339(&snippet.created_at).is_ok(),
            "API timestamps must parse in the CLI: {}",
            snippet.created_at
        );
    }

    #[tokio::test]
    async fn unicode_snippet_limit_counts_characters() {
        let state = test_state().await;
        seed_snippet(&state).await;
        let result = create_snippet(
            State(state),
            test_user(),
            Json(CreateSnippetRequest {
                content: "界".repeat(5000),
                description: Some("界".repeat(255)),
                language: None,
            }),
        )
        .await;
        assert!(
            result.is_ok(),
            "documented character limits must accept Unicode: {result:?}"
        );
    }

    #[tokio::test]
    async fn passwords_over_bcrypt_limit_are_rejected() {
        let state = test_state().await;
        let result = register_user(
            State(state),
            ConnectInfo("127.0.0.1:1234".parse().expect("valid test address")),
            Json(CreateUserRequest {
                username: "alice".into(),
                password: "a".repeat(73),
            }),
        )
        .await;
        assert_eq!(
            result
                .expect_err("overlong password must be rejected")
                .status,
            StatusCode::BAD_REQUEST
        );
    }

    async fn json_body(response: axum::response::Response) -> serde_json::Value {
        let bytes = axum::body::to_bytes(response.into_body(), 100_000)
            .await
            .expect("read JSON response");
        serde_json::from_slice(&bytes).expect("decode JSON response")
    }

    #[tokio::test]
    async fn http_authentication_password_changes_and_key_rotation_work() {
        let state = test_state().await;
        let response = http_request(
            state.clone(),
            "POST",
            "/api/register",
            serde_json::json!({"username": "alice", "password": " secret123 "}),
            None,
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()["cache-control"], "no-store");
        let account = json_body(response).await;
        let key = account["api_key"].as_str().expect("API key returned");
        let response = http_request(
            state.clone(),
            "POST",
            "/api/login",
            serde_json::json!({"username": "alice", "password": " secret123 "}),
            None,
        )
        .await;
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "password whitespace must be preserved"
        );
        let response = http_request(
            state.clone(),
            "POST",
            "/api/change-password",
            serde_json::json!({"old_password": " secret123 ", "new_password": "１２３４５６"}),
            Some(key),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        let response = http_request(
            state.clone(),
            "POST",
            "/api/login",
            serde_json::json!({"username": "alice", "password": " secret123 "}),
            None,
        )
        .await;
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "old password must stop working"
        );
        let response = http_request(
            state.clone(),
            "POST",
            "/api/login",
            serde_json::json!({"username": "alice", "password": "１２３４５６"}),
            None,
        )
        .await;
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Unicode passwords must work"
        );
        let response = http_request(
            state.clone(),
            "POST",
            "/api/revoke-key",
            serde_json::Value::Null,
            Some(key),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        let rotated = json_body(response).await;
        let new_key = rotated["new_api_key"]
            .as_str()
            .expect("new API key returned");
        assert_ne!(key, new_key);
        let response = http_request(
            state.clone(),
            "GET",
            "/api/snippets",
            serde_json::Value::Null,
            Some(key),
        )
        .await;
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "old API key must stop working"
        );
        let response = http_request(
            state,
            "GET",
            "/api/snippets",
            serde_json::Value::Null,
            Some(new_key),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn rate_limits_are_atomic_and_reset_after_expiry() {
        let state = test_state().await;
        let mut attempts = tokio::task::JoinSet::new();
        for _ in 0..12 {
            let db = state.db.clone();
            attempts.spawn(async move {
                db.check_rate_limit("test", "alice", 5, 60)
                    .await
                    .expect("count attempt")
            });
        }
        let mut allowed = 0;
        while let Some(result) = attempts.join_next().await {
            allowed += usize::from(result.expect("rate limit task finishes"));
        }
        assert_eq!(allowed, 5, "concurrent requests share one counter");
        let reopened = Database::new(state.db.pool().clone());
        assert!(
            !reopened
                .check_rate_limit("test", "alice", 5, 60)
                .await
                .expect("read persisted limit"),
            "creating a new handle must not reset the limit"
        );
        sqlx::query("UPDATE rate_limits SET expires_at = unixepoch() - 1")
            .execute(state.db.pool())
            .await
            .expect("expire test window");
        assert!(
            reopened
                .check_rate_limit("test", "alice", 5, 60)
                .await
                .expect("begin new window"),
            "expired windows must reset"
        );
    }

    async fn http_request(
        state: AppState,
        method: &str,
        uri: &str,
        body: serde_json::Value,
        key: Option<&str>,
    ) -> axum::response::Response {
        use tower::ServiceExt;
        let mut builder = axum::http::Request::builder()
            .method(method)
            .uri(uri)
            .extension(ConnectInfo(
                "127.0.0.1:1234"
                    .parse::<SocketAddr>()
                    .expect("test peer address"),
            ))
            .header("Content-Type", "application/json");
        if let Some(key) = key {
            builder = builder.header("X-API-Key", key);
        }
        crate::server::router(state)
            .oneshot(
                builder
                    .body(Body::from(body.to_string()))
                    .expect("build test request"),
            )
            .await
            .expect("router returns a response")
    }

    #[tokio::test]
    async fn http_rate_limits_apply_without_external_services() {
        let state = test_state().await;
        for index in 0..6 {
            let response = http_request(state.clone(), "POST", "/api/register",
                serde_json::json!({"username": format!("bad user {index}"), "password": "secret123"}), None).await;
            assert_eq!(
                response.status(),
                if index < 5 {
                    StatusCode::BAD_REQUEST
                } else {
                    StatusCode::TOO_MANY_REQUESTS
                },
                "registration limits must apply by peer IP across usernames"
            );
        }
        for index in 0..11 {
            let response = http_request(
                state.clone(),
                "POST",
                "/api/login",
                serde_json::json!({"username": "missing", "password": "secret123"}),
                None,
            )
            .await;
            assert_eq!(
                response.status(),
                if index < 10 {
                    StatusCode::UNAUTHORIZED
                } else {
                    StatusCode::TOO_MANY_REQUESTS
                },
                "login limits must apply without Redis"
            );
        }
    }

    #[tokio::test]
    async fn http_reads_are_fresh_and_never_cache_credentials() {
        let state = test_state().await;
        seed_snippet(&state).await;
        let response = http_request(
            state.clone(),
            "GET",
            "/api/snippets?page=9223372036854775807&limit=100",
            serde_json::Value::Null,
            None,
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()["cache-control"], "no-store");
        assert_eq!(response.headers()["x-content-type-options"], "nosniff");
        let response = http_request(
            state.clone(),
            "GET",
            "/api/snippets",
            serde_json::Value::Null,
            Some("bad-key"),
        )
        .await;
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "invalid authentication must not silently become a guest request"
        );
        let response = http_request(
            state.clone(),
            "DELETE",
            "/api/snippets/1",
            serde_json::Value::Null,
            Some("key"),
        )
        .await;
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let response =
            http_request(state, "GET", "/api/snippets", serde_json::Value::Null, None).await;
        let bytes = axum::body::to_bytes(response.into_body(), 100_000)
            .await
            .expect("read response");
        let body: serde_json::Value = serde_json::from_slice(&bytes).expect("decode snippets");
        assert_eq!(
            body["total"], 0,
            "deleted snippets must disappear immediately"
        );
        assert_eq!(body["snippets"], serde_json::json!([]));
    }

    #[tokio::test]
    async fn comment_depth_is_bounded_and_parent_snippets_are_checked() {
        let state = test_state().await;
        seed_snippet(&state).await;
        let mut parent_id = None;
        for _ in 0..config::MAX_COMMENT_DEPTH {
            let Json(comment) = create_comment(
                State(state.clone()),
                test_user(),
                Path(1),
                Json(CreateCommentRequest {
                    content: "界".repeat(1000),
                    parent_id,
                }),
            )
            .await
            .expect("accept documented depth and Unicode length");
            parent_id = Some(comment.id);
            assert!(
                chrono::DateTime::parse_from_rfc3339(&comment.created_at).is_ok(),
                "comment timestamps must use RFC 3339"
            );
        }
        let error = create_comment(
            State(state.clone()),
            test_user(),
            Path(1),
            Json(CreateCommentRequest {
                content: "too deep".into(),
                parent_id,
            }),
        )
        .await
        .expect_err("reject excess depth");
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        sqlx::query("INSERT INTO snippets (id, user_id, content) VALUES (2, 1, 'other')")
            .execute(state.db.pool())
            .await
            .expect("create second snippet");
        let error = create_comment(
            State(state),
            test_user(),
            Path(2),
            Json(CreateCommentRequest {
                content: "wrong parent".into(),
                parent_id,
            }),
        )
        .await
        .expect_err("reject cross-snippet reply");
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn views_are_durable_and_deleted_origins_detach_forks() {
        let state = test_state().await;
        seed_snippet(&state).await;
        for views in 1..=3 {
            let Json(snippet) = get_snippet(State(state.clone()), OptionalAuthUser(None), Path(1))
                .await
                .expect("read snippet");
            assert_eq!(snippet.views, views);
        }
        get_raw_snippet(State(state.clone()), Path(1))
            .await
            .expect("read raw snippet");
        let views: i64 = sqlx::query_scalar("SELECT views FROM snippets WHERE id = 1")
            .fetch_one(state.db.pool())
            .await
            .expect("read durable counter");
        assert_eq!(views, 4);
        let bob = AuthUser {
            id: 2,
            username: "bob".into(),
            api_key: "bob-key".into(),
        };
        let Json(fork) = fork_snippet(State(state.clone()), bob, Path(1))
            .await
            .expect("fork original");
        delete_snippet(State(state.clone()), test_user(), Path(1))
            .await
            .expect("delete original");
        let Json(copy) = get_snippet(State(state), OptionalAuthUser(None), Path(fork.forked_id))
            .await
            .expect("fork survives original deletion");
        assert_eq!(
            copy.forked_from, None,
            "forks must not point at deleted originals"
        );
    }

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
