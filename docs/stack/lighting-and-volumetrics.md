# Lighting and volumetrics

Shadows, fog and light shafts: stable, soft and cheap by tier.

- Snap each cascade to the shadow-map texel grid so shadows do not shimmer when the camera moves.
- Penumbra widens with distance from the occluder (PCSS) on the high tier; the low tier uses fixed PCF.
- Chunks are culled per cascade; shadow passes use the same indirect draw data as the main pass.
- Fog density, scattering color and anisotropy are parameters of the environment (day cycle, biome, interior), not constants in shaders.
- Lights opt in to volumetrics with a flag; only the sun and a few key lights scatter.
- Render the froxel grid at reduced resolution and upsample; jitter per frame and blend with history to hide banding.
