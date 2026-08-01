#!/usr/bin/env bash
# StellarPlan — deploy plan_vault to Stellar testnet
#
# Prerequisites:
#   1. stellar-cli v22+  (https://developers.stellar.org/docs/tools/developer-tools/stellar-cli)
#   2. A funded testnet keypair (get one from https://laboratory.stellar.org/#account-creator)
#   3. The Stellar testnet friendbot to fund the account
#
# Usage:
#   ./scripts/deploy.sh <SOURCE_ACCOUNT_SECRET>

set -euo pipefail

NETWORK="${1:-testnet}"
CONTRACT_WASM="target/wasm32v1-none/release/plan_vault.wasm"
SOURCE_ACCOUNT="${STELLAR_SECRET_KEY:-}"

if [ -z "$SOURCE_ACCOUNT" ]; then
  echo "ERROR: Set STELLAR_SECRET_KEY before running."
  exit 1
fi

echo "==> Building contract..."
cargo build --release --target wasm32v1-none

echo "==> Installing wasm into network..."
stellar contract install \
  --wasm "$CONTRACT_WASM" \
  --network "$NETWORK"

WASM_HASH=$(stellar contract install \
  --wasm "$CONTRACT_WASM" \
  --network "$NETWORK" 2>&1 | tail -1)

echo "==> Creating contract instance..."
CONTRACT_ID=$(stellar contract create \
  --wasm-hash "$WASM_HASH" \
  --network "$NETWORK" \
  --source "$SOURCE_ACCOUNT" \
  -- \
  --owner "$(stellar keys show "$SOURCE_ACCOUNT" 2>/dev/null || stellar keys public "$SOURCE_ACCOUNT")" \
  --token "$(stellar contract asset --network "$NETWORK" native 2>/dev/null || echo 'USDC:GBBD47ifRHETYWFMDLQ7RdjwS9q3TzrC6tS2h9xV7vT8W7T8zB3M2u9')" \
  | tail -1)

echo ""
echo "Contract deployed!"
echo "Contract ID: $CONTRACT_ID"
echo ""
echo "Next steps:"
echo "  - Add VAULT_CONTRACT_ID=$CONTRACT_ID to your .env / Render secrets"
echo "  - Fund the contract (for release testing) via a testnet payment"
