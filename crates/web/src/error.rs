use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
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

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            Self::Storage(StorageError::NotFound) => StatusCode::NOT_FOUND,
            Self::Storage(StorageError::ConstraintViolation(_)) => StatusCode::CONFLICT,
            Self::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = match &self {
            Self::Storage(StorageError::NotFound) => {
                json!({
                    "error": "Resource not found"
                })
            }
            Self::Storage(StorageError::ConstraintViolation(msg)) => {
                json!({
                    "error": msg
                })
            }
            Self::Storage(e) => {
                tracing::error!("Storage error: {:?}", e);
                json!({
                    "error": "An internal error occurred"
                })
            }
            Self::Validation(errors) => {
                let field_errors: Vec<String> = errors
                    .field_errors()
                    .iter()
                    .flat_map(|(field, errors)| {
                        errors.iter().map(move |e| {
                            format!(
                                "{}: {}",
                                field,
                                e.message
                                    .as_ref()
                                    .map(|m| m.to_string())
                                    .unwrap_or_else(|| e.code.to_string())
                            )
                        })
                    })
                    .collect();

                json!({
                    "error": "Validation failed",
                    "details": field_errors
                })
            }
            Self::BadRequest(msg) => {
                json!({
                    "error": msg
                })
            }
            Self::Unauthorized => {
                json!({
                    "error": "Unauthorized"
                })
            }
            Self::NotFound => {
                json!({
                    "error": "Resource not found"
                })
            }
            Self::InternalServerError(msg) => {
                tracing::error!("Internal server error: {}", msg);
                json!({
                    "error": "An internal error occurred"
                })
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
