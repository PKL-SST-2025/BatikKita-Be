# Build stage
FROM rust:1.70 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Install build dependencies (penting untuk sqlx, openssl, dsb)
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Install sqlx-cli untuk migrasi
RUN cargo install sqlx-cli --no-default-features --features postgres

# Build the application
RUN cargo build --release --verbose

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/be /app/be
COPY migrations ./migrations

# Expose port
EXPOSE 8080

ENV PORT=8080

# Jalankan migrasi sebelum menjalankan aplikasi
CMD sqlx migrate run && ./be