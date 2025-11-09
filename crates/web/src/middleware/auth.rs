use actix_web::{dev::ServiceRequest, error::ErrorUnauthorized, Error};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use std::collections::HashSet;

pub async fn api_key_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let api_keys = req
        .app_data::<actix_web::web::Data<ApiKeys>>()
        .expect("ApiKeys not configured");

    if api_keys.is_valid(credentials.token()) {
        Ok(req)
    } else {
        tracing::warn!("Invalid API key attempt");
        Err((ErrorUnauthorized("Invalid API key"), req))
    }
}

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
