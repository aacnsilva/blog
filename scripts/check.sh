#!/usr/bin/env bash
# Every check CI runs, in the order that fails fastest. Run it before pushing.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "==> Formatting"
cargo fmt --all --check

echo "==> Clippy"
cargo clippy --locked --all-targets -- -D warnings

echo "==> Tests, site contract and site lints"
cargo test --locked
