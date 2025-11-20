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
    /// Import from LiftControl
    /// Format: base_slug:sub_slug1,sub_slug2,...
    /// Example: liftcontrol "annecy-4-lift-2025:annecy-4-lift-2025-dimanche-matin-39,annecy-4-lift-2025-dimanche-apres-midi-40"
    LiftControl {
        /// Competition identifier in format: base_slug:sub_slug1,sub_slug2,...
        identifier: String,
    },
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
        Commands::LiftControl { identifier } => {
            let importer = LiftControlImporter::new();
            importer.import(&identifier, &context).await?;
            tracing::info!("Import completed successfully!");
        }
    }

    Ok(())
}
