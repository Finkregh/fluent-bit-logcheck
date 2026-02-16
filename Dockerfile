# syntax=docker/dockerfile:1.21
# Multi-stage build for logcheck-fluent-bit-filter using cargo-chef for optimal caching
# Builds for native architecture only (linux/amd64 on GitHub Actions CI)

ARG RUST_VERSION=1.85
FROM rust:${RUST_VERSION}-slim AS chef

# Install system dependencies
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

# Build dependencies
FROM chef AS builder
RUN rustup target add wasm32-unknown-unknown
COPY --from=planner /app/recipe.json recipe.json

# Cook dependencies for native CLI and WASM target
# Build sequentially to reduce memory usage
RUN cargo chef cook --release --recipe-path recipe.json && \
    cargo chef cook --release --target wasm32-unknown-unknown --recipe-path recipe.json

# Build both CLI (native) and WASM filter
COPY . .
RUN cargo build --release --bin logcheck-filter && \
    cargo build --release --target wasm32-unknown-unknown --lib

# Final runtime image with Fluent Bit
FROM fluent/fluent-bit:4.2.2

# Copy WASM filter (architecture-independent)
COPY --from=builder /app/target/wasm32-unknown-unknown/release/logcheck_fluent_bit_filter.wasm /fluent-bit/filters/

# Copy CLI binary (native architecture from default target)
COPY --from=builder /app/target/release/logcheck-filter /usr/local/bin/logcheck-filter

LABEL org.opencontainers.image.source=https://github.com/finkregh/fluent-bit-logcheck
LABEL org.opencontainers.image.description="Fluent-bit with logcheck WASM filter and CLI tool"
LABEL org.opencontainers.image.licenses=MIT

# Default command runs Fluent Bit
CMD ["/fluent-bit/bin/fluent-bit", "-c", "/fluent-bit/etc/fluent-bit.conf"]
