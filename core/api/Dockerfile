# syntax=docker/dockerfile:1.7

FROM rust:1-bookworm AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends libpq-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app/core

COPY core/Cargo.toml core/Cargo.lock ./
COPY core/api/Cargo.toml api/Cargo.toml
COPY core/application/Cargo.toml application/Cargo.toml
COPY core/domain/Cargo.toml domain/Cargo.toml
COPY core/infrastructure/Cargo.toml infrastructure/Cargo.toml

RUN mkdir -p api/src application/src domain/src infrastructure/src \
    && printf 'fn main() {}\n' > api/src/main.rs \
    && printf 'pub fn placeholder() {}\n' > application/src/lib.rs \
    && printf 'pub fn placeholder() {}\n' > domain/src/lib.rs \
    && printf 'pub fn placeholder() {}\n' > infrastructure/src/lib.rs

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo fetch --locked

COPY core/ ./

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/core/target \
    find api/src application/src domain/src infrastructure/src -type f -exec touch {} + \
    && cargo build --release -p api \
    && cp /app/core/target/release/api /tmp/api

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libpq5 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /tmp/api /usr/local/bin/api

EXPOSE 3000

CMD ["api"]
