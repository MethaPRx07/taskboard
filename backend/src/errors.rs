use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use serde::Serialize;
use thiserror::Error;
use std::fmt;

// ─── Response body for all errors ────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub code: u16,
    pub error: String,
    pub message: String,
}

// ─── Application error enum ───────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Validation error")]
    Validation(#[from] validator::ValidationErrors),

    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("Internal server error")]
    InternalServerError,
}

// ─── Convert AppError → HTTP response ────────────────────────────────────────

impl actix_web::ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_)           => StatusCode::NOT_FOUND,
            AppError::Unauthorized(_)       => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_)          => StatusCode::FORBIDDEN,
            AppError::BadRequest(_)         => StatusCode::BAD_REQUEST,
            AppError::Conflict(_)           => StatusCode::CONFLICT,
            AppError::Validation(_)         => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Database(_)
            | AppError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();

        let error_key = match self {
            AppError::NotFound(_)           => "NOT_FOUND",
            AppError::Unauthorized(_)       => "UNAUTHORIZED",
            AppError::Forbidden(_)          => "FORBIDDEN",
            AppError::BadRequest(_)         => "BAD_REQUEST",
            AppError::Conflict(_)           => "CONFLICT",
            AppError::Validation(_)         => "VALIDATION_ERROR",
            AppError::Database(_)           => "DATABASE_ERROR",
            AppError::InternalServerError   => "INTERNAL_SERVER_ERROR",
        };

        // Don't expose internal database errors to the client
        let message: String = match self {
            AppError::Database(_) | AppError::InternalServerError => {
                "An internal error occurred".to_string()
            }
            // Use the Display impl from thiserror (fmt::Display via Error)
            other => fmt::format(format_args!("{}", other)),
        };

        HttpResponse::build(status).json(ErrorResponse {
            code: status.as_u16(),
            error: error_key.to_string(),
            message,
        })
    }
}
