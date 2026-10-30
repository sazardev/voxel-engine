use bevy::prelude::Resource;
use voxel_gen::WorldSeed;

/// Seed of the world shown until new-game and save flows choose one (FR-29).
const DEFAULT_SEED: WorldSeed = WorldSeed(0x5EED_0000_0000_0001);
/// View distance in chunks of 8 m (ADR 0015). Per-tier memory budgets will set it
/// (docs/stack/platform-and-tiers.md).
const DEFAULT_VIEW_RADIUS: u32 = 12;

/// What world to stream and how far.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource)]
pub struct WorldSettings {
    pub seed: WorldSeed,
    /// Radius, in chunks, of the meshed area around the focus.
    pub view_radius: u32,
}

impl Default for WorldSettings {
    fn default() -> Self {
        Self {
            seed: DEFAULT_SEED,
            view_radius: DEFAULT_VIEW_RADIUS,
        }
    }
}
