# 0014. Player collision and rays against the voxel grid, not a physics engine

- Status: accepted
- Date: 2026-10-30
- Requirements: SPEC FR-16, FR-22, FR-57, FR-58; docs/stack/camera-and-input.md

## Context

The player has to walk on the terrain, the third person camera must never enter a solid block, and interaction (FR-58) needs to know which voxel the player looks at. `avian3d` is in the stack, but colliding a character with it means building a collider per chunk (a triangle mesh or thousands of cubes) and rebuilding it on every edit, plus contact solving the player does not need. The grid already answers "is this voxel solid?" exactly.

## Decision

- `voxel-core` owns two pure grid queries that take `is_solid` as a function, so they are tested without the engine (FR-22):
  - `Aabb::sweep`: moves a box one axis at a time (y first), stopping each axis at the first solid voxel with a 0.001 skin. A box that starts clear ends clear (property test over 50 000 random cases).
  - `cast_ray`: walks voxel boundaries (Amanatides and Woo) and returns the first solid voxel, its distance and face normal.
- `ChunkPos::split` maps a world voxel to its chunk and local position. One voxel is one metre; world voxel coordinates are `i64`.
- The game treats voxels of chunks that are not loaded as solid, so the player never falls through a world that is still streaming.
- `avian3d` stays for dynamic bodies (debris, FR-15), which is a later decision.

## Consequences

Collision is exact against the grid, costs a few hash lookups per step and needs no rebuild after edits. Slopes are steps of one voxel; stepping up without jumping (auto-step) is a later rule in the same sweep. Uniform chunk storage (ADR 0012) adds a branch to every `Chunk::set`: filling a chunk voxel by voxel went from 16 us to 26 us, acceptable because edits are rare.

## Alternatives considered

- `avian3d` character controller with chunk colliders: rebuild cost per edit and a second representation of the world.
- Collision against render meshes: couples gameplay to rendering and to mesh timing.
