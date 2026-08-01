# Plan Vault — Contract Reference

One contract instance is deployed **per user wallet**. It holds that user's
time-locked plans in a single Stellar asset (USDC on testnet).

Deployed instance per user means: wallet → its own vault contract address.
The API stores it on `users.vaultContractId` after deployment.

## Interface

| Method | Auth | Notes |
|---|---|---|
| `__constructor(owner, token)` | deployer (owner) | runs once |
| `create_plan(name, amount, plan_type, unlock_date) → u32` | owner | pulls funds in via SAC transfer |
| `release_plan(plan_id)` | anyone (safe — pays owner only) | Bill plans, after unlock date |
| `request_early_withdraw(plan_id)` | owner | step 1 of break flow |
| `confirm_early_withdraw(plan_id)` | owner | step 2 — pays out |
| `get_plan(plan_id) → Plan` | — | |
| `list_plans() → Vec<Plan>` | — | |
| `get_protected_total() → i128` | — | sum of locked amounts |
| `version() → Symbol` | — | `v1_0_0` |

## Plan types

| Type | `unlock_date` semantics | Release path |
|---|---|---|
| `Bill` | unix timestamp, must be future | `release_plan` after date; callable by anyone |
| `Emergency` | ignored (stored as 0) | only via two-step early withdrawal |
| `Savings` | unix timestamp | same as Bill on-chain (UI differs) |

## Errors (1–14)

`AlreadyInitialized` · `NotInitialized` · `Unauthorized` · `InvalidAmount` ·
`InvalidUnlockDate` · `UnlockDateInPast` · `PlanNotFound` · `PlanNotLocked` ·
`PlanNotBill` · `NotYetUnlocked` · `InsufficientBalance` ·
`EarlyWithdrawDelayNotMet` · `EarlyWithdrawNotRequested` · `Overflow`

## Events

Topics start with `vault`: `init`, `create`, `release`, `ew_req`, `ew_done`.

## Storage layout

Instance: `Owner`, `Token`, `Initialized`, `PlanCount`.
Persistent: `Plan(u32)`, `EarlyWithdrawRequest(u32)` — TTL-bumped ~30 days on write.

## Deploy

See [`scripts/deploy.sh`](../scripts/deploy.sh). After deployment, save the
contract id into the API as `VAULT_CONTRACT_ID` (or call the per-user deploy
flow when self-custody deployment is enabled).
