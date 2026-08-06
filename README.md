# StellarPlan Contracts

> The Soroban smart contracts behind StellarPlan — time-locked "plans" that protect a wallet's salary allocations on Stellar until their due dates.

<p align="center"><em>Rust · Soroban · built for the Drips Stellar Wave program (testnet)</em></p>

[![CI](https://github.com/stellarplan/stellarplan-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/stellarplan/stellarplan-contracts/actions/workflows/ci.yml)
![License](https://img.shields.io/badge/license-MIT-blue)
![Rust](https://img.shields.io/badge/Rust-wasm32v1--none-orange)
![Soroban](https://img.shields.io/badge/Soroban-PlanVault-blueviolet)

---

## What it is

Contracts live in [`contracts/`](./contracts). The shared workspace targets
`wasm32v1-none` for Soroban deployment.

| Contract | Description |
|---|---|
| `plan_vault` | One vault per wallet. Creates time-locked "plans" (formerly vaults) that protect salary allocations until their due dates. |

## Quick start

One command installs the wasm target (if missing), builds the release wasm, and
runs the tests:

```bash
./scripts/setup.sh
```

<details>
<summary>Manual steps (what the script automates)</summary>

```bash
rustup target add wasm32v1-none
cargo build --release --target wasm32v1-none
cargo test
```

</details>

The compiled artifact is written to `target/wasm32v1-none/release/plan_vault.wasm`.
The test suite is 10 unit/integration tests in `contracts/plan_vault/src/test.rs`.

## Deploy (testnet)

Set your keypair and USDC token, then run the deploy script (it prints the
contract id):

```bash
export STELLAR_SECRET_KEY=...        # funded testnet keypair
export USDC_TOKEN_CONTRACT=...       # USDC token contract on testnet
./scripts/deploy.sh                  # macOS / Linux — defaults to the testnet network
```

On Windows, use `./scripts/deploy.ps1`.

You will need:

- A funded Stellar testnet account (friendbot)
- `stellar` CLI installed (`stellar-cli` v22+)

After deploy, add the printed `VAULT_CONTRACT_ID` to the API's `.env`.

## Architecture

`plan_vault` uses a **single-owner-per-wallet** model: each wallet gets one vault,
which holds the wallet's time-locked plans until they're due.

```
        ┌─────────────────────────────────────────────┐
        │ plan_vault  (one vault instance per owner)   │
        │   owner  ── the wallet that controls it      │
        │   token  ── USDC token contract              │
        │   plans  ── [ { amount, unlock_at, ... } ]   │
        └─────────────────────────────────────────────┘
                        ▲
                        │  create / release / early-withdraw
                        │
                  StellarPlan API (NestJS)
```

The contract enforces ownership and time-locks only — orchestration, salary
detection, and scheduling all live in the [API](https://github.com/stellarplan/stellarplan-api),
which is the sole caller.

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

## CI

Every push and pull request to `main` runs the [CI workflow](./.github/workflows/ci.yml):
`build` (release wasm), `test` (`cargo test`), and `fmt` (`cargo fmt --check`
plus `cargo clippy -D warnings`). Use `build`, `test`, and `fmt` as required
status checks for branch protection on `main`.

## Related repositories

- [stellarplan-api](https://github.com/stellarplan/stellarplan-api) — NestJS backend and the only caller of this contract.
- [stellarplan-web](https://github.com/stellarplan/stellarplan-web) — Next.js frontend for the plans this contract secures.

## Maintainers

| Name | Contact |
|---|---|
| StellarPlan Team | <!-- add Telegram/email --> |

<!-- Maintainer: replace the placeholder above with a real name and a Telegram handle or email. -->

## Contributing

Contributions are welcome — see [CONTRIBUTING.md](./CONTRIBUTING.md). `main` is
protected, so all changes land via pull request with CI green.

**Security:** please report vulnerabilities privately per [SECURITY.md](./SECURITY.md).

**License:** [MIT](./LICENSE).

## Contributors

[![Contributors](https://contrib.rocks/image?repo=stellarplan/stellarplan-contracts)](https://github.com/stellarplan/stellarplan-contracts/graphs/contributors)
