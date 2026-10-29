# 0009. Dev mode as a compile-time feature

- Status: accepted
- Date: 2026-10-29
- Requirements: STACK.md (Automation, Quality gates), NFR frame budgets

## Context

Testing the game needs an overlay, debug hotkeys, verbose logs and profiling, but a player build must be small, quiet and free of cheats. The tools must stay tested and linted so they do not rot.

## Decision

- A `dev` Cargo feature on `game` compiles the developer tools (`game/src/dev/`); `profile` implies `dev` and adds Chrome tracing. Player builds use no features.
- `main.rs` registers `DevPlugin` under `#[cfg(feature = "dev")]`; `GamePlugin` knows nothing about it. A guard test enforces that no other file references dev code.
- `RunMode` (`Player` or `Dev`, from `cfg!(feature = "dev")`) selects the log filter before startup.
- Quality gates run with `--all-features` (clippy and tests) so dev code is checked in CI; the wasm and release builds stay on the player configuration.
- Cargo aliases: `cargo dev`, `cargo play`, `cargo profile`.
- Dev tools change state only through gameplay resources (for example `Environment`).

## Consequences

Dev code is absent from shipped binaries and cannot leak cheats. Every dev tool is a small file with a test. Features are additive, so `--all-features` is a valid superset build.

## Alternatives considered

- Runtime flag or debug console in all builds: ships cheats and dev code to players.
- Build profile only (`debug_assertions`): cannot express profiling or release-with-tools builds.
