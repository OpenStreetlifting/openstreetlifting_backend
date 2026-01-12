use anyhow::Context;
use axum::Router;
use storage::Database;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod config;
mod error;
mod features;
mod middleware;

use config::Config;
use middleware::auth::ApiKeys;

#[derive(OpenApi)]
#[openapi(
    paths(
        features::competitions::handlers::list_competitions,
        features::competitions::handlers::list_competitions_detailed,
        features::competitions::handlers::get_competition,
        features::competitions::handlers::get_competition_detailed,
        features::competitions::handlers::create_competition,
        features::competitions::handlers::update_competition,
        features::competitions::handlers::delete_competition,
        features::athletes::handlers::list_athletes,
        features::athletes::handlers::get_athlete,
        features::athletes::handlers::get_athlete_detailed,
        features::athletes::handlers::create_athlete,
        features::athletes::handlers::update_athlete,
        features::athletes::handlers::delete_athlete,
        features::ranking::handlers::get_global_ranking,
    ),
    components(
        schemas(
            storage::dto::competition::CreateCompetitionRequest,
            storage::dto::competition::UpdateCompetitionRequest,
            storage::dto::competition::CompetitionResponse,
            storage::dto::competition::CompetitionListResponse,
            storage::dto::competition::CompetitionDetailResponse,
            storage::dto::competition::CategoryDetail,
            storage::dto::competition::ParticipantDetail,
            storage::dto::competition::LiftDetail,
            storage::dto::competition::AttemptInfo,
            storage::dto::competition::FederationInfo,
            storage::dto::competition::CategoryInfo,
            storage::dto::competition::AthleteInfo,
            storage::dto::competition::MovementInfo,
            storage::dto::athlete::CreateAthleteRequest,
            storage::dto::athlete::UpdateAthleteRequest,
            storage::dto::athlete::AthleteResponse,
            storage::dto::athlete::AthleteDetailResponse,
            storage::dto::athlete::AthleteCompetitionSummary,
            storage::dto::athlete::PersonalRecord,
            storage::dto::common::PaginationMeta,
            storage::dto::ranking::GlobalRankingEntry,
            storage::dto::ranking::AthleteInfo,
            storage::dto::ranking::CompetitionInfo,
            storage::models::Competition,
            storage::models::Athlete,
            storage::models::Category,
            storage::models::Federation,
            storage::models::Movement,
            storage::models::Lift,
            storage::models::Attempt,
            storage::models::CompetitionParticipant,
            storage::models::Record,
            storage::models::Social,
            storage::models::Rulebook,
            storage::models::AthleteSocial,
        )
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

    let api_keys = ApiKeys::from_comma_separated(&config.api_keys);

    let bind_address = format!("{}:{}", config.host, config.port);
    tracing::info!("Starting server at http://{}", bind_address);

    tracing::info!(
        "Swagger UI available at http://{}/swagger-ui/",
        bind_address
    );

    let openapi = ApiDoc::openapi();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(3600));

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
        .nest(
            "/api/competitions",
            features::competitions::routes(api_keys.clone()),
        )
        .nest(
            "/api/athletes",
            features::athletes::routes(api_keys.clone()),
        )
        .nest("/api/rankings", features::ranking::routes())
        .nest("/api/ris", features::ris::routes())
        .nest("/participants", features::ris::participant_routes())
        .nest("/admin/ris", features::ris::admin_routes())
        .layer(cors)
        .with_state(db);

    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .context("Failed to bind to address")?;

    tracing::info!("Server listening on {}", bind_address);

    axum::serve(listener, app).await.context("Server error")?;

    Ok(())
}
