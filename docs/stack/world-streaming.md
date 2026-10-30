# World streaming and LOD

How chunks load, cull and fade with bounded work per frame.

- Chunks load in rings by priority (in view first), with a per-frame budget for uploads and a cap on chunks in flight.
- Far LOD uses merged, simplified meshes and fades in with dithering, never a visible pop.
- Culling happens on the GPU from a single list of chunk bounds; the CPU does not iterate chunks per frame.

Current implementation (`crates/game/src/world/`). A chunk is 32³ voxels of 25 cm, 8 m per side (ADR 0015); full detail reaches 12 chunks (96 m), then height-field patches reach 192 m and 384 m (ADR 0016).

- A `StreamPlan` around the `StreamFocus` column lists the chunks to mesh (within the view radius, on the layers that can hold the surface) and the voxel data to keep (one more chunk around them, so every meshed chunk has its 26 neighbours). Both lists are nearest first; the plan changes only when the focus enters another chunk column.
- Generation and meshing run as jobs on the task pool, or inline on devices without threads (ADR 0013). Every queue is bounded by a `StreamBudget`: jobs started per frame, jobs in flight and mesh uploads per frame. Data, jobs and meshes outside the plan are dropped, so memory is bounded by the view radius.
- Uniform chunks hold no voxel array (ADR 0012), and uniform air or buried solid chunks are never meshed.
- Measured with 1 m voxels, before ADR 0015 (release, headless, radius 6): 35 ms and 65 frames to stream with threads, 134 ms and 132 frames inline; worst frame about 1.4 ms; 14.7 MB of voxel data.
- Measured with 25 cm voxels (release, headless, radius 12 = 96 m): 7630 chunks of voxel data (51 MB), 583 meshes with 1.07 M vertices (about 43 MB of vertex data at 40 bytes each). Full load in 356 ms with threads (910 ms inline); worst frame 1.7 ms (2.2 ms inline).
- Sprinting (6.5 m/s, 60 fps, 20 s), main-thread streaming work per frame with 25 cm voxels: with threads mean 0.49 ms, p99 1.1 ms, worst 1.3 ms; inline mean 0.75 ms, p99 1.7 ms, worst 5.2 ms. The quality gate allows no frame above 33 ms while walking at full speed.
- With level of detail (ADR 0016), defaults: 773 patches add about 0.6 M vertices; the scene has 1.67 M vertices (about 67 MB of vertex data) and 61 MB of voxel data; full load 640 ms with threads, worst frame about 2.4 ms; sprinting mean 0.73 ms, worst 2.0 ms with threads.
- Next costs to cut: vertex data of detailed chunks (positions, normals and colors as 32-bit floats, about 40 bytes per vertex), and keeping a patch until the detail replacing it is meshed.
