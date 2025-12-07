use crate::handle_canonical_import;
use importer::extractors::{CsvExtractor, HtmlExtractor, ImageExtractor};
use std::path::PathBuf;

pub async fn handle_html_extract(
    url: String,
    output: PathBuf,
    ollama_url: String,
    model: String,
    auto_import: bool,
    database_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Extracting HTML from: {}", url);

    let extractor = HtmlExtractor::new(ollama_url, model);
    let filepath = extractor.extract_and_save(&url, &output).await?;

    tracing::info!("✓ Extracted to: {}", filepath.display());

    if auto_import {
        tracing::info!("Auto-importing to database...");
        handle_canonical_import(filepath, false, database_url).await?;
    } else {
        tracing::info!("Review and import with:");
        tracing::info!(
            "   cargo run --bin import -- canonical {}",
            filepath.display()
        );
    }

    Ok(())
}

pub async fn handle_csv_extract(
    file: PathBuf,
    output: PathBuf,
    ollama_url: String,
    model: String,
    auto_import: bool,
    database_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Extracting CSV from: {}", file.display());

    let extractor = CsvExtractor::new(ollama_url, model);
    let filepath = extractor
        .extract_and_save(file.to_str().unwrap(), &output)
        .await?;

    tracing::info!("✓ Extracted to: {}", filepath.display());

    if auto_import {
        tracing::info!("Auto-importing to database...");
        handle_canonical_import(filepath, false, database_url).await?;
    } else {
        tracing::info!("Review and import with:");
        tracing::info!(
            "   cargo run --bin import -- canonical {}",
            filepath.display()
        );
    }

    Ok(())
}

pub async fn handle_image_extract(
    source: String,
    is_url: bool,
    output: PathBuf,
    ollama_url: String,
    model: String,
    auto_import: bool,
    database_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if is_url {
        tracing::info!("Extracting image from URL: {}", source);
    } else {
        tracing::info!("Extracting image from file: {}", source);
    }

    let extractor = ImageExtractor::new(ollama_url, model);
    let filepath = extractor.extract_and_save(&source, &output, is_url).await?;

    tracing::info!("✓ Extracted to: {}", filepath.display());

    if auto_import {
        tracing::info!("Auto-importing to database...");
        handle_canonical_import(filepath, false, database_url).await?;
    } else {
        tracing::info!("Review and import with:");
        tracing::info!(
            "   cargo run --bin import -- canonical {}",
            filepath.display()
        );
    }

    Ok(())
}
