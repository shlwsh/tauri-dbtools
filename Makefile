.PHONY: all dev build install-deps clean

all: build

dev:
	@echo "Starting Tauri development mode..."
	cd frontend && bun install
	bun run tauri:dev

build:
	@echo "Building Tauri application..."
	cd frontend && bun install
	bun run tauri:build

install-deps:
	@echo "Installing dependencies..."
	cd frontend && bun install
	cd src-tauri && cargo build

clean:
	@echo "Cleaning build artifacts..."
	cd frontend && rm -rf dist node_modules .bun
	cd src-tauri && cargo clean
