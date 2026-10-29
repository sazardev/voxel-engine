# World streaming and LOD

How chunks load, cull and fade with bounded work per frame.

- Chunks load in rings by priority (in view first), with a per-frame budget for uploads and a cap on chunks in flight.
- Far LOD uses merged, simplified meshes and fades in with dithering, never a visible pop.
- Culling happens on the GPU from a single list of chunk bounds; the CPU does not iterate chunks per frame.
