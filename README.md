# StellarPlan Contracts

Soroban smart contracts for the StellarPlan automatic financial planning platform.

Contracts live in [`contracts/`](./contracts). The shared workspace targets `wasm32v1-none` for Soroban deployment.

## Contracts

| Contract | Description |
|---|---|
| `plan_vault` | One vault per wallet. Creates time-locked "plans" (formerly vaults) that protect salary allocations until their due dates. |

## Setup

```bash
rustup target add wasm32v1-none
```

## Build

```bash
cargo build --release --target wasm32v1-none
```

The compiled contract artifact is written to:

```
target/wasm32v1-none/release/plan_vault.wasm
```

## Test

```bash
cargo test
```

10 unit/integration tests living in `contracts/plan_vault/src/test.rs`.

## Deploy (testnet)

```bash
./scripts/deploy.sh  # macOS / Linux
```

or

```powershell
./scripts/deploy.ps1  # Windows
```

You will need:

- A funded Stellar testnet account (friendbot)
- `stellar` CLI installed (`stellar-cli` v22+)

Fill in `scripts/deploy.sh` / `scripts/deploy.ps1` with your keypair before running.

## Structure

```
stellarplan-contracts/
├── contracts/
│   └── plan_vault/
│       ├── src/
│       │   ├── lib.rs       # Contract entrypoint
│       │   ├── types.rs     # Domain model
│       │   ├── storage.rs   # Storage helpers
│       │   ├── events.rs    # Contract events
│       │   ├── errors.rs    # Error codes
│       │   └── test.rs      # Unit tests
│       └── test_snapshots/
└── target/
```
