FROM rust:1-slim-bookworm AS chef

RUN apt-get update && apt-get install -y \
  pkg-config \
  libssl-dev \
  curl \
  && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-chef

WORKDIR /app

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY crates/web/Cargo.toml crates/web/Cargo.toml
COPY crates/storage/Cargo.toml crates/storage/Cargo.toml
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --locked --recipe-path recipe.json

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release --locked --bin web

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
  ca-certificates \
  libssl3 \
  curl \
  && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1001 appuser

WORKDIR /app

COPY --from=builder /app/target/release/web /app/web

COPY --from=builder /app/crates/storage/migrations /app/migrations

RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/api-docs/openapi.json || exit 1

CMD ["/app/web"]
