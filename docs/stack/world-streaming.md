# World streaming and LOD

How chunks load, cull and fade with bounded work per frame.

- Chunks load in rings by priority (in view first), with a per-frame budget for uploads and a cap on chunks in flight.
- Far LOD uses merged, simplified meshes and fades in with dithering, never a visible pop.
- Culling happens on the GPU from a single list of chunk bounds; the CPU does not iterate chunks per frame.

Current implementation (`crates/game/src/world/`):

- A `StreamPlan` around the `StreamFocus` column lists the chunks to mesh (within the view radius, on the layers that can hold the surface) and the voxel data to keep (one more chunk around them, so every meshed chunk has its 26 neighbours). Both lists are nearest first; the plan changes only when the focus enters another chunk column.
- Generation and meshing run as jobs on the task pool, or inline on devices without threads (ADR 0013). Every queue is bounded by a `StreamBudget`: jobs started per frame, jobs in flight and mesh uploads per frame. Data, jobs and meshes outside the plan are dropped, so memory is bounded by the view radius.
- Uniform chunks hold no voxel array (ADR 0012), and uniform air or buried solid chunks are never meshed.
- Measured (release, headless, radius 6): 35 ms and 65 frames to stream with threads, 134 ms and 132 frames inline; worst frame about 1.4 ms; 14.7 MB of voxel data.
