# Rust Compiler
COMPILER = cross

# Server details
# Lade die .env Datei, wenn sie existiert
ifneq (,$(wildcard .env))
	include .env
endif

# Targets
.PHONY: all build test deploy run clean

all: build test

build:
	$(COMPILER) build --target aarch64-unknown-linux-gnu --release
	@echo "Build complete."

test:
	$(COMPILER) test --target aarch64-unknown-linux-gnu
	@echo "Tests finished."

deploy: build
	scp -i $(SSH_KEY) -P $(PORT) target/aarch64-unknown-linux-gnu/release/$(TARGET_NAME) $(SERVER):$(REMOTE_PATH)/
	@echo "Deployment complete."
