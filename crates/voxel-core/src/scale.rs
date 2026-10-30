//! How big a voxel is in the world: the one place that defines it (DESIGN.md, ADR 0015).
//! Pure code works in voxel units; only rendering, physics and the camera convert to metres.

/// World voxels per metre.
pub const VOXELS_PER_METRE: i64 = 4;
/// Edge of a world voxel (terrain, buildings), in metres.
#[expect(
    clippy::cast_precision_loss,
    reason = "VOXELS_PER_METRE is a small power of two, exact in f32"
)]
pub const VOXEL_SIZE_METRES: f32 = 1.0 / VOXELS_PER_METRE as f32;
/// Edge of a model voxel (characters, items, props), in metres: half a world voxel.
pub const MODEL_VOXEL_SIZE_METRES: f32 = VOXEL_SIZE_METRES / 2.0;
