# Changelog

## v0.2.0 (contract version `v1_1_0`) — 2026-10-09

### Fixed
- Storage was only extended when a vault was created, although the docs said it
  was extended on every interaction. Every state-changing call now extends the
  instance entry and the entries it touches by about 30 days.
- A pending early-withdraw request now has its storage lifetime extended when it
  is recorded.
- The early-withdraw delay check used unchecked addition. It now uses checked
  arithmetic and returns `Overflow` instead of aborting.

### Added
- `set_early_withdraw_delay` and `get_early_withdraw_delay`: an owner-set
  cooling-off period (0 to 30 days, default 0) between requesting and
  confirming an early withdrawal. The previous behavior is the default.
- `cancel_early_withdraw`: abandon a pending request; the plan stays locked.
- `bump_ttl`: anyone can extend the storage lifetime of a vault and its plans.
  It moves no funds and changes no data.
- Plan names must be 1 to 64 bytes (`InvalidName`).
- Events `ew_can` and `ew_dly`; errors `InvalidName` (15) and `InvalidDelay` (16).
- 17 new tests (27 in total) for authorization failures, validation, delay
  boundaries, overflow, cancellation, and exact TTL values.

### Compatibility
All changes are additive. Existing entrypoints, error codes 1 to 14, and event
topics are unchanged, so the API and web app keep working without changes.

## v0.1.0
- Initial `plan_vault` contract with create, release, and two-step early withdrawal.
