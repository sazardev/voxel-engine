# 0001. Rust workspace, toolchain and quality gates

- Status: accepted
- Date: 2026-10-29
- Requirements: STACK.md (Stack, Commands, Quality gates)

## Context

All code is written by AI agents, so rules must be enforced by tools, not by memory. The game targets six platforms from one codebase.

## Decision

- One Cargo workspace (`crates/*`), edition 2024, resolver 3, stable toolchain pinned in `rust-toolchain.toml` with the `wasm32-unknown-unknown` target.
- Lints are centralized in `[workspace.lints]` (clippy pedantic as warnings, `unwrap_used`/`todo`/`dbg_macro` denied, `unsafe_code` denied); CI runs clippy with `-D warnings`.
- `rustfmt.toml` and `.editorconfig` fix formatting; `cargo fmt --check` is a gate.
- `cargo-deny` checks advisories, licenses, bans and sources; it also enforces ADR 0002.
- CI (`.github/workflows/ci.yml`) runs exactly the commands in STACK.md; actions are pinned by SHA; Renovate keeps dependencies current.
- Profiles: `dev` (opt 1, deps 3), `release` (thin LTO), `web` (size-optimized).

## Consequences

Agents get fast, objective feedback. Pedantic clippy is noisy at first; allow individual lints in the workspace table with a reason, not in code.

## Alternatives considered

- `cargo audit` alone: covers advisories only; `cargo-deny` covers licenses, bans and sources too.
- Nightly toolchain: unstable; nothing in the stack requires it.
