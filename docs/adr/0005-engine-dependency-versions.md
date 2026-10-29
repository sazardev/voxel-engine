# 0005. Engine dependency versions

- Status: accepted
- Date: 2026-10-29
- Requirements: STACK.md (Stack, Dependencies)
- Supersedes: 0004

## Context

A Rust toolchain is now available, so the Bevy ecosystem versions could be resolved and compiled.

## Decision

Pin these minors under `[workspace.dependencies]`: `bevy 0.20`, `avian3d 0.8`, `leafwing-input-manager 0.22`. They resolve to a single `bevy_ecs 0.20.1`, and `cargo check -p game` passes natively and for `wasm32-unknown-unknown`. They are updated together as one Renovate group.

- `deny.toml` allows `MIT-0` (used by `encase`) and ignores RUSTSEC-2026-0192 (`ttf-parser` unmaintained, pulled in by Bevy text); revisit when Bevy drops it.
- Ubuntu CI installs `libasound2-dev` and `libudev-dev` for Bevy.
- `block-mesh`, `bincode` and `zstd` are added by the features that use them.

## Consequences

The first feature starts with a working engine. Each Bevy minor is a deliberate, grouped upgrade with its own pull request.

## Alternatives considered

- Wait until the first feature (ADR 0004): left dependency resolution as a hidden cost.
