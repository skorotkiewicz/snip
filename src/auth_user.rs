use axum::{extract::FromRequestParts, http::request::Parts};

use crate::{AppState, app_error::AppError};

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
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if !parts.headers.contains_key("x-api-key") {
            return Ok(Self(None));
        }
        AuthUser::from_request_parts(parts, state)
            .await
            .map(|user| Self(Some(user)))
    }
}
