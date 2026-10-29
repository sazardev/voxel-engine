# Automation: CI, hooks and dependency updates

How the quality gates run. CI and hooks run exactly the commands in the Commands table.

- CI runs exactly the commands in the Commands table; no CI-only scripts.
- Third-party actions are pinned to a full commit SHA; `permissions:` default to `contents: read`.
- `concurrency` cancels superseded runs on the same branch; `main` is protected and requires a green CI.
- Hooks run fast checks on commit (format, lint) and tests plus `cargo deny` on push; CI is the authority (ADR 0003).
- Hooks run the same commands as the Commands table.
- Related packages update together (framework + its plugins, test tooling, linters).
- Patch updates and lockfile maintenance automerge only when CI is green; minor/major are reviewed.
- A major update gets its own pull request with the migration notes linked.
