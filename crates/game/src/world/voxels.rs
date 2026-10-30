//! Voxel data of the chunks around the focus: generated within a per-frame budget, dropped
//! when the focus moves away.

use std::collections::HashMap;
use std::sync::Arc;

use bevy::prelude::*;
use voxel_core::{Chunk, ChunkPos};
use voxel_gen::TerrainGenerator;

use super::plan::ActivePlan;

/// Chunks generated per frame. About 1 ms of main-thread work in a release build; this is also
/// the single-thread path the web needs (docs/stack/platform-and-tiers.md).
pub const GENERATE_PER_FRAME: usize = 8;

/// The world's terrain generator, built from the world settings at startup.
#[derive(Debug, Resource)]
pub struct Terrain(pub TerrainGenerator);

/// Loaded voxel data. Bounded by the active plan: chunks outside it are dropped.
#[derive(Debug, Default, Resource)]
pub struct ChunkVoxels {
    chunks: HashMap<ChunkPos, Arc<Chunk>>,
}

impl ChunkVoxels {
    #[must_use]
    pub fn get(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos).map(Arc::as_ref)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
}

pub(super) fn drop_far_voxels(plan: Res<ActivePlan>, mut voxels: ResMut<ChunkVoxels>) {
    if plan.is_changed() {
        voxels.chunks.retain(|&pos, _| plan.0.wants_voxels(pos));
    }
}

pub(super) fn generate_voxels(
    plan: Res<ActivePlan>,
    terrain: Res<Terrain>,
    mut voxels: ResMut<ChunkVoxels>,
) {
    let missing: Vec<ChunkPos> = plan
        .0
        .voxels()
        .iter()
        .copied()
        .filter(|pos| !voxels.chunks.contains_key(pos))
        .take(GENERATE_PER_FRAME)
        .collect();
    for pos in missing {
        voxels
            .chunks
            .insert(pos, Arc::new(terrain.0.generate_chunk(pos)));
    }
}
