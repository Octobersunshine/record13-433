use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Invalid schedule time: {0}")]
    InvalidScheduleTime(String),

    #[error("Product not found: {0}")]
    ProductNotFound(String),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Scheduler error: {0}")]
    SchedulerError(String),

    #[error("Task already executed: {0}")]
    TaskAlreadyExecuted(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    code: u16,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::InvalidScheduleTime(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::ProductNotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::TaskNotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::TaskAlreadyExecuted(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::SchedulerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };

        let body = Json(ErrorResponse {
            code: status.as_u16(),
            message,
        });

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
