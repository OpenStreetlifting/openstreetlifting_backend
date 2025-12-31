use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::collections::HashSet;

#[derive(Clone)]
pub struct ApiKeys {
    keys: HashSet<String>,
}

impl ApiKeys {
    pub fn from_comma_separated(keys_str: &str) -> Self {
        let keys = keys_str
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        Self { keys }
    }

    pub fn is_valid(&self, key: &str) -> bool {
        self.keys.contains(key)
    }
}

pub async fn require_auth(
    State(api_keys): State<ApiKeys>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if api_keys.is_valid(token) {
        Ok(next.run(request).await)
    } else {
        tracing::warn!("Invalid API key attempt");
        Err(StatusCode::UNAUTHORIZED)
    }
}
