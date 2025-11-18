use importer::{CompetitionImporter, ImportContext, LiftControlImporter};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/openstreetlifting".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let context = ImportContext { pool: pool.clone() };

    let importer = LiftControlImporter::new();

    let event_slug = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "annecy-4-lift-2025-dimanche-matin-39".to_string());

    println!("Importing event: {}", event_slug);

    importer.import(&event_slug, &context).await?;

    println!("Import completed successfully!");

    Ok(())
}
