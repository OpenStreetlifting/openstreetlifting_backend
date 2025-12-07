# LLM Setup for Data Extraction

## Start Ollama

```bash
docker-compose -f docker-compose.ollama.yml up -d
docker exec -it openstreetlifting_ollama ollama pull qwen2.5:7b
docker exec -it openstreetlifting_ollama ollama pull llava:7b
```

## Extract Data

**HTML:**
```bash
cargo run --bin import -- extract-html https://competition-results.com
```

**CSV:**
```bash
cargo run --bin import -- extract-csv results.csv
```

**Image (Instagram/Screenshot):**
```bash
cargo run --bin import -- extract-image photo.jpg
cargo run --bin import -- extract-image https://instagram.com/p/xyz --is-url
```

**Auto-import:**
```bash
cargo run --bin import -- extract-html https://... --auto-import
```

## Files

Extracted to: `./imports/{slug}/{timestamp}_{format}.json`

Import manually:
```bash
cargo run --bin import -- canonical ./imports/comp-name/2025-12-07_html.json
```

## Models

- `qwen2.5:7b` - HTML/CSV (recommended for M2 8GB)
- `llava:7b` - Images
- `qwen2.5:3b` - Faster/smaller
