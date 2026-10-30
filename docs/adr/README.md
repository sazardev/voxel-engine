# Architecture Decision Records

One decision per file, numbered `NNNN-kebab-title.md`, from `0000-template.md`.

- Status: `proposed` → `accepted` → `superseded by NNNN` or `deprecated`.
- Accepted ADRs are immutable; to change one, write a new ADR that supersedes it.
- Write an ADR for anything costly to reverse: dependencies, crate boundaries, data formats, platform trade-offs, deviations from `STACK.md`.

| #    | Title                                             | Status             |
| ---- | ------------------------------------------------- | ------------------ |
| 0001 | Rust workspace, toolchain and quality gates       | accepted           |
| 0002 | Crate boundaries: pure core, adapters at the edge | accepted           |
| 0003 | Hooks: fast pre-commit, tests on pre-push         | accepted           |
| 0004 | Defer Bevy and engine crates to the first feature | superseded by 0005 |
| 0005 | Engine dependency versions                        | accepted           |
| 0006 | Serialization: postcard, compression deferred     | accepted           |
| 0007 | Guard rails: architecture tests and lint policy   | accepted           |
| 0008 | Documentation structure and extra checks          | accepted           |
| 0009 | Dev mode as a compile-time feature                | accepted           |
| 0010 | World generation in integer arithmetic            | accepted           |
| 0011 | Own greedy mesher with AO, without block-mesh     | accepted           |
