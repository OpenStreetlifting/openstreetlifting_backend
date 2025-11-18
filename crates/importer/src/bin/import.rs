use clap::{Parser, Subcommand};
use importer::{CompetitionImporter, ImportContext, LiftControlImporter};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "osl-import")]
#[command(about = "OpenStreetLifting Competition Data Importer", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, env = "DATABASE_URL")]
    database_url: String,

    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    LiftControl { event_slug: String },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("import={},importer={}", log_level, log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&cli.database_url)
        .await?;

    let context = ImportContext { pool };

    match cli.command {
        Commands::LiftControl { event_slug } => {
            tracing::info!("Starting LiftControl import for event: {}", event_slug);
            let importer = LiftControlImporter::new();
            importer.import(&event_slug, &context).await?;
            tracing::info!("Import completed successfully!");
        }
    }

    Ok(())
}
