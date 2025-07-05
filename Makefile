# ===================================================================
# Makefile for axum_hexagonal_template
#
# Provides common development, build, test, and cleanup commands.
# Designed for a Cargo Workspace setup.
# ===================================================================

# --- Configuration Variables ---
# The main application binary crate.
APP_CRATE := app
# Default host and port for running the application.
HOST := 127.0.0.1
PORT := 8080
# Path to the main executable within the target directory.
# Adjust if your APP_CRATE produces a different binary name.
APP_BINARY_PATH := target/debug/$(APP_CRATE)
RELEASE_APP_BINARY_PATH := target/release/$(APP_CRATE)

# --- Default Goal ---
.DEFAULT_GOAL := help

# --- Help Command ---
help:
	@echo "=========================================================="
	@echo " Rust Service Scaffold - Makefile Commands"
	@echo "=========================================================="
	@echo " Usage: make [command]"
	@echo ""
	@echo " Development Commands:"
	@echo "   dev               : Start development server with hot reload"
	@echo "   build             : Build the application in development mode"
	@echo "   build-release     : Build the application in release mode (optimized)"
	@echo "   run               : Run the last built development binary"
	@echo "   run-release       : Run the last built release binary"
	@echo ""
	@echo " Testing & Quality:"
	@echo "   test              : Run all tests (unit, integration, doc tests)"
	@echo "   quality-check     : Run complete quality check pipeline"
	@echo "   fmt               : Format all Rust code"
	@echo "   clippy            : Run Clippy linter"
	@echo "   audit             : Run security vulnerability check"
	@echo ""
	@echo " Docker & Infrastructure:"
	@echo "   docker-up         : Start development dependencies (PostgreSQL, Redis)"
	@echo "   docker-down       : Stop development dependencies"
	@echo "   docker-logs       : Show logs from development dependencies"
	@echo ""
	@echo " Utilities:"
	@echo "   check             : Run a quick check for type errors"
	@echo "   doc               : Generate HTML documentation"
	@echo "   clean             : Clean build artifacts"
	@echo "   help              : Display this help message"
	@echo "=========================================================="

# --- Development Commands ---

.PHONY: dev
dev:
	@echo "🚀 Starting development server with hot reload..."
	./scripts/dev.sh

.PHONY: run
run:
	@echo "🏃 Running $(APP_CRATE) development binary..."
	$(APP_BINARY_PATH)

.PHONY: build
build:
	@echo "🛠️ Building $(APP_CRATE) in development mode..."
	cargo build -p $(APP_CRATE)

.PHONY: build-release
build-release:
	@echo "🚀 Building $(APP_CRATE) in release mode (optimized)..."
	cargo build -p $(APP_CRATE) --release

.PHONY: run-release
run-release: build-release
	@echo "🚀 Running $(APP_CRATE) release binary..."
	$(RELEASE_APP_BINARY_PATH) --port $(PORT) # Pass port if your config allows CLI override

# --- Testing & Quality Commands ---

.PHONY: test
test:
	@echo "🧪 Running comprehensive test suite..."
	./scripts/test.sh

.PHONY: quality-check
quality-check:
	@echo "🔍 Running complete quality check pipeline..."
	./scripts/quality-check.sh

.PHONY: check
check:
	@echo "🔍 Checking for type errors..."
	cargo check --workspace

.PHONY: fmt
fmt:
	@echo "💅 Formatting code..."
	cargo fmt --all

.PHONY: clippy
clippy:
	@echo "✅ Running Clippy linter..."
	cargo clippy --workspace -- -D warnings # Treat warnings as errors

.PHONY: doc
doc:
	@echo "📚 Generating documentation..."
	cargo doc --workspace --no-deps --open

.PHONY: audit
audit:
	@echo "🔒 Checking for security vulnerabilities with `cargo audit`..."
	# Requires `cargo install cargo-audit`
	cargo audit

# --- Cleanup ---

.PHONY: clean
clean:
	@echo "🗑️ Cleaning build artifacts..."
	cargo clean

# --- Docker Commands ---

.PHONY: docker-up
docker-up:
	@echo "🐳 Starting development dependencies..."
	docker-compose up -d
	@echo "⏳ Waiting for services to be ready..."
	sleep 5
	@echo "✅ Development environment is ready!"

.PHONY: docker-down
docker-down:
	@echo "🐳 Stopping development dependencies..."
	docker-compose down

.PHONY: docker-logs
docker-logs:
	@echo "📋 Showing logs from development dependencies..."
	docker-compose logs -f

# --- Phony Targets ---
# .PHONY declares targets that are not actual files.
# This ensures that Make always runs the associated commands.
.PHONY: help dev run build build-release run-release test check fmt clippy doc audit clean