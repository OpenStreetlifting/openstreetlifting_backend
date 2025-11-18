# Importer

A modular data importer system for OpenStreetlifting that supports multiple data sources.

## Architecture

The importer crate provides:

- **Trait-based design**: `CompetitionImporter` trait allows easy addition of new data sources
- **Modular sources**: Each data source is a self-contained module
- **Database integration**: Direct integration with the storage layer via SQLx

## Supported Sources

### LiftControl

Imports competition data from LiftControl API.

**Usage:**

```rust
use importer::{CompetitionImporter, ImportContext, LiftControlImporter};

let importer = LiftControlImporter::new();
let context = ImportContext { pool };

importer.import("event-slug", &context).await?;
```

## Adding New Sources

To add a new data source:

1. Create a new module in `src/sources/`
2. Implement the `CompetitionImporter` trait
3. Add transformation logic to adapt source data to storage models

**Example:**

```rust
pub struct CsvImporter {
    // ... fields
}

#[async_trait::async_trait]
impl CompetitionImporter for CsvImporter {
    async fn import(&self, identifier: &str, context: &ImportContext) -> Result<()> {
        // Implementation
        Ok(())
    }
}
```

## Running Examples

```bash
# Set DATABASE_URL
export DATABASE_URL="postgresql://user:password@localhost/openstreetlifting"

# Run LiftControl importer
cargo run --example import_liftcontrol -- event-slug-here
```

## Features

- Upsert operations (insert or update)
- Transaction support for data integrity
- Automatic athlete, category, and movement management
- Support for equipment settings and attempt tracking
