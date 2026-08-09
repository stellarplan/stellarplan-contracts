# StellarPlan — deploy plan_vault to Stellar testnet
# Windows PowerShell equivalent of deploy.sh
#
# Requires Stellar CLI v22+ (validated against v27.1.0). Uses the single-step
# `stellar contract deploy`, which uploads the wasm, creates the instance, and
# runs the contract's __constructor(owner, token) in one transaction.

param(
    [string]$Network = "testnet"
)

$ErrorActionPreference = "Stop"

$CONTRACT_WASM  = "target/wasm32v1-none/release/plan_vault.wasm"
$SOURCE_ACCOUNT = $env:STELLAR_SECRET_KEY   # secret key: signs the tx (never printed)
$TOKEN_CONTRACT = $env:USDC_TOKEN_CONTRACT  # USDC token contract for the constructor

# Service account PUBLIC key — the __constructor `owner` argument. This is a
# public G-address (safe to expose). Override with $env:STELLAR_PUBLIC_KEY if you
# deploy from a different account.
if ($env:STELLAR_PUBLIC_KEY) {
    $OWNER_PUBLIC_KEY = $env:STELLAR_PUBLIC_KEY
} else {
    $OWNER_PUBLIC_KEY = "GC3FKTAJVY2JE2LQJU4DIUV3Y2J2ONKQ7VN7G3TY2QLUJEQJBT3OVRHN"
}

if (-not $SOURCE_ACCOUNT) {
    Write-Error "Set STELLAR_SECRET_KEY before running: `$env:STELLAR_SECRET_KEY = 'S...'"
    exit 1
}

if (-not $TOKEN_CONTRACT) {
    Write-Error "Set USDC_TOKEN_CONTRACT before running: `$env:USDC_TOKEN_CONTRACT = 'C...'"
    exit 1
}

Write-Host "==> Building contract..." -ForegroundColor Cyan
cargo build --release --target wasm32v1-none
if ($LASTEXITCODE -ne 0) { Write-Error "cargo build failed"; exit 1 }

if (-not (Test-Path $CONTRACT_WASM)) {
    Write-Error "WASM not found at $CONTRACT_WASM after build."
    exit 1
}

Write-Host "==> Deploying (upload + instantiate + __constructor)..." -ForegroundColor Cyan
Write-Host "    owner = $OWNER_PUBLIC_KEY" -ForegroundColor DarkGray
Write-Host "    token = $TOKEN_CONTRACT"   -ForegroundColor DarkGray

# --source accepts the secret key and uses it to sign (read from env, not logged).
# Constructor args follow the `--` separator as `--name value`.
$CONTRACT_ID = stellar contract deploy `
    --wasm $CONTRACT_WASM `
    --source $SOURCE_ACCOUNT `
    --network $Network `
    -- `
    --owner $OWNER_PUBLIC_KEY `
    --token $TOKEN_CONTRACT

if ($LASTEXITCODE -ne 0) { Write-Error "stellar contract deploy failed"; exit 1 }

Write-Host ""
Write-Host "Contract deployed!" -ForegroundColor Green
Write-Host "Contract ID: $CONTRACT_ID" -ForegroundColor Yellow
Write-Host ""
Write-Host "Add VAULT_CONTRACT_ID=$CONTRACT_ID to your API .env file."
