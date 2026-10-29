## What and why

<!-- One concern per PR. Link the requirement (FR-/NFR-) and any ADR. -->

## Checklist (reviewers verify)

- [ ] `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --all-features`, `cargo deny check` pass
- [ ] Every changed behaviour has a test in this change; bug fixes have a regression test
- [ ] New or changed decision has an ADR; `STACK.md` / `SPEC.md` updated if affected
- [ ] No Bevy in `voxel-*`; no platform branching in gameplay; no network; no `utils`/`helpers`
- [ ] Each new file does one thing; no function over the limits without a reason here
- [ ] No error is swallowed; no unbounded queue; Low tier still looks right (render changes)
- [ ] Dependency updates are in their own PR
