use voxel_core::VoxelId;

/// Layers below the grass that are dirt before stone starts.
const DIRT_DEPTH: i64 = 3;

/// The materials the ground is made of, and how they layer from the surface down.
///
/// The default ids are placeholders until the material registry (FR-28) assigns them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerrainPalette {
    pub grass: VoxelId,
    pub dirt: VoxelId,
    pub stone: VoxelId,
}

impl TerrainPalette {
    /// The voxel `depth` layers below the surface; negative depths are above ground.
    #[must_use]
    pub const fn at_depth(&self, depth: i64) -> VoxelId {
        match depth {
            ..0 => VoxelId::AIR,
            0 => self.grass,
            1..=DIRT_DEPTH => self.dirt,
            _ => self.stone,
        }
    }
}

impl Default for TerrainPalette {
    fn default() -> Self {
        Self {
            grass: VoxelId(1),
            dirt: VoxelId(2),
            stone: VoxelId(3),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layers_grass_over_dirt_over_stone() {
        let palette = TerrainPalette::default();

        assert_eq!(palette.at_depth(-1), VoxelId::AIR);
        assert_eq!(palette.at_depth(0), palette.grass);
        assert_eq!(palette.at_depth(DIRT_DEPTH), palette.dirt);
        assert_eq!(palette.at_depth(DIRT_DEPTH + 1), palette.stone);
    }
}
