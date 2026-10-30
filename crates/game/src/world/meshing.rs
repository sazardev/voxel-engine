//! Chunk meshes: built by bounded jobs from loaded voxels, uploaded a few per frame, despawned
//! when out of view.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use bevy::prelude::*;
use voxel_core::{CHUNK_EDGE, ChunkPos};

use super::jobs::{ChunkWorkers, Job};
use super::mesh_job::{Neighbourhood, mesh_chunk};
use super::plan::ActivePlan;
use super::voxel_colors::VoxelColors;
use super::voxels::ChunkVoxels;

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

/// Meshing jobs in flight and finished meshes waiting for their upload slot. Both together stay
/// within the in-flight budget.
#[derive(Debug, Default, Resource)]
pub struct MeshJobs {
    running: HashMap<ChunkPos, Job<Option<Mesh>>>,
    finished: VecDeque<(ChunkPos, Option<Mesh>)>,
}

impl MeshJobs {
    fn len(&self) -> usize {
        self.running.len() + self.finished.len()
    }

    fn contains(&self, pos: ChunkPos) -> bool {
        self.running.contains_key(&pos) || self.finished.iter().any(|(done, _)| *done == pos)
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
    mut jobs: ResMut<MeshJobs>,
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
    jobs.running.retain(|&pos, _| plan.0.wants_mesh(pos));
    jobs.finished.retain(|(pos, _)| plan.0.wants_mesh(*pos));
}

#[expect(
    clippy::too_many_arguments,
    reason = "a Bevy system: each parameter is a resource it reads or writes"
)]
pub(super) fn start_meshing(
    plan: Res<ActivePlan>,
    voxels: Res<ChunkVoxels>,
    colors: Res<VoxelColors>,
    workers: Res<ChunkWorkers>,
    meshed: Res<MeshedChunks>,
    mut jobs: ResMut<MeshJobs>,
) {
    let budget = workers.budget();
    let room = budget.meshing_in_flight.saturating_sub(jobs.len());
    let ready: Vec<(ChunkPos, Neighbourhood)> = plan
        .0
        .meshes()
        .iter()
        .copied()
        .filter(|&pos| !meshed.contains(pos) && !jobs.contains(pos))
        .filter_map(|pos| Some((pos, Neighbourhood::gather(&voxels, pos)?)))
        .take(room.min(budget.meshing_starts))
        .collect();
    let colors = Arc::new(colors.clone());
    for (pos, neighbourhood) in ready {
        let colors = Arc::clone(&colors);
        jobs.running.insert(
            pos,
            Job::start(*workers, move || mesh_chunk(&neighbourhood, &colors)),
        );
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "a Bevy system: each parameter is a resource it reads or writes"
)]
pub(super) fn upload_meshes(
    mut commands: Commands,
    workers: Res<ChunkWorkers>,
    material: Res<ChunkMaterial>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut meshed: ResMut<MeshedChunks>,
    mut jobs: ResMut<MeshJobs>,
) {
    let jobs = &mut *jobs;
    for (&pos, job) in &mut jobs.running {
        if let Some(mesh) = job.poll() {
            jobs.finished.push_back((pos, mesh));
        }
    }
    for (pos, _) in &jobs.finished {
        jobs.running.remove(pos);
    }
    let uploads = workers.budget().mesh_uploads.min(jobs.finished.len());
    for (pos, mesh) in jobs.finished.drain(..uploads) {
        let entity = mesh.map(|mesh| {
            commands
                .spawn((
                    ChunkMesh(pos),
                    Mesh3d(mesh_assets.add(mesh)),
                    MeshMaterial3d(material.0.clone()),
                    Transform::from_translation(chunk_origin(pos)),
                ))
                .id()
        });
        meshed.chunks.insert(pos, entity);
    }
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
