use bevy::prelude::Resource;
use voxel_core::{CHUNK_EDGE, VOXEL_SIZE_METRES};
use voxel_gen::WorldSeed;

/// Seed of the world shown until new-game and save flows choose one (FR-29).
const DEFAULT_SEED: WorldSeed = WorldSeed(0x5EED_0000_0000_0001);
/// Detailed view distance in chunks of 8 m (ADR 0015). Per-tier memory budgets will set it
/// (docs/stack/platform-and-tiers.md).
const DEFAULT_VIEW_RADIUS: u32 = 12;
/// Coarser rings beyond it, each reaching twice as far (ADR 0016): 96 m, 192 m, 384 m.
const DEFAULT_LOD_LEVELS: u8 = 2;

/// What world to stream and how far.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource)]
pub struct WorldSettings {
    pub seed: WorldSeed,
    /// Radius, in chunks, of the area drawn with full voxel detail around the focus.
    pub view_radius: u32,
    /// Rings of coarser terrain beyond it; each doubles the reach (ADR 0016).
    pub lod_levels: u8,
}

impl WorldSettings {
    /// How far the streamed world reaches from the focus, coarse rings included, in metres.
    #[must_use]
    pub fn view_distance_metres(&self) -> f32 {
        #[expect(
            clippy::cast_precision_loss,
            reason = "view radii are a few dozen chunks, exact in f32"
        )]
        let chunks = (self.view_radius << self.lod_levels) as f32;
        chunks * CHUNK_EDGE_METRES
    }
}

#[expect(clippy::cast_precision_loss, reason = "CHUNK_EDGE is 32, exact in f32")]
const CHUNK_EDGE_METRES: f32 = CHUNK_EDGE as f32 * VOXEL_SIZE_METRES;

impl Default for WorldSettings {
    fn default() -> Self {
        Self {
            seed: DEFAULT_SEED,
            view_radius: DEFAULT_VIEW_RADIUS,
            lod_levels: DEFAULT_LOD_LEVELS,
        }
    }
}
