use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct Snippet {
    pub id: i64,
    pub user_id: i64,
    pub content: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SnippetWithAuthor {
    pub id: i64,
    pub content: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub author: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateSnippetRequest {
    pub content: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateSnippetResponse {
    pub id: i64,
    pub content: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
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
