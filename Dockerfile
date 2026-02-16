# syntax=docker/dockerfile:1.21
# Multi-stage build for logcheck-fluent-bit-filter using cargo-chef for optimal caching
# Builds for x86_64 (amd64) only

ARG RUST_VERSION=1.84
FROM rust:${RUST_VERSION}-slim AS chef

# Install system dependencies and cargo-chef
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-chef for dependency caching
RUN cargo install cargo-chef

WORKDIR /app

# Prepare recipe - analyzes project structure for caching
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY xtask/ xtask/
RUN cargo chef prepare --recipe-path recipe.json

# Setup build targets
FROM chef AS builder-base
RUN rustup target add x86_64-unknown-linux-gnu wasm32-unknown-unknown
COPY --from=planner /app/recipe.json recipe.json

# Cook native dependencies (x86_64) - this layer is cached!
FROM builder-base AS native-deps
RUN cargo chef cook \
    --release \
    --target x86_64-unknown-linux-gnu \
    --recipe-path recipe.json

# Cook WASM dependencies - this layer is cached!
FROM builder-base AS wasm-deps
RUN cargo chef cook \
    --release \
    --target wasm32-unknown-unknown \
    --recipe-path recipe.json

# Build native CLI binary
FROM native-deps AS cli-builder
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-gnu --bin logcheck-filter

# Build WASM filter
FROM wasm-deps AS wasm-builder
COPY . .
RUN cargo build --release --target wasm32-unknown-unknown --lib

# Final runtime image with Fluent Bit
FROM fluent/fluent-bit:4.2.2

# Copy WASM filter (architecture-independent)
COPY --from=wasm-builder /app/target/wasm32-unknown-unknown/release/logcheck_fluent_bit_filter.wasm /fluent-bit/filters/

# Copy CLI binary (x86_64)
# Binary is already executable from cargo build, no chmod needed
COPY --from=cli-builder /app/target/x86_64-unknown-linux-gnu/release/logcheck-filter /usr/local/bin/logcheck-filter

LABEL org.opencontainers.image.source=https://github.com/finkregh/fluent-bit-logcheck
LABEL org.opencontainers.image.description="Fluent-bit with logcheck WASM filter and CLI tool"
LABEL org.opencontainers.image.licenses=MIT

# Default command runs Fluent Bit
CMD ["/fluent-bit/bin/fluent-bit", "-c", "/fluent-bit/etc/fluent-bit.conf"]
