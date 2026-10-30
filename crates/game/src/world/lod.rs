//! Coarse patches beyond the detailed area: meshed by bounded jobs, uploaded a few per frame,
//! despawned when the layout no longer wants them (ADR 0016).

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use bevy::prelude::*;

use super::jobs::{ChunkWorkers, Job};
use super::lod_layout::LodPatch;
use super::lod_mesh::lod_mesh;
use super::meshing::ChunkMaterial;
use super::plan::ActiveLod;
use super::voxel_colors::VoxelColors;
use super::voxels::Terrain;

type PatchMesh = Option<(Mesh, Transform)>;

/// A rendered coarse patch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct LodMesh(pub LodPatch);

/// Patches already meshed, with their entity (`None` when the patch has no faces).
#[derive(Debug, Default, Resource)]
pub struct LodMeshes {
    patches: HashMap<LodPatch, Option<Entity>>,
}

impl LodMeshes {
    #[must_use]
    pub fn contains(&self, patch: LodPatch) -> bool {
        self.patches.contains_key(&patch)
    }
}

/// Patch jobs in flight and finished meshes waiting for an upload slot.
#[derive(Debug, Default, Resource)]
pub struct LodJobs {
    running: HashMap<LodPatch, Job<PatchMesh>>,
    finished: VecDeque<(LodPatch, PatchMesh)>,
}

impl LodJobs {
    fn len(&self) -> usize {
        self.running.len() + self.finished.len()
    }

    fn contains(&self, patch: LodPatch) -> bool {
        self.running.contains_key(&patch) || self.finished.iter().any(|(done, _)| *done == patch)
    }
}

pub(super) fn despawn_far_patches(
    mut commands: Commands,
    lod: Res<ActiveLod>,
    mut meshes: ResMut<LodMeshes>,
    mut jobs: ResMut<LodJobs>,
) {
    if !lod.is_changed() {
        return;
    }
    meshes.patches.retain(|&patch, entity| {
        let keep = lod.wants(patch);
        if let (false, Some(entity)) = (keep, entity) {
            commands.entity(*entity).despawn();
        }
        keep
    });
    jobs.running.retain(|&patch, _| lod.wants(patch));
    jobs.finished.retain(|(patch, _)| lod.wants(*patch));
}

#[expect(
    clippy::too_many_arguments,
    reason = "a Bevy system: each parameter is a resource it reads or writes"
)]
pub(super) fn start_patches(
    lod: Res<ActiveLod>,
    terrain: Res<Terrain>,
    colors: Res<VoxelColors>,
    workers: Res<ChunkWorkers>,
    meshes: Res<LodMeshes>,
    mut jobs: ResMut<LodJobs>,
) {
    let budget = workers.budget();
    let room = budget
        .lod_in_flight
        .saturating_sub(jobs.len())
        .min(budget.lod_starts);
    let missing: Vec<LodPatch> = lod
        .layout
        .patches
        .iter()
        .copied()
        .filter(|&patch| !meshes.contains(patch) && !jobs.contains(patch))
        .take(room)
        .collect();
    let colors = Arc::new(colors.clone());
    for patch in missing {
        let (terrain, colors) = (terrain.0.clone(), Arc::clone(&colors));
        jobs.running.insert(
            patch,
            Job::start(*workers, move || lod_mesh(&terrain, &colors, patch)),
        );
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "a Bevy system: each parameter is a resource it reads or writes"
)]
pub(super) fn upload_patches(
    mut commands: Commands,
    workers: Res<ChunkWorkers>,
    material: Res<ChunkMaterial>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut meshes: ResMut<LodMeshes>,
    mut jobs: ResMut<LodJobs>,
) {
    let jobs = &mut *jobs;
    for (&patch, job) in &mut jobs.running {
        if let Some(mesh) = job.poll() {
            jobs.finished.push_back((patch, mesh));
        }
    }
    for (patch, _) in &jobs.finished {
        jobs.running.remove(patch);
    }
    let uploads = workers.budget().lod_uploads.min(jobs.finished.len());
    for (patch, mesh) in jobs.finished.drain(..uploads) {
        let entity = mesh.map(|(mesh, transform)| {
            commands
                .spawn((
                    LodMesh(patch),
                    Mesh3d(mesh_assets.add(mesh)),
                    MeshMaterial3d(material.0.clone()),
                    transform,
                ))
                .id()
        });
        meshes.patches.insert(patch, entity);
    }
}
