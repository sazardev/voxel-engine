# AGENTS.md

All code here is written by AI agents. These rules are binding; a review checks them.

Source of truth, read in this order before changing anything:

1. `STACK.md`: index and commands; then `docs/stack/rules-for-agents.md` and `docs/stack/layout.md`, plus the topic you touch (`docs/stack/README.md`).
2. `SPEC.md`: what the game does (requirements `FR-*`, `NFR-*`).
3. `DESIGN.md`: visual language.
4. `docs/principles/README.md`: how code is written; read the files for the area you change.
5. `docs/adr/`: why decisions were made. Read the ADRs touching the area you change.

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
typos
dprint check
```

Hooks (`lefthook install`) run them: pre-commit fmt, clippy, spelling and docs format; pre-push test + deny. In Claude Code, `/check` runs them all and a Stop hook runs the fast ones before you finish. Commands: `/new-adr`, `/new-feature`.

## Architecture in one screen

- `voxel-core`, `voxel-gen`, `voxel-mesh`: pure crates, **no Bevy**, fully unit-tested. Dependencies point inward only (`voxel-mesh` → `voxel-core`).
- `platform`: adapters (storage, input, capability probe). Ports are declared by the core and implemented here.
- `game`: composition root. The only place that wires concrete adapters; gameplay branches on capabilities and tier, never on `#[cfg(target_os)]`.
- Hexagonal rules, one-file-one-responsibility and the file size limits in `STACK.md` apply. No `utils`/`helpers`/`common` modules.

## Rails: how the principles are enforced

Do not rely on memory; the build fails on these (ADR 0007). Fix the code, never the check.

| Principle                    | Enforced by                                                              |
| ---------------------------- | ------------------------------------------------------------------------ |
| One file, one responsibility | `guard`: files over 500 lines fail; no `utils`/`helpers`/`common`/`misc` |
| Hexagonal / dependency rule  | `cargo deny` bans (DAG) and `guard` (no Bevy, IO, clock in `voxel-*`)    |
| Small functions, low nesting | clippy: 60 lines, 5 parameters, cognitive complexity                     |
| No swallowed errors / panics | clippy: `unwrap_used`, `expect_used`, `panic`, `todo` denied or warned   |
| No silent lint bypass        | clippy: `allow_attributes` denied; use `#[expect(..., reason = "...")]`  |
| Offline, platform-agnostic   | `guard` + `deny`: no network crates, no `target_os` in `game`            |
| Decisions are recorded       | `guard`: every ADR indexed and complete                                  |

SOLID in practice here:

- **S**: one reason to change per file; if its description needs "and", split it. Folders by feature (`world/streaming.rs`, `world/save.rs`), not by type.
- **O**: add a new plugin, pass or implementation instead of growing a `match`/`if` chain. Render effects are new passes; features are new `Plugin`s registered in `GamePlugin`.
- **L**: an implementation of a port honours its whole contract; never `unimplemented!()` a method.
- **I**: ports are small traits owned by the consumer (`ChunkStore`, `Clock`), declared in the core crate that uses them.
- **D**: the core takes ports as parameters or Bevy resources; only `game` (composition root) picks concrete adapters from `platform`.

Where things go:

| You are adding…                          | Put it in                                                                                         |
| ---------------------------------------- | ------------------------------------------------------------------------------------------------- |
| Voxel data, edits, coordinates           | `voxel-core` (pure, with tests)                                                                   |
| World generation                         | `voxel-gen` (seeded, deterministic)                                                               |
| Meshing, AO, LOD meshes                  | `voxel-mesh`                                                                                      |
| Storage, input mapping, capability probe | `platform`, implementing a port from the core                                                     |
| A gameplay or render feature             | `game/src/<feature>/` as a `Plugin`, added in `GamePlugin`                                        |
| Shared error or logging need             | an error enum (`thiserror`) in the crate that owns it; `tracing` for logs; never a `utils` module |

There is no network layer. "Connections" means ports and adapters: a new external resource (file, storage, device) is a port in the core plus an adapter in `platform`.

## Definition of done

1. Requirement id and ADRs read; the change does one thing.
2. Tests added first or alongside; behaviour, not implementation.
3. All the checks above pass locally (hooks run them).
4. ADR written if a hard-to-reverse decision was made; `STACK.md`/`SPEC.md` updated if affected.
5. Commit is Conventional, scoped to a crate or area, without unrelated changes.

## Rust specifics

- Edition 2024, stable toolchain pinned by `rust-toolchain.toml`. Lints live in the workspace `Cargo.toml`; crates opt in with `[lints] workspace = true`.
- No `unwrap()` in non-test code; `expect` only for invariants, with a message. Errors are typed (`thiserror`) in libraries; map them once at the boundary.
- `unsafe` is denied. Allow it only in a documented module with `#[expect(unsafe_code, reason = "...")]`, a `// SAFETY:` comment and an ADR.
- Serialization is `postcard` + `serde` (ADR 0006); logging is `tracing` (Bevy's `info!` etc. in `game`).
- Deterministic code takes seeds and clocks as parameters; no `SystemTime`, `thread_rng` or `HashMap` iteration order in anything that feeds world data.
- Every queue and cache is bounded. Everything works offline; no network crates.
- New dependency: justify it in the change, add it under `[workspace.dependencies]`, check `cargo deny check`, and keep Bevy out of `voxel-*`.

## Tests

- Behaviour through the public API, named after the behaviour (`rejects_edit_outside_chunk`). Bug fix = regression test first.
- `voxel-core` is the reference: copy its patterns (typed errors, a port with an in-memory fake, a property test, a criterion benchmark).
- Pure crates: unit + property tests. Bevy code: headless `App` tests. No sleeps, no real clock, no network.
