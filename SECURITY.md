# Security Policy

## Reporting a vulnerability

Email: security@stellarplan.app (or open a private GitHub security advisory).

## Contract invariants

Audit any change against these invariants:

1. `release_plan` pays **only** the stored owner, and only after `unlock_date` (Bill plans).
2. Emergency plans can never be auto-released — early withdrawal only.
3. Early withdrawal is two-step (`request_early_withdraw` → `confirm_early_withdraw`).
4. Storage TTL must be bumped on every write (`INSTANCE_BUMP_AMOUNT`).
5. Double-release and re-initialization must always fail.

Before mainnet: set a non-zero early-withdrawal delay
(`storage::get_early_withdraw_delay`) and get an external Soroban audit.
