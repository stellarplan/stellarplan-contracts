# StellarPlan — deploy plan_vault to Stellar testnet
# Windows PowerShell equivalent of deploy.sh

param(
    [string]$Network = "testnet"
)

$CONTRACT_WASM = "target/wasm32v1-none/release/plan_vault.wasm"
$SOURCE_ACCOUNT = $env:STELLAR_SECRET_KEY
$TOKEN_CONTRACT = $env:USDC_TOKEN_CONTRACT

if (-not $SOURCE_ACCOUNT) {
    Write-Error "Set STELLAR_SECRET_KEY before running: `$env:STELLAR_SECRET_KEY = 'S...'"
    exit 1
}

if (-not $TOKEN_CONTRACT -or $TOKEN_CONTRACT -eq "YOUR_USDC_TESTNET_TOKEN_CONTRACT_ID_HERE") {
    Write-Error "Set USDC_TOKEN_CONTRACT before running: `$env:USDC_TOKEN_CONTRACT = 'C...'"
    exit 1
}

Write-Host "==> Building contract..." -ForegroundColor Cyan
cargo build --release --target wasm32v1-none

Write-Host "==> Installing wasm into network..." -ForegroundColor Cyan
$wasmHash = stellar contract install `
    --wasm $CONTRACT_WASM `
    --network $Network 2>&1 | Select-Object -Last 1

Write-Host "WASM hash: $wasmHash" -ForegroundColor Yellow

Write-Host "==> Creating contract instance..." -ForegroundColor Cyan
$CONTRACT_ID = stellar contract create `
    --wasm-hash $wasmHash `
    --network $Network `
    --source $SOURCE_ACCOUNT `
    -- `
    --owner (stellar keys show $SOURCE_ACCOUNT 2>$null || stellar keys public $SOURCE_ACCOUNT) `
    --token $TOKEN_CONTRACT `
    2>&1 | Select-Object -Last 1

Write-Host ""
Write-Host "Contract deployed!" -ForegroundColor Green
Write-Host "Contract ID: $CONTRACT_ID" -ForegroundColor Yellow
Write-Host ""
Write-Host "Add VAULT_CONTRACT_ID=$CONTRACT_ID to your API .env file."

