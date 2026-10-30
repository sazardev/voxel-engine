//! Chunk meshes: built from loaded voxels within a per-frame budget, despawned when out of view.

use std::collections::HashMap;

use bevy::prelude::*;
use voxel_core::{CHUNK_EDGE, ChunkPos};
use voxel_mesh::{MeshBuffers, PaddedChunk, mesh_quads};

use super::chunk_mesh::chunk_mesh;
use super::plan::ActivePlan;
use super::voxel_colors::VoxelColors;
use super::voxels::ChunkVoxels;

/// Chunk meshes built and uploaded per frame: about 1 ms of meshing on smooth terrain plus the
/// uploads it causes, both bounded (docs/stack/world-streaming.md).
pub const MESH_PER_FRAME: usize = 4;

/// A rendered chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct ChunkMesh(pub ChunkPos);

/// The material every chunk shares; colors come from the vertices.
#[derive(Debug, Resource)]
pub struct ChunkMaterial(pub Handle<StandardMaterial>);

/// Chunks already meshed, with their entity (`None` when the chunk has no visible face).
#[derive(Debug, Default, Resource)]
pub struct MeshedChunks {
    chunks: HashMap<ChunkPos, Option<Entity>>,
}

impl MeshedChunks {
    #[must_use]
    pub fn contains(&self, pos: ChunkPos) -> bool {
        self.chunks.contains_key(&pos)
    }
}

pub(super) fn create_material(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.9,
        ..default()
    });
    commands.insert_resource(ChunkMaterial(material));
}

pub(super) fn despawn_far_meshes(
    mut commands: Commands,
    plan: Res<ActivePlan>,
    mut meshed: ResMut<MeshedChunks>,
) {
    if !plan.is_changed() {
        return;
    }
    meshed.chunks.retain(|&pos, entity| {
        let keep = plan.0.wants_mesh(pos);
        if let (false, Some(entity)) = (keep, entity) {
            commands.entity(*entity).despawn();
        }
        keep
    });
}

#[expect(
    clippy::too_many_arguments,
    reason = "a Bevy system: each parameter is a resource it reads or writes"
)]
pub(super) fn mesh_chunks(
    mut commands: Commands,
    plan: Res<ActivePlan>,
    voxels: Res<ChunkVoxels>,
    colors: Res<VoxelColors>,
    material: Res<ChunkMaterial>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut meshed: ResMut<MeshedChunks>,
) {
    let ready: Vec<(ChunkPos, MeshBuffers)> = plan
        .0
        .meshes()
        .iter()
        .copied()
        .filter(|&pos| !meshed.contains(pos))
        .filter_map(|pos| Some((pos, mesh_buffers(&voxels, pos)?)))
        .take(MESH_PER_FRAME)
        .collect();
    for (pos, buffers) in ready {
        let entity = (!buffers.is_empty()).then(|| {
            commands
                .spawn((
                    ChunkMesh(pos),
                    Mesh3d(mesh_assets.add(chunk_mesh(buffers, &colors))),
                    MeshMaterial3d(material.0.clone()),
                    Transform::from_translation(chunk_origin(pos)),
                ))
                .id()
        });
        meshed.chunks.insert(pos, entity);
    }
}

/// Meshes the chunk at `pos`, or `None` until it and all 26 neighbours are loaded.
fn mesh_buffers(voxels: &ChunkVoxels, pos: ChunkPos) -> Option<MeshBuffers> {
    let at = |[dx, dy, dz]: [i32; 3]| voxels.get(ChunkPos::new(pos.x + dx, pos.y + dy, pos.z + dz));
    let all_loaded =
        (-1..=1).all(|dz| (-1..=1).all(|dy| (-1..=1).all(|dx| at([dx, dy, dz]).is_some())));
    if !all_loaded {
        return None;
    }
    let padded = PaddedChunk::from_chunks(at([0, 0, 0])?, at);
    Some(MeshBuffers::from_quads(&mesh_quads(&padded)))
}

#[expect(
    clippy::cast_precision_loss,
    reason = "chunk coordinates times 32 stay far below f32's exact integer range in play"
)]
fn chunk_origin(pos: ChunkPos) -> Vec3 {
    let edge = CHUNK_EDGE as f32;
    Vec3::new(
        pos.x as f32 * edge,
        pos.y as f32 * edge,
        pos.z as f32 * edge,
    )
}
