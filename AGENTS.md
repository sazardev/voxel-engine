# AGENTS.md

All code here is written by AI agents. These rules are binding; a review checks them.

Source of truth, read in this order before changing anything:

1. `STACK.md`: stack, layout, conventions, commands, quality gates, **Rules for agents**, **Engineering principles**.
2. `SPEC.md`: what the game does (requirements `FR-*`, `NFR-*`).
3. `DESIGN.md`: visual language.
4. `docs/adr/`: why decisions were made. Read the ADRs touching the area you change.

## Workflow

- Before a change: read the relevant ADRs and requirement ids. After: run the checks below.
- A decision that is hard to reverse (new dependency, crate boundary, format, platform trade-off) gets an ADR in the same change. Copy `docs/adr/0000-template.md`; never edit an accepted ADR, supersede it with a new one.
- If `STACK.md` or `SPEC.md` must change, change it in the same commit and say why.
- One concern per commit. Never mix dependency updates with feature changes.
- Commits follow Conventional Commits: `feat(voxel-core): …`, scope = crate or area.
- Never use `--no-verify`, never weaken or skip a CI step, never silence a lint without a comment saying why (`#[expect(lint, reason = "…")]`, not `#[allow]`).

## Checks (run before declaring done)

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
```

Hooks (`lefthook install`) run them: pre-commit format + lint, pre-push test + deny.

## Architecture in one screen

- `voxel-core`, `voxel-gen`, `voxel-mesh`: pure crates, **no Bevy**, fully unit-tested. Dependencies point inward only (`voxel-mesh` → `voxel-core`).
- `platform`: adapters (storage, input, capability probe). Ports are declared by the core and implemented here.
- `game`: composition root. The only place that wires concrete adapters; gameplay branches on capabilities and tier, never on `#[cfg(target_os)]`.
- Hexagonal rules, one-file-one-responsibility and the file size limits in `STACK.md` apply. No `utils`/`helpers`/`common` modules.

## Rust specifics

- Edition 2024, stable toolchain pinned by `rust-toolchain.toml`. Lints live in the workspace `Cargo.toml`; crates opt in with `[lints] workspace = true`.
- No `unwrap()` in non-test code; `expect` only for invariants, with a message. Errors are typed (`thiserror`) in libraries; map them once at the boundary.
- `unsafe` is denied. Allow it only in a documented module with a `// SAFETY:` comment and an ADR.
- Deterministic code takes seeds and clocks as parameters; no `SystemTime`, `thread_rng` or `HashMap` iteration order in anything that feeds world data.
- Every queue and cache is bounded. Everything works offline; no network crates.
- New dependency: justify it in the change, add it under `[workspace.dependencies]`, check `cargo deny check`, and keep Bevy out of `voxel-*`.

## Tests

- Behaviour through the public API, named after the behaviour (`rejects_edit_outside_chunk`). Bug fix = regression test first.
- Pure crates: unit + property tests. Bevy code: headless `App` tests. No sleeps, no real clock, no network.
