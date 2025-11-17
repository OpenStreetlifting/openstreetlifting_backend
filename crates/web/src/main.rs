use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use anyhow::Context;
use storage::Database;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod config;
mod error;
mod handlers;
mod middleware;
mod routes;

use config::Config;
use middleware::auth::ApiKeys;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::competitions::list_competitions,
        handlers::competitions::get_competition,
        handlers::competitions::create_competition,
        handlers::competitions::update_competition,
        handlers::competitions::delete_competition,
    ),
    components(
        schemas(
            storage::dto::competition::CreateCompetitionRequest,
            storage::dto::competition::UpdateCompetitionRequest,
            storage::dto::competition::CompetitionResponse,
            storage::models::Competition,
            storage::models::Athlete,
            storage::models::Category,
            storage::models::Federation,
            storage::models::Movement,
            storage::models::Lift,
            storage::models::Attempt,
            storage::models::CompetitionGroup,
            storage::models::CompetitionParticipant,
            storage::models::Record,
            storage::models::Social,
            storage::models::Rulebook,
            storage::models::AthleteSocial,
        )
    ),
    tags(
        (name = "competitions", description = "Public competition endpoints"),
    ),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("API Key")
                        .build(),
                ),
            )
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    tracing::info!("Starting OpenStreetLifting API");

    let config = Config::from_env().context("Failed to load API configuration")?;
    tracing::info!("Configuration loaded successfully");

    tracing::info!(
        "Connecting to database at: {}",
        config
            .database_url
            .split('@')
            .next_back()
            .unwrap_or("unknown")
    );
    let db = Database::new(&config.database_url)
        .await
        .context("Failed to initialize database")?;
    tracing::info!("Database connection established");

    tracing::info!("Running database migrations");
    db.run_migrations()
        .await
        .context("Failed to run migrations")?;
    tracing::info!("Database migrations completed successfully");

    let db_data = web::Data::new(db);
    let api_keys = web::Data::new(ApiKeys::from_comma_separated(&config.api_keys));

    let bind_address = format!("{}:{}", config.host, config.port);
    tracing::info!("Starting server at http://{}", bind_address);

    tracing::info!(
        "Swagger UI available at http://{}/swagger-ui/",
        bind_address
    );

    let openapi = ApiDoc::openapi();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(db_data.clone())
            .app_data(api_keys.clone())
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
            .configure(routes::configure)
    })
    .bind(&bind_address)?
    .run()
    .await?;

    Ok(())
}
