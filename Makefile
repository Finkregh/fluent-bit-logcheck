.PHONY: build test build-cli build-wasm build-plugin install-cli clean help
.PHONY: test_json test_msgpack test_deps
.PHONY: build-all-cli build-all-plugin

DOCKER_IMAGE_NAME=fluent/fluent-bit:latest

# Auto-detect platform for local builds
UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

ifeq ($(UNAME_S),Linux)
    ifeq ($(UNAME_M),x86_64)
        NATIVE_TARGET=x86_64-unknown-linux-gnu
    else ifeq ($(UNAME_M),aarch64)
        NATIVE_TARGET=aarch64-unknown-linux-gnu
    else
        NATIVE_TARGET=x86_64-unknown-linux-gnu
    endif
else ifeq ($(UNAME_S),Darwin)
    ifeq ($(UNAME_M),arm64)
        NATIVE_TARGET=aarch64-apple-darwin
    else
        NATIVE_TARGET=x86_64-apple-darwin
    endif
else
    NATIVE_TARGET=x86_64-unknown-linux-gnu
endif

# Common targets from CI
CLI_TARGETS=x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu x86_64-apple-darwin aarch64-apple-darwin
MUSL_TARGETS=x86_64-unknown-linux-musl aarch64-unknown-linux-musl
WINDOWS_TARGETS=x86_64-pc-windows-msvc
WASM_TARGET=wasm32-unknown-unknown

# Default target
help:
	@echo "Available targets:"
	@echo "  build          - Build WASM filter (release mode)"
	@echo "  build-cli      - Build CLI for native platform ($(NATIVE_TARGET))"
	@echo "  build-wasm     - Build WASM filter explicitly"
	@echo "  build-plugin   - Build shared library plugin for native platform"
	@echo "  build-all-cli  - Build CLI for all supported platforms"
	@echo "  build-all-plugin - Build plugin for all supported platforms"
	@echo "  install-cli    - Install CLI to ~/.local/bin"
	@echo "  test_json      - Test WASM filter with JSON format"
	@echo "  test_msgpack   - Test WASM filter with MessagePack format"
	@echo "  clean          - Clean build artifacts"
	@echo ""
	@echo "Detected native target: $(NATIVE_TARGET)"

test_deps:
# this if is a makefile operation that will be performed at makefile-parse-time
ifeq ("$(shell docker images -q ${DOCKER_IMAGE_NAME} 2> /dev/null)","")
	docker pull ${DOCKER_IMAGE_NAME}
endif

build:
	# Debug builds for whatever reason seem to use a tonne of stack space that
	# causes "wasm operand stack overflow" errors in the msgpack hello world
	# example specifically, so build in release by default.
	# (Fluentbit currently only gives 8kb of stack and heap to the WASM
	# execution context for its WASM filter:
	# https://github.com/fluent/fluent-bit/blob/v3.1.3/src/wasm/flb_wasm.c#L87
	# )
	cargo build --release --target $(WASM_TARGET) --lib

# Explicit WASM build target
build-wasm: build

# Build CLI tool for native target
build-cli:
	cargo build --release --target $(NATIVE_TARGET) --bin logcheck-filter

# Build shared library plugin for native target
build-plugin:
	cargo build --release --target $(NATIVE_TARGET) --lib

# Build CLI for all supported platforms (requires cross-compilation setup)
build-all-cli:
	@echo "Building CLI for all platforms..."
	@for target in $(CLI_TARGETS) $(MUSL_TARGETS) $(WINDOWS_TARGETS); do \
		echo "Building for $$target..."; \
		cargo build --release --target $$target --bin logcheck-filter || echo "Failed to build for $$target"; \
	done

# Build plugin for all supported platforms (requires cross-compilation setup)
build-all-plugin:
	@echo "Building plugin for all platforms..."
	@for target in $(CLI_TARGETS) $(MUSL_TARGETS); do \
		echo "Building plugin for $$target..."; \
		cargo build --release --target $$target --lib || echo "Failed to build plugin for $$target"; \
	done

test_json: test_deps build-wasm
	docker run --rm \
		--mount type=bind,source=$(shell pwd)/target/$(WASM_TARGET)/release,target=/build_out \
		${DOCKER_IMAGE_NAME} \
		/opt/fluent-bit/bin/fluent-bit \
			-i dummy \
			-F wasm -p event_format=json -p wasm_path=/build_out/logcheck_fluent_bit_filter.wasm -p function_name=hello_world__json -m '*' \
			-o stdout -m '*'

test_msgpack: test_deps build-wasm
	docker run --rm \
		--mount type=bind,source=$(shell pwd)/target/$(WASM_TARGET)/release,target=/build_out \
		${DOCKER_IMAGE_NAME} \
		/opt/fluent-bit/bin/fluent-bit \
			-i dummy \
			-F wasm -p event_format=msgpack -p wasm_path=/build_out/logcheck_fluent_bit_filter.wasm -p function_name=hello_world__msgpack -m '*' \
			-o stdout -m '*'

# Install CLI tool to local bin directory
install-cli: build-cli
	@mkdir -p ~/.local/bin
	cp target/$(NATIVE_TARGET)/release/logcheck-filter ~/.local/bin/
	@echo "Installed logcheck-filter to ~/.local/bin/"

# Clean build artifacts
clean:
	cargo clean
	@echo "Cleaned build artifacts"