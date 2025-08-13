# Build stage
FROM rust:1.70 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Install build dependencies (penting untuk sqlx, openssl, dsb)
RUN apt-get update && apt-get install -y pkg-config libssl-dev libpq-dev

# Install sqlx-cli versi terbaru yang cocok
RUN cargo install sqlx-cli --version 0.7.3 --no-default-features --features postgres

# Build the application
RUN cargo build --release --verbose

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/be /app/be
COPY migrations ./migrations

EXPOSE 8080
ENV PORT=8080

CMD sqlx migrate run && ./be