# World streaming and LOD

How chunks load, cull and fade with bounded work per frame.

- Chunks load in rings by priority (in view first), with a per-frame budget for uploads and a cap on chunks in flight.
- Far LOD uses merged, simplified meshes and fades in with dithering, never a visible pop.
- Culling happens on the GPU from a single list of chunk bounds; the CPU does not iterate chunks per frame.

Current implementation (`crates/game/src/world/`):

- A `StreamPlan` around the `StreamFocus` column lists the chunks to mesh (within the view radius, on the layers that can hold the surface) and the voxel data to keep (one more chunk around them, so every meshed chunk has its 26 neighbours). Both lists are nearest first; the plan changes only when the focus enters another chunk column.
- Each frame generates at most `GENERATE_PER_FRAME` chunks and meshes and uploads at most `MESH_PER_FRAME`. Data and meshes outside the plan are dropped, so memory is bounded by the view radius.
- The work runs on the main thread with these budgets, which is also the single-thread web path. Moving it to a task pool on threaded devices comes when a measurement shows the budgets are too slow.
