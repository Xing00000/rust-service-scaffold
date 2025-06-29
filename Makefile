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
	@echo " Axum Hexagonal Template - Makefile Commands"
	@echo "=========================================================="
	@echo " Usage: make [command]"
	@echo ""
	@echo " Core Commands:"
	@echo "   help              : Display this help message."
	@echo "   dev               : Build and run the application in development mode."
	@echo "   run               : Run the last built development binary."
	@echo "   build             : Build the application in development mode."
	@echo "   build-release     : Build the application in release mode (optimized)."
	@echo "   run-release       : Run the last built release binary."
	@echo "   test              : Run all tests (unit, integration, doc tests)."
	@echo "   check             : Run a quick check for type errors without building."
	@echo "   fmt               : Format all Rust code."
	@echo "   clippy            : Run Clippy linter."
	@echo "   doc               : Generate HTML documentation."
	@echo "   clean             : Clean build artifacts."
	@echo "   audit             : Run `cargo audit` to check for security vulnerabilities."
	@echo "=========================================================="

# --- Development Commands ---

.PHONY: dev
dev: build
	@echo "🚀 Running $(APP_CRATE) in development mode..."
	$(APP_BINARY_PATH) --port $(PORT) # Pass port if your config allows CLI override
	# Alternative: use `cargo run` if you don't want a pre-build step
	# cargo run -p $(APP_CRATE) -- --port $(PORT)

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
	@echo "🧪 Running all tests..."
	# Run unit tests (within each crate's src/tests or inline) and doc tests
	# `cargo test --workspace` runs tests for all members.
	cargo test --workspace

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

# --- Phony Targets ---
# .PHONY declares targets that are not actual files.
# This ensures that Make always runs the associated commands.
.PHONY: help dev run build build-release run-release test check fmt clippy doc audit clean