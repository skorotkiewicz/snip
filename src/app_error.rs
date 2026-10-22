use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug)]
pub struct AppError {
    pub status: StatusCode,
    message: String,
}

impl AppError {
    pub fn new(status: StatusCode, msg: impl Into<String>) -> Self {
        Self {
            status,
            message: msg.into(),
        }
    }
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, msg)
    }
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, msg)
    }
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, msg)
    }
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, msg)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        if self.status.is_server_error() {
            tracing::error!(error = %self.message, "Request failed");
            return (self.status, "Internal server error").into_response();
        }
        (self.status, self.message).into_response()
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
