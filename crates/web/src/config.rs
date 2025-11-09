use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub api_keys: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            host: std::env::var("HOST").context("Cannot load HOST env variable")?,
            port: std::env::var("PORT")
                .context("PORT must be a number")?
                .parse()?,
            database_url: std::env::var("DATABASE_URL")
                .context("Cannot load DATABASE_URL env variable")?,
            api_keys: std::env::var("API_KEYS").unwrap_or_default(),
        })
    }
}
