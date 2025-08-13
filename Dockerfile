# Stage 1: Build
FROM rust:1.82 as builder

WORKDIR /app

# Copy only what's needed to build
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true

COPY src ./src

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev libpq-dev build-essential

# Set env for sqlx if needed
ARG DATABASE_URL
ENV DATABASE_URL=${DATABASE_URL}

# Build the binary
RUN cargo build --release

RUN cargo sqlx migrate run

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates libssl3 libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# ✅ Copy only the binary, not the whole target folder
COPY --from=builder /app/target/release/be /app/be

EXPOSE 8080
ENV PORT=8080
ENV DATABASE_URL=${DATABASE_URL}

CMD ["./be"]