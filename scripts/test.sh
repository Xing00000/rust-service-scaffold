#!/bin/bash
# Test script for running all tests with coverage

set -e

echo "🧪 Running comprehensive test suite..."

# Run unit tests
echo "📋 Running unit tests..."
cargo test --workspace --lib

# Run integration tests
echo "🔗 Running integration tests..."
cargo test --workspace --test '*'

# Run doc tests
echo "📚 Running documentation tests..."
cargo test --workspace --doc

echo "✅ All tests completed successfully!"