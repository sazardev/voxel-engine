# 0003. Hooks: fast pre-commit, tests on pre-push

- Status: accepted
- Date: 2026-10-29
- Requirements: STACK.md (Conventions: hooks)

## Context

STACK.md says hooks run only fast checks and tests stay in CI. Agents commit often and a failure caught only in CI costs a round trip.

## Decision

Lefthook runs format and clippy on pre-commit, `cargo test` and `cargo deny check` on pre-push, and a Conventional Commits check on commit-msg. CI still runs everything, and is the authority. This refines the STACK.md convention: tests run on pre-push, not pre-commit.

## Consequences

Pushes are slower by the test time; if that grows, move heavy tests behind a feature or to CI only.

## Alternatives considered

- Tests only in CI: slower feedback for agents.
