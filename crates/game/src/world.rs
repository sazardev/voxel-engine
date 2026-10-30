//! The voxel world around a focus: generated from the seed, meshed and streamed with bounded
//! work per frame (FR-1, FR-26, FR-30; docs/stack/world-streaming.md).

mod chunk_mesh;
mod meshing;
mod plan;
mod settings;
mod streaming;
mod voxel_colors;
mod voxels;

use bevy::prelude::*;
use voxel_gen::{TerrainGenerator, TerrainPalette};

pub use meshing::{ChunkMaterial, ChunkMesh, MESH_PER_FRAME, MeshedChunks};
pub use plan::{ActivePlan, StreamFocus};
pub use settings::WorldSettings;
pub use streaming::StreamPlan;
pub use voxel_colors::VoxelColors;
pub use voxels::{ChunkVoxels, GENERATE_PER_FRAME, Terrain};

/// Streams the world. Insert [`WorldSettings`] before adding it to choose the seed and radius.
#[derive(Debug, Default)]
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldSettings>()
            .init_resource::<StreamFocus>()
            .init_resource::<ChunkVoxels>()
            .init_resource::<MeshedChunks>()
            .init_resource::<VoxelColors>()
            .add_systems(Startup, (start_world, meshing::create_material))
            .add_systems(
                Update,
                (
                    plan::follow_focus,
                    voxels::drop_far_voxels,
                    meshing::despawn_far_meshes,
                    voxels::generate_voxels,
                    meshing::mesh_chunks,
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
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Mesh>()
            .init_asset::<StandardMaterial>()
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

    /// Runs frames until every planned chunk is meshed, checking the per-frame budget each time.
    fn stream_until_meshed(app: &mut App) {
        let mut before = chunk_meshes(app).len();
        for _ in 0..ENOUGH_FRAMES {
            app.update();
            let now = chunk_meshes(app).len();
            assert!(
                now <= before + MESH_PER_FRAME,
                "{now} meshes after {before}"
            );
            before = now;
            if fully_meshed(app) {
                return;
            }
        }
        panic!("the world did not finish streaming in {ENOUGH_FRAMES} frames");
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
    fn moving_the_focus_unloads_what_is_left_behind() {
        let mut app = world_app(1);
        stream_until_meshed(&mut app);

        app.world_mut().resource_mut::<StreamFocus>().0 = Vec3::new(32.0 * 10.0, 0.0, 0.0);
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
