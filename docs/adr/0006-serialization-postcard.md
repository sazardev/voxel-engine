# 0006. Serialization: postcard, compression deferred

- Status: accepted
- Date: 2026-10-29
- Requirements: STACK.md (Save), SPEC FR-24, G-1

## Context

STACK.md named `bincode` + `zstd`. `bincode` 3.0.0 on crates.io is a tombstone release that only contains `compile_error!`, and the 2.x line is no longer developed. `zstd` builds C code and needs `clang` to target `wasm32-unknown-unknown`, which breaks the web build on machines without it.

## Decision

- Use `postcard` with `serde` for chunk and save serialization: maintained, compact, stable wire format, works on every target.
- Do not add a compressor yet. The save feature writes its own ADR choosing between `zstd` (requires `clang` for web) and a pure-Rust compressor such as `ruzstd` or `lz4_flex`.
- Saves carry a format version (SPEC FR-25).

## Consequences

The web build needs no C toolchain today. The save feature must settle compression before it ships.

## Alternatives considered

- `bincode` 2.x: unmaintained.
- `bitcode`: faster and smaller, but a less stable format guarantee; revisit if size matters.
