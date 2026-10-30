use voxel_core::{CHUNK_EDGE, Chunk, ChunkPos};

use crate::heightmap::{MAX_SURFACE_HEIGHT, MIN_SURFACE_HEIGHT, column_heights, surface_height};
use crate::palette::TerrainPalette;
use crate::seed::WorldSeed;

/// Version of the generation algorithm. Any change to generated voxels bumps it, because saves
/// store only edits on top of what the generator produces (ADR 0010).
pub const GENERATOR_VERSION: u32 = 1;

#[expect(
    clippy::cast_possible_wrap,
    reason = "CHUNK_EDGE is 32, checked by the assertion below"
)]
const EDGE: i64 = CHUNK_EDGE as i64;
#[expect(
    clippy::cast_possible_truncation,
    reason = "EDGE is 32 and fits every usize"
)]
const _: () = assert!(EDGE as usize == CHUNK_EDGE);

/// Generates terrain chunks from a seed. Pure: a chunk depends only on the seed, the palette
/// and its position, so chunks can be generated in any order and still join seamlessly (FR-30).
#[derive(Debug, Clone)]
pub struct TerrainGenerator {
    seed: WorldSeed,
    palette: TerrainPalette,
}

impl TerrainGenerator {
    #[must_use]
    pub const fn new(seed: WorldSeed, palette: TerrainPalette) -> Self {
        Self { seed, palette }
    }

    /// World height of the topmost solid voxel of column (`x`, `z`), in voxels.
    #[must_use]
    pub fn surface_height(&self, x: i64, z: i64) -> i64 {
        surface_height(self.seed, x, z)
    }

    #[must_use]
    pub fn generate_chunk(&self, pos: ChunkPos) -> Chunk {
        let bottom = i64::from(pos.y) * EDGE;
        if bottom > MAX_SURFACE_HEIGHT {
            return Chunk::empty();
        }
        // The shallowest possible depth of this chunk's top voxel: deep enough means all stone.
        let top = bottom + EDGE - 1;
        if self.palette.at_depth(MIN_SURFACE_HEIGHT - top) == self.palette.stone {
            return Chunk::from_fn(|_| self.palette.stone);
        }
        let heights = column_heights(
            self.seed,
            (i64::from(pos.x) * EDGE, i64::from(pos.z) * EDGE),
            EDGE,
        );
        Chunk::from_fn(|local| {
            let column = usize::from(local.z()) * CHUNK_EDGE + usize::from(local.x());
            self.palette
                .at_depth(heights[column] - (bottom + i64::from(local.y())))
        })
    }
}
