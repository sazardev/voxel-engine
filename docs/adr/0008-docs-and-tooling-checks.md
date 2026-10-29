# 0008. Documentation structure and extra checks

- Status: accepted
- Date: 2026-10-29
- Requirements: STACK.md (Automation), AGENTS.md

## Context

`STACK.md` had grown into one 290-line file mixing the stack, rendering rules, principles and agent rules. Agents read all of it for every change, and some principles (REST, N+1, indexes) did not apply to an offline game.

## Decision

- `STACK.md` is an index with the summary and the commands table. Detail lives in `docs/stack/` (one topic per file) and `docs/principles/` (one principle per file); each folder has a `README.md` index.
- The API-design principles are replaced by `docs/principles/data-contracts.md`; performance, security and error-handling are rewritten for an offline Rust game.
- The `guard` crate also checks the docs: every topic file is indexed and every referenced `.md` exists.
- New gates, run locally (hooks), in the Stop hook for agents and in CI: `typos` (spelling) and `dprint check` (Markdown and TOML formatting).
- Coverage (`cargo llvm-cov`) and benchmarks (`cargo bench -p voxel-core`) are signals, not gates. `cargo nextest` is not adopted: it adds nothing over `cargo test` here and skips doctests.
- `.claude/` ships a Stop hook that runs the fast gates, and the commands `/check`, `/new-adr`, `/new-feature`.
- `voxel-core` is the reference skeleton (typed errors, ports with fakes, property test, benchmark) that agents copy.

## Consequences

Agents load only the topics they touch. Adding a doc means indexing it or the build fails. Formatting churn is limited to table alignment.

## Alternatives considered

- Keep one `STACK.md`: simple, but costly to read and easy to leave stale.
- Prettier for Markdown: needs Node in CI; `dprint` is a single Rust binary that also formats TOML.
