# 0015. World voxels of 25 cm, model voxels of 12.5 cm

- Status: accepted
- Date: 2026-10-30
- Requirements: SPEC NFR-4, FR-11, FR-26, FR-30; DESIGN.md (voxel size, references)

## Context

DESIGN.md set one voxel unit of 1/16 block with 1 m blocks, Minecraft scale, and the first terrain rendered 1 m cubes. Reviewing it in the game, the owner found it reads as Minecraft, not as voxel art, and chose a Teardown-like look: small cubes everywhere, with light doing the work. Voxel size drives chunk size in metres, memory, meshing cost, collision, destruction detail and view distance on phones and the web, so it is costly to change later.

## Decision

- The world grid uses voxels of 25 cm (4 per metre): terrain, buildings and everything that can be dug or destroyed.
- Characters, items and props use voxels of 12.5 cm, exactly half, so models align with the world grid.
- Both sizes are constants in `voxel-core` (`VOXEL_SIZE_METRES`, `VOXELS_PER_METRE`, `MODEL_VOXEL_SIZE_METRES`). Pure crates work in voxel units; the game converts to metres for rendering, physics speeds and the camera.
- A chunk stays 32³ voxels, now 8 m per side. Terrain generation is rescaled (a new `GENERATOR_VERSION`), and the player steps up ledges of up to 50 cm without jumping.
- Far terrain is drawn with simpler level-of-detail meshes as needed to keep the view distance within the memory and frame budgets.

## Consequences

Surfaces have 16 times more voxel faces per square metre and chunks cover 64 times less volume, so streaming handles many more chunks for the same distance: uniform chunks (ADR 0012), jobs (ADR 0013) and level of detail carry that cost. Destruction and editing get finer detail. A 1.8 m character is about 14 model voxels tall. DESIGN.md's "one voxel size" rule becomes "two sizes, one grid".

## Alternatives considered

- 1 m blocks with 1/16 detail painted on faces: cheapest, but still reads as Minecraft.
- 1 m blocks with 16³ relief near the camera: two representations of every block.
- 12.5 cm world voxels: closer to Teardown, but about 8 times the cost of 25 cm; phones and the web would get a very small detailed area.
- 50 cm world voxels: cheap, but too close to the block look being replaced.
