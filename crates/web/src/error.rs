use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};
use std::fmt;
use storage::error::StorageError;
use validator::ValidationErrors;

/// Web layer errors
#[derive(Debug)]
pub enum WebError {
    Storage(StorageError),
    Validation(ValidationErrors),
    BadRequest(String),
    #[allow(dead_code)]
    Unauthorized,
    #[allow(dead_code)]
    NotFound,
    #[allow(dead_code)]
    InternalServerError(String),
}

impl fmt::Display for WebError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(e) => write!(f, "Storage error: {}", e),
            Self::Validation(e) => write!(f, "Validation error: {}", e),
            Self::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            Self::Unauthorized => write!(f, "Unauthorized"),
            Self::NotFound => write!(f, "Resource not found"),
            Self::InternalServerError(msg) => write!(f, "Internal server error: {}", msg),
        }
    }
}

fn error_json(message: &str) -> Value {
    json!({ "error": message })
}

fn internal_error_json() -> Value {
    json!({ "error": "An internal error occurred" })
}

fn format_validation_errors(errors: &ValidationErrors) -> Value {
    let field_errors: Vec<String> = errors
        .field_errors()
        .iter()
        .flat_map(|(field, errors)| {
            errors.iter().map(move |e| {
                let message = e
                    .message
                    .as_ref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| e.code.to_string());
                format!("{}: {}", field, message)
            })
        })
        .collect();

    json!({
        "error": "Validation failed",
        "details": field_errors
    })
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        let (status_code, body) = match &self {
            Self::Storage(StorageError::NotFound) | Self::NotFound => {
                (StatusCode::NOT_FOUND, error_json("Resource not found"))
            }
            Self::Storage(StorageError::ConstraintViolation(msg)) => {
                (StatusCode::CONFLICT, error_json(msg))
            }
            Self::Storage(e) => {
                tracing::error!("Storage error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, internal_error_json())
            }
            Self::Validation(errors) => {
                (StatusCode::BAD_REQUEST, format_validation_errors(errors))
            }
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, error_json(msg)),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, error_json("Unauthorized")),
            Self::InternalServerError(msg) => {
                tracing::error!("Internal server error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, internal_error_json())
            }
        };

        (status_code, Json(body)).into_response()
    }
}

impl From<StorageError> for WebError {
    fn from(error: StorageError) -> Self {
        Self::Storage(error)
    }
}

impl From<ValidationErrors> for WebError {
    fn from(error: ValidationErrors) -> Self {
        Self::Validation(error)
    }
}

pub type WebResult<T> = Result<T, WebError>;
pub type ApiResult<T> = Result<T, WebError>;
