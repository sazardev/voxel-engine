# Render pipeline

Post-processing chain, color grading and the single `Environment` state.

- Order of the chain is fixed and documented: TAA → depth of field → motion blur → bloom → tone mapping → color grading → film effects → UI.
- Color grading is a 3D LUT (32³) per environment, blended on transitions; the HUD is drawn after grading and is never graded.
- Depth of field and motion blur are physically motivated (focus distance, shutter fraction) and have a strength slider and an off switch.
- Every effect is a pass with off / low / high tiers; the settings screen exposes the ones that can cause discomfort.
- One `Environment` state (time, weather, biome, interior) is the single source for sun direction, light color, fog, sky and the grading LUT.
- Atmospheric perspective tints distant chunks toward the horizon color; fog distance follows the view distance setting.
- Interiors and caves override the environment smoothly (blend over about 1 second), never as a pop.

Current implementation: `environment/atmosphere.rs` sets the clear color and every 3D camera's `DistanceFog` from the `Environment` each frame. The sky at the horizon uses the palette's `ambient` ramp (day `#9FB7E8`, night `#27324F`); fog is exponential and leaves 5% contrast at 85% of the view distance, so the streamed edge fades into the sky.
