# Contributing to StellarPlan Contracts

## Getting started

```bash
rustup target add wasm32v1-none
cargo build --release --target wasm32v1-none
cargo test
```

## Rules for this codebase

- The contract must stay **intentionally small** (per the PRD): create plans, lock funds,
  release funds, early withdrawal. Dashboards, reminders, and allocation logic belong in
  `stellarplan-api`, never here.
- Any state-model change requires a new test in `src/test.rs` plus a commit of the
  updated `test_snapshots/`.
- Funds may only ever transfer **to the vault owner**. Reject any PR that adds an
  arbitrary-destination transfer path.
- Keep events short (`symbol_short!` ≤ 9 chars) and prefixed with `vault`.

## Build profiles

- `release` — production artifact (`opt-level = "z"`, LTO, `panic = "abort"`).
- `release-assertions` — same optimization with debug assertions on; use for testnet QA.
