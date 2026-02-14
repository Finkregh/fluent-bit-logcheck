.PHONY: build test build-cli install-cli

DOCKER_IMAGE_NAME=fluent/fluent-bit:latest

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
	cargo build --release

test_json: test_deps build
	docker run --rm \
		--mount type=bind,source=$(shell pwd)/target/wasm32-unknown-unknown/release,target=/build_out \
		${DOCKER_IMAGE_NAME} \
		/opt/fluent-bit/bin/fluent-bit \
			-i dummy \
			-F wasm -p event_format=json -p wasm_path=/build_out/logcheck_fluent_bit_filter.wasm -p function_name=hello_world__json -m '*' \
			-o stdout -m '*'
test_msgpack: test_deps build
	docker run --rm \
		--mount type=bind,source=$(shell pwd)/target/wasm32-unknown-unknown/release,target=/build_out \
		${DOCKER_IMAGE_NAME} \
		/opt/fluent-bit/bin/fluent-bit \
			-i dummy \
			-F wasm -p event_format=msgpack -p wasm_path=/build_out/logcheck_fluent_bit_filter.wasm -p function_name=hello_world__msgpack -m '*' \
			-o stdout -m '*'

# Build CLI tool for native target
build-cli:
	cargo build --release --target x86_64-apple-darwin --bin logcheck-filter

# Install CLI tool to local bin directory
install-cli: build-cli
	cp target/x86_64-apple-darwin/release/logcheck-filter ~/.local/bin/