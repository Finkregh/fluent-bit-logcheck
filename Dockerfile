# syntax=docker/dockerfile:1.21
# Multi-stage build for logcheck-fluent-bit-filter with multi-architecture support
# Use buildplatform for cross-compilation performance
FROM --platform=$BUILDPLATFORM rust:1.84-slim AS builder

# Build arguments for multi-architecture support
ARG BUILDPLATFORM
ARG TARGETPLATFORM
ARG TARGETOS
ARG TARGETARCH

WORKDIR /build

# Install base dependencies and cross-compilation tools
RUN apt-get update && \
    apt-get install -y \
        pkg-config \
        libssl-dev \
        build-essential \
        gcc-aarch64-linux-gnu \
        libc6-dev-arm64-cross \
        && \
    rm -rf /var/lib/apt/lists/*

# Configure target architecture and Rust targets
RUN case "$TARGETARCH" in \
        "amd64") \
            echo "export TARGET_ARCH=x86_64-unknown-linux-gnu" >> /build/env && \
            echo "export CC=gcc" >> /build/env && \
            echo "export CXX=g++" >> /build/env \
            ;; \
        "arm64") \
            echo "export TARGET_ARCH=aarch64-unknown-linux-gnu" >> /build/env && \
            echo "export CC=aarch64-linux-gnu-gcc" >> /build/env && \
            echo "export CXX=aarch64-linux-gnu-g++" >> /build/env && \
            echo "export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc" >> /build/env && \
            echo "export PKG_CONFIG_ALLOW_CROSS=1" >> /build/env \
            ;; \
        *) echo "Unsupported architecture: $TARGETARCH" && exit 1 ;; \
    esac

# Source environment and install Rust targets
RUN . /build/env && \
    rustup target add wasm32-unknown-unknown && \
    rustup target add $TARGET_ARCH

# Copy manifests first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy source to cache dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    echo "// dummy lib" > src/lib.rs

# Build dependencies with cache mounts
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/build/target,id=target-${TARGETARCH} \
    . /build/env && \
    cargo build --release --target $TARGET_ARCH --bin logcheck-filter && \
    cargo build --release --target wasm32-unknown-unknown --lib && \
    rm -rf src

# Copy actual source code
COPY src ./src

# Build WASM filter with cache mounts (release mode required due to stack constraints)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/build/target,id=target-wasm \
    cargo build --release --target wasm32-unknown-unknown --lib

# Build native CLI binary with cache mounts and cross-compilation
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/build/target,id=target-${TARGETARCH} \
    . /build/env && \
    cargo build --release --bin logcheck-filter --target $TARGET_ARCH

# Intermediate stage to select the correct binary
FROM builder AS binary-selector

ARG TARGETARCH

# Create output directory and copy the appropriate binary
RUN . /build/env && \
    mkdir -p /output && \
    cp /build/target/$TARGET_ARCH/release/logcheck-filter /output/logcheck-filter

# Final runtime image with Fluent Bit
FROM fluent/fluent-bit:4.2.2

# Copy WASM filter (architecture-independent)
COPY --from=builder /build/target/wasm32-unknown-unknown/release/logcheck_fluent_bit_filter.wasm /fluent-bit/filters/

# Copy CLI binary for current architecture
COPY --from=binary-selector /output/logcheck-filter /usr/local/bin/logcheck-filter

# Ensure binary is executable
RUN chmod +x /usr/local/bin/logcheck-filter

# Create configuration directory
RUN mkdir -p /fluent-bit/etc

LABEL org.opencontainers.image.source=https://github.com/finkregh/fluent-bit-logcheck
LABEL org.opencontainers.image.description="Fluent-bit with logcheck WASM filter and CLI tool"
LABEL org.opencontainers.image.licenses=MIT

# Default command runs Fluent Bit
CMD ["/fluent-bit/bin/fluent-bit", "-c", "/fluent-bit/etc/fluent-bit.conf"]
