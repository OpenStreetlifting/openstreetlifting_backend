# OpenStreetlifting Backend

## Quick Start

```bash
# Clone and setup
git clone <repository-url>
cd openstreetlifting_backend

# Configure environment
cp .env.example .env

# Launch with Docker
docker-compose up --build
```

The API will be available at `http://localhost:8080`
Swagger documentation at `http://localhost:8080/swagger-ui/`

## Development Setup

## Configuration

Configuration is managed through environment variables. See `.env.example` for all available options.

### Key Variables

| Variable       | Description                  | Default            |
| -------------- | ---------------------------- | ------------------ |
| `DATABASE_URL` | PostgreSQL connection string | See `.env.example` |
| `HOST`         | Server bind address          | `127.0.0.1`        |
| `PORT`         | Server port                  | `8080`             |
| `API_KEYS`     | Comma-separated API keys     | Optional           |

## SQLX Preparation

To work with compile time hints from SQLX, but without a live database connection, you can prepare the SQL queries with

```sh
cargo sqlx prepare --workspace
```

## API Documentation

Interactive API documentation is available via Swagger UI:

- **Local**: <http://localhost:8080/swagger-ui/>
- **Production**: <https://api.openstreetlifting.org/swagger-ui/>
