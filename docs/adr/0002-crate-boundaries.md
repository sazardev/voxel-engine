# 0002. Crate boundaries: pure core, adapters at the edge

- Status: accepted
- Date: 2026-10-29
- Requirements: STACK.md (Project layout, Hexagonal architecture), SPEC G-6

## Context

Game rules must run in unit tests without an engine (SPEC G-6), and the voxel logic must be reusable and deterministic across platforms.

## Decision

- `voxel-core`, `voxel-gen`, `voxel-mesh` never depend on Bevy or on platform code. `cargo-deny` bans `bevy` except for `game` and `platform`.
- Ports (traits such as storage or clock) are declared in the core crate that uses them; `platform` implements them.
- `game` is the only composition root.

## Consequences

Pure crates compile fast and test headless. Some glue types are duplicated at the Bevy boundary; that is accepted.

## Alternatives considered

- Bevy types everywhere: faster at first, but couples rules to the engine and slows tests.
