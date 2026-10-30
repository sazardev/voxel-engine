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

## Agent hooks (Claude Code)

Project hooks live in `.claude/hooks/` and are registered in `.claude/settings.json`. They are a fast first line; the gates above stay the authority. `.claude/hooks/test-hooks.sh` self-tests the guards.

- `guard-write.sh` (PreToolUse Edit/Write) denies `utils`/`helpers`/`common`/`misc` modules, network crates, Bevy in `voxel-*`, `#[allow]` and `unwrap()` outside tests. Edits to `lefthook.yml`, `deny.toml`, `rust-toolchain.toml`, workspace lints and `.claude/` ask the human.
- `guard-bash.sh` (PreToolUse Bash) denies `--no-verify`, force push, amending a pushed commit, `reset --hard`, weakened `cargo deny`/`clippy` flags and `rm -r` outside the repo.
- `post-edit.sh` (PostToolUse Edit/Write) formats the edited file, runs `cargo check -p <crate>` and warns above 450 lines. It is silent on success.
- `prompt-context.sh` (UserPromptSubmit) adds repo state only when it changed, plus the `FR-*`/`NFR-*` text and ADR names the prompt mentions.
- `session-context.sh` (SessionStart) prints branch, changes, recent commits, the last fast-gate result and recent ADRs.
- `stop-check.sh` (Stop) runs the fast gates and records the result in `target/.claude-gates`.
- Hooks print nothing when all is well; output costs tokens, so keep messages short and capped.
