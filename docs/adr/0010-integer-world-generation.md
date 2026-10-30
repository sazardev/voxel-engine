# 0010. World generation in integer arithmetic, without a noise crate

- Status: accepted
- Date: 2026-10-30
- Requirements: SPEC FR-1, FR-29, FR-30, G-1; STACK.md (Project layout: `voxel-gen`)

## Context

SPEC G-1 requires that the same seed produces byte-identical world data on every supported platform, including `wasm32`. Generated terrain is not stored: saves keep only the edits on top of it, so the generator itself is part of the save format.

Noise crates (`noise`, `fastnoise-lite`, `bracket-noise`) compute in `f32`/`f64` and some call transcendental functions (`sin`, `exp`, `powf`) whose results come from the platform's math library and may differ in the last bit between targets. A single bit of difference near a threshold changes a voxel, and their output can also change between crate versions. A heightmap only needs smooth, seeded 2D noise, which is a few dozen lines.

## Decision

- `voxel-gen` implements its own noise with integers only: a `SplitMix64`-based lattice hash, 2D value noise with a smoothstep fade in 16.16 fixed point, and a sum of octaves. No floats are used for anything that becomes world data.
- No noise dependency is added.
- `GENERATOR_VERSION` (now 1) identifies the algorithm. Any change that alters generated voxels bumps it and needs an ADR; a world records the version it was created with so old saves keep their terrain.
- A golden test (`crates/voxel-gen/tests/golden.rs`) pins a digest of the postcard bytes of chunks from a fixed seed, so any change to generated worlds fails the build.
- The terrain guarantees hard bounds that other code may rely on: surface height in `MIN_SURFACE_HEIGHT..=MAX_SURFACE_HEIGHT` and at most `MAX_SURFACE_STEP` between neighbouring columns (no cliffs at seams, FR-30).

## Consequences

Determinism holds by construction on every target, and the generator has no dependency to update. We own the noise code: richer features (biomes, caves, gradient or simplex noise, domain warping) must be written in fixed point too, and each one that changes output bumps `GENERATOR_VERSION` and keeps the old version generating for existing worlds. Value noise looks slightly blockier than gradient noise; acceptable for the first terrain.

## Alternatives considered

- `noise` crate: floating point and transcendental functions; output not guaranteed identical across targets or versions.
- `fastnoise-lite`: same floating-point concern; its output is not part of its compatibility promise.
- Floats with a software math library (`libm`): removes the platform library but still couples worlds to a float pipeline and a dependency's exact operations; integers are simpler to reason about.
