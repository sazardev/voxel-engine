---
description: Start a feature following the project rails (plan, tests first, one concern)
argument-hint: <feature or requirement id>
---

Implement: $ARGUMENTS

1. Read `AGENTS.md`, `docs/stack/layout.md`, `docs/stack/rules-for-agents.md`, the SPEC requirement ids involved and the ADRs for the area.
2. Decide where each part goes (see "Where things go" in `AGENTS.md`): pure rules in a `voxel-*` crate, adapters in `platform`, the Bevy feature as a `Plugin` in `game/src/<feature>/` registered in `GamePlugin`.
3. Declare any new port (trait) in the core crate that uses it, with a doc comment stating its contract and an in-memory fake.
4. Write the tests first (behaviour through the public API; property tests for pure logic; a headless `App` test for Bevy code).
5. Implement in small files, one responsibility each; typed errors; bounded queues; no `unwrap`.
6. Add an ADR if a hard-to-reverse decision was made; update the relevant `docs/` file and its index.
7. Run `/check`, then commit with a Conventional Commit scoped to the crate or area.
