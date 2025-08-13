# Stage 1: Builder
FROM rust:1.82 as builder

WORKDIR /app

# Copy manifest first for caching
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true

# Copy actual source
COPY src ./src

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev libpq-dev build-essential

# Set DATABASE_URL for sqlx
# Replace with your actual Railway DB URL or use ARG + ENV
ARG DATABASE_URL
ENV DATABASE_URL=${DATABASE_URL}

# Optional: run migrations during build
# RUN cargo sqlx migrate run

# Final build
RUN cargo build --release --verbose

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates libssl3 libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/be /app/be

EXPOSE 8080
ENV PORT=8080

# Load DATABASE_URL at runtime
ENV DATABASE_URL=${DATABASE_URL}

CMD ["./be"]