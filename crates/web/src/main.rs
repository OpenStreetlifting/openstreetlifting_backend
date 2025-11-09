use std::path::PathBuf;

use actix_web::{App, HttpServer, web};
use anyhow::Context;
use storage::Database;

mod config;
mod error;
mod handlers;
mod routes;

use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .join(".env");

    dotenvy::from_path(&workspace_root)?;

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = Config::from_env().context("Failed to load API configuration")?;

    let db = Database::new(&config.database_url)
        .await
        .context("Failed to initialize database")?;

    db.run_migrations()
        .await
        .context("Failed to run migrations")?;

    let db_data = web::Data::new(db);

    let bind_address = format!("{}:{}", config.host, config.port);
    tracing::info!("Starting server at http://{}", bind_address);

    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .configure(routes::configure)
    })
    .bind(&bind_address)?
    .run()
    .await?;

    Ok(())
}
