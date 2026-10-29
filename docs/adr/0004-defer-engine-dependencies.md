# 0004. Defer Bevy and engine crates to the first feature

- Status: superseded by 0005
- Date: 2026-10-29
- Requirements: STACK.md (Stack, Dependencies)

## Context

The foundation was prepared without a Rust toolchain available to resolve and compile dependencies. Bevy, `avian3d` and `leafwing-input-manager` must be pinned to mutually compatible versions.

## Decision

The workspace starts with empty crates. The first feature change adds Bevy (pin the minor) and compatible plugin versions under `[workspace.dependencies]`, runs `cargo deny check`, and records the chosen versions in a new ADR.

## Consequences

CI is green from day one. The first feature carries the dependency-resolution work.

## Alternatives considered

- Guess versions now: risks an unresolvable lockfile.
