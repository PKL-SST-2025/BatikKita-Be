# Stage 1: Builder
FROM rust:1.82 as builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true

COPY src ./src

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev libpq-dev build-essential

# ✅ Install sqlx CLI versi lama
RUN cargo install sqlx-cli --version 0.7.3 --no-default-features --features postgres

# ✅ Set env for migration
ARG DATABASE_URL
ENV DATABASE_URL=${DATABASE_URL}

# ✅ Run migration
RUN cargo sqlx migrate run

# Final build
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates libssl3 libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/be /app/be

EXPOSE 8080
ENV PORT=8080
ENV DATABASE_URL=${DATABASE_URL}

CMD ["./be"]