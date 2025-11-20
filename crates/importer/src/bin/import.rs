use clap::{Parser, Subcommand};
use importer::{
    CompetitionImporter, ImportContext, LiftControlCompetitionId, LiftControlImporter,
    LiftControlRegistry, LiftControlSpec,
};
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
    /// Import from LiftControl API platform
    ///
    /// Use a predefined competition by name (e.g., "annecy")
    ///
    /// Examples:
    ///   osl-import liftcontrol --competition annecy
    ///   osl-import liftcontrol --list
    LiftControl {
        #[command(flatten)]
        source: LiftControlSource,
    },
}

#[derive(clap::Args)]
#[group(required = true, multiple = false)]
struct LiftControlSource {
    #[arg(short, long)]
    competition: Option<String>,

    #[arg(short, long)]
    list: bool,
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

    match cli.command {
        Commands::LiftControl { source } => {
            handle_liftcontrol_import(source, &cli.database_url).await?;
        }
    }

    Ok(())
}

async fn handle_liftcontrol_import(
    source: LiftControlSource,
    database_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let registry = LiftControlRegistry::new();

    // Handle --list flag
    if source.list {
        tracing::info!("Available predefined LiftControl competitions:");
        for comp_id in registry.list_competitions() {
            if let Some(config) = registry.get_config(comp_id) {
                tracing::info!("  - {} ({} sessions)", comp_id, config.sub_slugs.len());
            }
        }
        return Ok(());
    }

    // Create the import spec from predefined competition
    let comp_name = source
        .competition
        .expect("Competition name is required (enforced by clap)");

    let comp_id = comp_name.parse::<LiftControlCompetitionId>().map_err(|_| {
        format!(
            "Unknown competition '{}'. Use --list to see available competitions.",
            comp_name
        )
    })?;

    let spec = registry
        .get_spec(comp_id)
        .ok_or_else(|| format!("Competition '{}' not found in registry", comp_id))?;

    tracing::info!(
        "Importing LiftControl competition: {} ({} sessions)",
        spec.base_slug(),
        spec.sub_slugs().len()
    );

    // Connect to database
    tracing::info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    let context = ImportContext { pool };

    // Execute import
    let importer = LiftControlImporter::new();
    importer.import(&spec, &context).await?;

    tracing::info!("Import completed successfully!");
    Ok(())
}
