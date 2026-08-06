#!/usr/bin/env bash
# StellarPlan Contracts — one-command build + test.
#
#   ./scripts/setup.sh
#
# Installs the wasm target if missing, builds the release wasm, and runs tests.
# To DEPLOY to testnet afterwards, set STELLAR_SECRET_KEY and USDC_TOKEN_CONTRACT
# in your environment and run ./scripts/deploy.sh (it prints the contract id).
set -euo pipefail
cd "$(dirname "$0")/.."

say() { printf '\n\033[1;36m==> %s\033[0m\n' "$1"; }

say "Ensuring wasm target is installed"
rustup target add wasm32v1-none 2>/dev/null || true

say "Building release wasm"
cargo build --release --target wasm32v1-none

say "Running tests"
cargo test

say "Contract built. Artifact: target/wasm32v1-none/release/plan_vault.wasm"
echo "   Deploy with:  STELLAR_SECRET_KEY=... USDC_TOKEN_CONTRACT=... ./scripts/deploy.sh"
