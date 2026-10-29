# Rules for agents

Project-specific rules for AI agents. The entry point and workflow are in `../../AGENTS.md`.

- Put voxel logic in `voxel-*` crates with tests and keep Bevy types out of them.
- Add every visual effect as a pass with a tier, a cost budget and a lower-tier fallback; verify the Low tier still looks right.
- Never branch gameplay on the platform; branch on capabilities and tier.
- Measure before optimizing and report frame time per tier in the change.
- Keep touch, keyboard and gamepad on one set of input actions.
- Use the single-thread path on web; never assume threads exist.
- Keep the game fully offline.
- Add effects by inserting a pass in the chain; never apply grading or blur inside gameplay or material code.
- Respect the "reduce motion" setting: motion blur, camera shake and head bob go to zero.
- Change time of day or weather only through the `Environment` state; never set a light or fog value from gameplay code.
- Tune bias and cascade splits in one config; never add per-object shadow hacks.
- Volumetrics is one pass with tiers off / low / high (grid size, step count); never ray-march fog inside a material shader.
- Bound every queue (chunks in flight, uploads per frame); an unbounded queue is a bug.
- Keep movement in the fixed-step simulation and the camera interpolated from it, so look is smooth at any frame rate.
- Keep one rig for first and third person with a mode switch; do not fork the player controller.
- Never weaken or skip a CI step to make a change pass.
- A new quality gate is added to the Commands table and to `ci.yml` in the same change.
- Architecture rules (file size, no `utils`, pure crates, no network, no platform branching in gameplay) are enforced by the `guard` crate inside `cargo test --workspace --all-features`; a new rule becomes a new guard test, not a prose-only rule.
- Crate dependency direction is enforced by `cargo deny` bans; a new crate adds its ban in the same change.
- Never put secrets in workflow files; use repository or environment secrets.
- Never bypass hooks with `--no-verify`.
- Commit messages follow Conventional Commits (`feat(scope): …`, `fix(scope): …`).
- Never pin a dependency to an old version to silence Renovate without a comment saying why.
- Never mix a dependency update with feature changes in one commit.
