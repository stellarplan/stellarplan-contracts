## Summary

<!-- What does this contract change and why? -->

## Changes

<!-- Bullet the key changes. -->
-

## Testing

- [ ] `cargo test` passes
- [ ] `cargo build --target wasm32-unknown-unknown --release` passes
- [ ] `cargo fmt --all -- --check` and `cargo clippy` clean

## Security

<!-- Contract code holds user funds. Call out any change to auth, time-locks,
     balance accounting, or the storage layout. Note if a migration is needed. -->

## Checklist

- [ ] No secret keys or deployed contract IDs committed
- [ ] Storage layout changes are backward compatible or migration documented
- [ ] New behaviour covered by a test in `test_snapshots`/unit tests
