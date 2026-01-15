use crate::utils::{request_id, response::error_resp};
use axum::{http::StatusCode, response::Response};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    pub fn into_response(self) -> Response {
        let req_id = request_id::generate_request_id();

        let (status, message) = match self {
            AppError::Database(err) => {
                tracing::error!(request_id = %req_id, error = ?err, "Database error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred")
            }
            AppError::Auth(ref msg) => {
                tracing::warn!(request_id = %req_id, message = %msg, "Authentication error");
                (StatusCode::UNAUTHORIZED, msg.as_str())
            }
            AppError::Validation(ref msg) => {
                tracing::warn!(request_id = %req_id, message = %msg, "Validation error");
                (StatusCode::BAD_REQUEST, msg.as_str())
            }
            AppError::NotFound(ref msg) => {
                tracing::warn!(request_id = %req_id, message = %msg, "Not found");
                (StatusCode::NOT_FOUND, msg.as_str())
            }
            AppError::Internal(err) => {
                tracing::error!(request_id = %req_id, error = ?err, "Internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        error_resp(status, req_id, message)
    }
}
