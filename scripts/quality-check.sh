#!/bin/bash
# Quality check script for CI/CD pipeline

set -e

echo "🔍 Running quality checks..."

# Format check
echo "💅 Checking code formatting..."
cargo fmt --all --check

# Clippy linting
echo "📎 Running Clippy linter..."
cargo clippy --all-targets --workspace -- -D warnings

# Security audit
echo "🔒 Running security audit..."
if ! command -v cargo-audit &> /dev/null; then
    echo "📦 Installing cargo-audit..."
    cargo install cargo-audit
fi
cargo audit

# Run tests
echo "🧪 Running tests..."
./scripts/test.sh

echo "✅ All quality checks passed!"