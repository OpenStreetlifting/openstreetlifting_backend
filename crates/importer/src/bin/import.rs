use clap::{Parser, Subcommand};
use importer::{
    ImportContext, LiftControlCompetitionId, LiftControlRegistry,
    canonical::{
        models::CanonicalFormat,
        transformer::CanonicalTransformer,
        validator::CanonicalValidator,
    },
    sources::liftcontrol::{LiftControlClient, LiftControlExporter},
};
use sqlx::postgres::PgPoolOptions;
use std::path::PathBuf;
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
    LiftControl {
        #[command(flatten)]
        source: LiftControlSource,

        #[arg(long, default_value = "./imports")]
        output: PathBuf,
    },
    Canonical {
        file: PathBuf,

        #[arg(long)]
        validate_only: bool,
    },
    BulkImport {
        #[arg(long, default_value = "./imports")]
        directory: PathBuf,

        #[arg(long)]
        validate_only: bool,
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
        Commands::LiftControl { source, output } => {
            handle_liftcontrol_export(source, output).await?;
        }
        Commands::Canonical {
            file,
            validate_only,
        } => {
            handle_canonical_import(file, validate_only, &cli.database_url).await?;
        }
        Commands::BulkImport {
            directory,
            validate_only,
        } => {
            handle_bulk_import(directory, validate_only, &cli.database_url).await?;
        }
    }

    Ok(())
}

async fn handle_liftcontrol_export(
    source: LiftControlSource,
    output: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let registry = LiftControlRegistry::new();

    if source.list {
        list_competitions(&registry);
        return Ok(());
    }

    let comp_name = source
        .competition
        .expect("Competition name is required (enforced by clap)");

    let comp_id = parse_competition_id(&comp_name, &registry)?;
    let spec = registry
        .get_spec(comp_id)
        .ok_or_else(|| format!("Competition '{}' not found in registry", comp_id))?;

    tracing::info!(
        "Exporting LiftControl competition: {} ({} sessions)",
        spec.base_slug(),
        spec.sub_slugs().len()
    );

    export_to_canonical(&spec, output).await?;

    Ok(())
}

async fn handle_canonical_import(
    file: PathBuf,
    validate_only: bool,
    database_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Loading canonical JSON from: {}", file.display());

    let json_content = tokio::fs::read_to_string(&file).await?;
    let canonical: CanonicalFormat = serde_json::from_str(&json_content)?;

    tracing::info!(
        "Loaded competition: {} (v{})",
        canonical.competition.name,
        canonical.format_version
    );

    tracing::info!("Validating canonical format...");
    let validation_report = CanonicalValidator::validate(&canonical)?;
    validation_report.log_warnings();
    tracing::info!("✓ Validation successful!");

    if validate_only {
        return Ok(());
    }

    tracing::info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    tracing::info!("Importing {} categories to database...", canonical.categories.len());
    let transformer = CanonicalTransformer::new(&pool);
    transformer.import_to_database(canonical).await?;

    tracing::info!("✓ Import completed successfully!");

    Ok(())
}

async fn handle_bulk_import(
    directory: PathBuf,
    validate_only: bool,
    database_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Scanning directory for canonical JSON files: {}", directory.display());

    let mut json_files = Vec::new();
    let mut entries = tokio::fs::read_dir(&directory).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            let mut sub_entries = tokio::fs::read_dir(&path).await?;
            while let Some(sub_entry) = sub_entries.next_entry().await? {
                let sub_path = sub_entry.path();
                if sub_path.extension().map_or(false, |ext| ext == "json") {
                    json_files.push(sub_path);
                }
            }
        } else if path.extension().map_or(false, |ext| ext == "json") {
            json_files.push(path);
        }
    }

    if json_files.is_empty() {
        tracing::warn!("No JSON files found in {}", directory.display());
        return Ok(());
    }

    json_files.sort();
    tracing::info!("Found {} canonical JSON file(s)", json_files.len());

    let pool = if !validate_only {
        tracing::info!("Connecting to database...");
        Some(PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?)
    } else {
        None
    };

    let mut success_count = 0;
    let mut error_count = 0;

    for (idx, file_path) in json_files.iter().enumerate() {
        tracing::info!("[{}/{}] Processing: {}", idx + 1, json_files.len(), file_path.display());

        match process_canonical_file(file_path, validate_only, pool.as_ref()).await {
            Ok(_) => {
                success_count += 1;
                tracing::info!("  ✓ Success");
            }
            Err(e) => {
                error_count += 1;
                tracing::error!("  ✗ Error: {}", e);
            }
        }
    }

    tracing::info!("Summary: {} succeeded, {} failed", success_count, error_count);

    if error_count > 0 {
        return Err(format!("{} file(s) failed to import", error_count).into());
    }

    Ok(())
}

async fn process_canonical_file(
    file_path: &PathBuf,
    validate_only: bool,
    pool: Option<&sqlx::PgPool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let json_content = tokio::fs::read_to_string(file_path).await?;
    let canonical: CanonicalFormat = serde_json::from_str(&json_content)?;

    let validation_report = CanonicalValidator::validate(&canonical)?;

    if !validation_report.warnings.is_empty() {
        for warning in &validation_report.warnings {
            tracing::warn!("  {}", warning);
        }
    }

    if !validate_only {
        if let Some(pool) = pool {
            let transformer = CanonicalTransformer::new(pool);
            transformer.import_to_database(canonical).await?;
        }
    }

    Ok(())
}

fn list_competitions(registry: &LiftControlRegistry) {
    tracing::info!("Available predefined LiftControl competitions:");
    for comp_id in registry.list_competitions() {
        if let Some(config) = registry.get_config(comp_id) {
            tracing::info!("  - {} ({} sessions)", comp_id, config.sub_slugs.len());
        }
    }
}

fn parse_competition_id(
    comp_name: &str,
    registry: &LiftControlRegistry,
) -> Result<LiftControlCompetitionId, String> {
    comp_name.parse::<LiftControlCompetitionId>().map_err(|_| {
        format!(
            "Unknown competition '{}'. Use --list to see available competitions.",
            comp_name
        )
    })
}

async fn export_to_canonical(
    spec: &importer::LiftControlSpec,
    output_dir: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = LiftControlClient::new();

    for sub_slug in spec.sub_slugs() {
        let sub_slug = sub_slug.trim();
        if sub_slug.is_empty() {
            continue;
        }

        tracing::info!("Fetching data for sub-slug: {}", sub_slug);
        let api_response = client.fetch_live_general_table(sub_slug).await?;

        tracing::info!("Competition status: {}", api_response.contest.status);

        let exporter =
            LiftControlExporter::new(spec.base_slug().to_string(), spec.metadata().clone());
        let canonical = exporter.to_canonical(api_response)?;

        let competition_dir = output_dir.join(spec.base_slug());
        tokio::fs::create_dir_all(&competition_dir).await?;

        let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H-%M-%S");
        let filename = format!("{}_{}_liftcontrol.json", timestamp, sub_slug);
        let filepath = competition_dir.join(&filename);

        let json = serde_json::to_string_pretty(&canonical)?;
        tokio::fs::write(&filepath, json).await?;

        tracing::info!("Exported to: {}", filepath.display());
    }
    tracing::info!("Review and edit if needed, then import with:");
    tracing::info!("   cargo run --bin import -- canonical <path-to-json>");

    Ok(())
}
