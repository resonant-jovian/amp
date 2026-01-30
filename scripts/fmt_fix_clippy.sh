#!/bin/bash
# Format, fix, and lint all Rust code in the project
#
# Runs rustfmt, clippy with auto-fixes, and final clippy check.

set -e

# Get repository root (parent of scripts directory)
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

echo "📦 Running rustfmt..."
cargo fmt --all

echo "🔧 Running clippy with fixes..."
cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged

echo "🔍 Final clippy check..."
cargo clippy --all-targets --all-features -- -D warnings

echo "✅ All checks passed!"
