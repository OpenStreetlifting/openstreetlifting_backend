# Importer

Data importer service for OpenStreetlifting that supports multiple data sources.

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

## CLI Usage

### Docker Compose

```bash
docker compose run --rm importer liftcontrol <event-slug>
```

Example:

```bash
docker compose run --rm importer liftcontrol annecy-4-lift-2025-dimanche-matin-39
```

### Environment Variables

- `DATABASE_URL`: PostgreSQL connection string (required)
- `RUST_LOG`: Override log level (optional, defaults to info)

### Running Examples

```bash
export DATABASE_URL="postgresql://user:password@localhost/openstreetlifting"
cargo run --example import_liftcontrol -- event-slug-here
```

## Features

- Upsert operations (insert or update)
- Transaction support for data integrity
- Automatic athlete, category, and movement management
- Support for equipment settings and attempt tracking
