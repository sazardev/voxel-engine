# 0016. Level of detail: a grid-aligned quadtree of height-field patches

- Status: accepted
- Date: 2026-10-30
- Requirements: SPEC FR-26 (distant terrain, no pops), FR-61 (memory), NFR-1; ADR 0015; docs/stack/world-streaming.md

## Context

With 25 cm voxels (ADR 0015) the detailed area costs about 1 M vertices and 51 MB of voxel data at 96 m, and doubling that distance in full detail would cost four times as much. The world needs to reach several hundred metres on desktop while phones and the web stay within tighter budgets.

## Decision

- The world around the focus is a quadtree of chunk columns aligned to a fixed grid: level 0 is detail (full voxels, the existing pipeline), level `n` is a patch of `2^n × 2^n` columns. A block splits while its nearest point is within the reach of the finer level (`view_radius × 2^level` chunks). Blocks never overlap or leave holes; the block holding the focus always splits. Defaults: detail to 96 m, level 1 to 192 m, level 2 to 384 m.
- Far terrain is a height field: each patch is one mesh of 16 × 16 columns (1 m at level 1, 2 m at level 2) built from `TerrainGenerator::cell_heights` with no voxel data kept. A column takes its highest voxel and rounds its surface up to a whole cell, so far terrain is made of bigger cubes and never dips below the detail.
- `voxel-mesh::mesh_heightfield` emits greedy-merged tops and walls, merged along each line; walls on a patch's outer edge reach four cells below the lower side (skirts) to hide gaps against other levels.
- Patches use the chunk material and the per-voxel tint scaled to their cells. Their jobs share the task-pool path of ADR 0013 with their own budget (background: 8 starts, 16 in flight, 4 uploads per frame; inline: 2, 2, 2). Fog reaches 5% contrast at 85% of the full reach.

## Consequences

Measured (release, headless, defaults): 773 patches add about 0.6 M vertices; with the detail area the scene has 1.67 M vertices (about 67 MB of vertex data), 61 MB of voxel data, a full load in 640 ms with threads, and the worst frame stays near 2.4 ms. Patches swap in and out when the focus crosses columns; a patch that becomes detail is removed at once, so there can be a brief hole until its chunks mesh. Caves and overhangs do not show beyond the detail area, which a height field cannot represent.

## Alternatives considered

- Downsampled voxel chunks for far levels (an octree of voxel LODs): handles overhangs but needs neighbour data per level and costs far more memory.
- More detail chunks with a larger radius: four times the cost for each doubling of distance.
- Patches with as many columns as voxels in a chunk (32 × 32): 4.3 M vertices for the same distance.
