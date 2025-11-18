use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
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

impl ResponseError for WebError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Storage(StorageError::NotFound) => StatusCode::NOT_FOUND,
            Self::Storage(StorageError::ConstraintViolation(_)) => StatusCode::CONFLICT,
            Self::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();

        let error_message = match self {
            Self::Storage(StorageError::NotFound) => "Resource not found".to_string(),
            Self::Storage(StorageError::ConstraintViolation(msg)) => msg.clone(),
            Self::Storage(e) => {
                tracing::error!("Storage error: {:?}", e);
                "An internal error occurred".to_string()
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
                                e.message.as_ref().map(|m| m.to_string()).unwrap_or_else(|| e.code.to_string())
                            )
                        })
                    })
                    .collect();

                return HttpResponse::build(status_code).json(json!({
                    "error": "Validation failed",
                    "details": field_errors
                }));
            }
            Self::BadRequest(msg) => msg.clone(),
            Self::Unauthorized => "Unauthorized".to_string(),
            Self::NotFound => "Resource not found".to_string(),
            Self::InternalServerError(msg) => {
                tracing::error!("Internal server error: {}", msg);
                "An internal error occurred".to_string()
            }
        };

        HttpResponse::build(status_code).json(json!({
            "error": error_message
        }))
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
