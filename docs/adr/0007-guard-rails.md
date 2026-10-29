# 0007. Guard rails: architecture tests and lint policy

- Status: accepted
- Date: 2026-10-29
- Requirements: STACK.md (Engineering principles: SOLID, one file one responsibility, Hexagonal)

## Context

Every change is written by an AI agent. Rules written only in prose get forgotten; rules that fail the build are followed.

## Decision

Enforce the principles mechanically, inside the existing gates:

- **`guard` crate** (`crates/guard/tests/architecture.rs`, run by `cargo test --workspace`): files over 500 lines fail (300 prints a note); no `utils`/`helpers`/`common`/`misc` modules; `voxel-*` crates contain no engine, IO, clock or ambient randomness; no network code; no `target_os`/`target_arch` in `game`; every ADR is indexed and has the template sections.
- **`cargo deny` bans** keep the crate graph a DAG: only `game` may depend on `platform`, `voxel-gen` and `voxel-mesh`; `bevy` is limited to `game`, `platform` and Bevy plugins; network stacks (`reqwest`, `hyper`, `ureq`, `tungstenite`, `openssl`, `native-tls`) are banned.
- **Clippy policy** (workspace `Cargo.toml`): pedantic warnings, `unwrap_used`, `todo`, `dbg_macro`, `allow_attributes`, `exit` denied; `#[expect(lint, reason = "...")]` replaces `#[allow]`; complexity, function length (60) and parameters (5) are bounded.
- `unsafe` is denied; a module that needs it uses `#[expect(unsafe_code, reason = "...")]`, a `// SAFETY:` comment and its own ADR.

## Consequences

Violations fail locally (pre-push) and in CI with a message naming the rule. A legitimate exception needs an ADR and a narrowly scoped change to the guard or the deny list, never a silent bypass.

## Alternatives considered

- A separate lint script in CI: breaks "CI runs exactly the Commands table" and is not run locally.
- Prose-only rules: not reliably followed by agents.
