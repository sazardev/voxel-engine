//! The voxel world around a focus: generated from the seed, meshed and streamed with bounded
//! work per frame (FR-1, FR-26, FR-30; docs/stack/world-streaming.md).

mod chunk_mesh;
mod jobs;
mod mesh_job;
mod meshing;
mod plan;
mod settings;
mod streaming;
mod voxel_colors;
mod voxel_tint;
mod voxels;

use bevy::prelude::*;
use voxel_gen::{TerrainGenerator, TerrainPalette};

pub use jobs::{ChunkWorkers, StreamBudget};
pub use meshing::{ChunkMaterial, ChunkMesh, MeshJobs, MeshedChunks};
pub use plan::{ActivePlan, StreamFocus};
pub use settings::WorldSettings;
pub use streaming::StreamPlan;
pub use voxel_colors::VoxelColors;
pub use voxels::{ChunkVoxels, GenerationJobs, Terrain};

/// Streams the world. Insert [`WorldSettings`] before adding it to choose the seed and radius,
/// and [`ChunkWorkers`] to force where chunk work runs (by default: this device's threads).
#[derive(Debug, Default)]
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<ChunkWorkers>() {
            app.insert_resource(ChunkWorkers::for_this_device());
        }
        app.init_resource::<WorldSettings>()
            .init_resource::<StreamFocus>()
            .init_resource::<ChunkVoxels>()
            .init_resource::<MeshedChunks>()
            .init_resource::<GenerationJobs>()
            .init_resource::<MeshJobs>()
            .init_resource::<VoxelColors>()
            .add_systems(Startup, (start_world, meshing::create_material))
            .add_systems(
                Update,
                (
                    plan::follow_focus,
                    voxels::drop_far_voxels,
                    meshing::despawn_far_meshes,
                    voxels::finish_generation,
                    voxels::start_generation,
                    meshing::start_meshing,
                    meshing::upload_meshes,
                )
                    .chain()
                    .run_if(resource_exists::<ActivePlan>),
            );
    }
}

fn start_world(mut commands: Commands, settings: Res<WorldSettings>, focus: Res<StreamFocus>) {
    let terrain = TerrainGenerator::new(settings.seed, TerrainPalette::default());
    commands.insert_resource(Terrain(terrain));
    commands.insert_resource(plan::initial_plan(&settings, *focus));
}

#[cfg(test)]
mod tests {
    use bevy::asset::AssetPlugin;
    use voxel_core::ChunkPos;

    use super::*;

    const ENOUGH_FRAMES: usize = 400;

    fn world_app(view_radius: u32) -> App {
        world_app_with(view_radius, ChunkWorkers::Inline)
    }

    fn world_app_with(view_radius: u32, workers: ChunkWorkers) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Mesh>()
            .init_asset::<StandardMaterial>()
            .init_asset::<Image>()
            .insert_resource(workers)
            .insert_resource(WorldSettings {
                view_radius,
                ..WorldSettings::default()
            })
            .add_plugins(WorldPlugin);
        app
    }

    fn chunk_meshes(app: &mut App) -> Vec<ChunkPos> {
        let mut query = app.world_mut().query::<&ChunkMesh>();
        query.iter(app.world()).map(|mesh| mesh.0).collect()
    }

    fn fully_meshed(app: &App) -> bool {
        let plan = &app.world().resource::<ActivePlan>().0;
        let meshed = app.world().resource::<MeshedChunks>();
        plan.meshes().iter().all(|&pos| meshed.contains(pos))
    }

    /// Runs frames until every planned chunk is meshed, checking the per-frame budgets each time.
    fn stream_until_meshed(app: &mut App) {
        stream_for(app, ENOUGH_FRAMES);
    }

    fn stream_for(app: &mut App, frames: usize) {
        let budget = app.world().resource::<ChunkWorkers>().budget();
        let mut before = chunk_meshes(app).len();
        for _ in 0..frames {
            app.update();
            let now = chunk_meshes(app).len();
            assert!(
                now <= before + budget.mesh_uploads,
                "{now} meshes after {before}"
            );
            let in_flight = app.world().resource::<GenerationJobs>().len();
            assert!(
                in_flight <= budget.generation_in_flight,
                "{in_flight} generation jobs"
            );
            before = now;
            if fully_meshed(app) {
                return;
            }
        }
        panic!("the world did not finish streaming in {frames} frames");
    }

    #[test]
    fn streams_in_the_surface_around_the_focus_within_the_frame_budget() {
        let mut app = world_app(2);

        stream_until_meshed(&mut app);

        let meshes = chunk_meshes(&mut app);
        assert_ne!(meshes, Vec::<ChunkPos>::new());
        let plan = &app.world().resource::<ActivePlan>().0;
        assert!(meshes.iter().all(|&pos| plan.wants_mesh(pos)));
        assert!(meshes.iter().any(|pos| pos.x == 0 && pos.z == 0));
    }

    #[test]
    fn background_workers_stream_the_same_chunks_as_inline_ones() {
        let mut inline = world_app(1);
        let mut background = world_app_with(1, ChunkWorkers::Background);

        stream_until_meshed(&mut inline);
        // Background jobs finish on other threads; frames are cheap, so allow many of them.
        stream_for(&mut background, 1_000_000);

        let mut expected = chunk_meshes(&mut inline);
        let mut streamed = chunk_meshes(&mut background);
        expected.sort_by_key(|pos| (pos.x, pos.y, pos.z));
        streamed.sort_by_key(|pos| (pos.x, pos.y, pos.z));
        assert_eq!(streamed, expected);
    }

    #[test]
    fn moving_the_focus_unloads_what_is_left_behind() {
        let mut app = world_app(1);
        stream_until_meshed(&mut app);

        // Ten chunks of 8 m along x.
        app.world_mut().resource_mut::<StreamFocus>().0 = Vec3::new(8.0 * 10.0 + 4.0, 0.0, 0.0);
        stream_until_meshed(&mut app);

        let meshes = chunk_meshes(&mut app);
        assert!(
            meshes.iter().all(|pos| (9..=11).contains(&pos.x)),
            "{meshes:?}"
        );
        let plan = &app.world().resource::<ActivePlan>().0;
        let voxels = app.world().resource::<ChunkVoxels>();
        assert_eq!(voxels.len(), plan.voxels().len());
    }

    #[test]
    fn the_same_seed_streams_the_same_world() {
        let mut first = world_app(1);
        let mut second = world_app(1);

        stream_until_meshed(&mut first);
        stream_until_meshed(&mut second);

        let pos = ChunkPos::new(0, 1, 0);
        let chunk = |app: &App| app.world().resource::<ChunkVoxels>().get(pos).cloned();
        assert_eq!(chunk(&first), chunk(&second));
    }
}
