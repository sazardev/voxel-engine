# 0011. Own greedy mesher with ambient occlusion, without block-mesh

- Status: accepted
- Date: 2026-10-30
- Requirements: DESIGN.md (ao: baked per vertex, 4 levels), SPEC NFR-4, FR-26; STACK.md (Meshing)

## Context

STACK.md named `block-mesh` for greedy quads. DESIGN.md requires ambient occlusion baked per vertex in four levels on every tier. `block-mesh` 0.2 merges faces by voxel value only, so a merged quad would stretch one face's corner darkening across the whole quad; its source leaves an AO-aware merge as a TODO. A custom merge rule means implementing its `unsafe fn find_quad` trait, and `unsafe` is denied in our crates (ADR 0007). It also pulls `glam 0.19`, a second copy next to Bevy's, and has had no release since 2022.

## Decision

`voxel-mesh` implements its own mesher in safe Rust:

- Input is a chunk padded with one voxel from each of its 26 neighbours, so faces and AO at chunk borders are correct.
- A face is emitted when a solid voxel touches air. Each corner gets an AO level from 0 (darkest) to 3 (open) from the two side voxels and the corner voxel in front of the face.
- Faces merge greedily per slice only when material and all four AO levels match. Because a shared vertex always has the same AO, a merged quad interpolates exactly like its unmerged faces.
- Quads become vertex buffers (positions, normals, AO level, material, indices). Each quad is split along its darker diagonal so AO does not show a triangle seam.

`block-mesh` is removed from the workspace.

## Consequences

AO is correct with greedy merging and no `unsafe`. We own and test the mesher: transparent voxels (water, glass), LOD meshes and other face rules are added here. Quads with different AO do not merge, so flat ground next to walls has more triangles than material-only merging.

## Alternatives considered

- `block-mesh` greedy quads without AO in the merge: wrong darkening across merged quads.
- `block-mesh` with a custom `MergeStrategy`: requires `unsafe`, an exception to ADR 0007 for little code.
- `block-mesh` visible faces (no merging) plus AO: correct, but about an order of magnitude more triangles on flat terrain.
