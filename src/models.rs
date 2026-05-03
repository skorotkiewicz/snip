use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use snip::config;

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub api_key: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct CreateUserResponse {
    pub id: i64,
    pub username: String,
    pub api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub username: String,
    pub api_key: String,
}

#[derive(Debug, Serialize)]
pub struct RevokeKeyResponse {
    pub username: String,
    pub old_api_key: String,
    pub new_api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct ChangePasswordResponse {
    pub username: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct Snippet {
    pub id: i64,
    pub user_id: i64,
    pub content: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub views: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SnippetWithAuthor {
    pub id: i64,
    pub content: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub created_at: DateTime<Utc>,
    pub author: String,
    pub views: i64,
    pub stars: i64,
    pub forks: i64,
    pub forked_from: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSnippetRequest {
    pub content: String,
    pub description: Option<String>,
    pub language: Option<String>,
}

impl CreateSnippetRequest {
    pub fn validate_language(lang: &Option<String>) -> Option<String> {
        match lang {
            None => Some("plaintext".to_string()),
            Some(l) => {
                let normalized = l.to_lowercase();
                if config::SUPPORTED_LANGUAGES.contains(&normalized.as_str()) {
                    Some(normalized)
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CreateSnippetResponse {
    pub id: i64,
    pub content: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
    pub q: Option<String>,
    pub lang: Option<String>,
}

fn default_page() -> i64 {
    1
}

fn default_limit() -> i64 {
    10
}

#[derive(Debug, Serialize)]
pub struct ListSnippetsResponse {
    pub snippets: Vec<SnippetWithAuthor>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
}

#[derive(Debug, Serialize)]
pub struct StarResponse {
    pub snippet_id: i64,
    pub starred: bool,
    pub total_stars: i64,
}

#[derive(Debug, Serialize)]
pub struct StarStatusResponse {
    pub snippet_id: i64,
    pub starred: bool,
    pub total_stars: i64,
}

#[derive(Debug, Serialize)]
pub struct ForkResponse {
    pub snippet_id: i64,
    pub forked_id: i64,
    pub total_forks: i64,
}
