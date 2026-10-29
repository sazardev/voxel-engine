---
description: Run every quality gate and report what fails
---

Run these in order and stop at the first failure, fixing the cause (never the check):

1. `cargo fmt --all --check`
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
3. `cargo test --workspace --all-features` (includes the `guard` architecture and docs tests)
4. `cargo deny check`
5. `typos`
6. `dprint check`

Report which gates passed and the exact output of any failure.
