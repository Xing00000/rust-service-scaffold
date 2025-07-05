#!/bin/bash
# Development script for running the application with hot reload

set -e

echo "🚀 Starting development server with hot reload..."

# Check if cargo-watch is installed
if ! command -v cargo-watch &> /dev/null; then
    echo "📦 Installing cargo-watch for hot reload..."
    cargo install cargo-watch
fi

# Start the development server with file watching
cargo watch -x "run -p bootstrap" -w crates -w presentation -w bootstrap