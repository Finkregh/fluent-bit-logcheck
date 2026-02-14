# Multi-stage build for logcheck-fluent-bit-filter
FROM rust:1.84-slim AS builder

WORKDIR /build

# Install required dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Install wasm32 target for building WASM filter
RUN rustup target add wasm32-unknown-unknown

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build WASM filter (release mode required due to stack constraints)
RUN cargo build --release --target wasm32-unknown-unknown --lib

# Build native CLI binary
RUN cargo build --release --bin logcheck-filter

# Final runtime image with Fluent Bit
FROM fluent/fluent-bit:4.2.2

# Copy WASM filter
COPY --from=builder /build/target/wasm32-unknown-unknown/release/fluentbit_rustwasmfilter.wasm /fluent-bit/filters/

# Copy CLI tool
COPY --from=builder /build/target/release/logcheck-filter /usr/local/bin/

# Create configuration directory
RUN mkdir -p /fluent-bit/etc

LABEL org.opencontainers.image.source=https://github.com/finkregh/fluent-bit-logcheck
LABEL org.opencontainers.image.description="Fluent-bit with logcheck WASM filter and CLI tool"
LABEL org.opencontainers.image.licenses=MIT

# Default command runs Fluent Bit
CMD ["/fluent-bit/bin/fluent-bit", "-c", "/fluent-bit/etc/fluent-bit.conf"]
