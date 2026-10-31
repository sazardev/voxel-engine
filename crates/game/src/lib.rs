//! Composition root. Wires adapters into the core; nothing else builds infrastructure (ADR 0002).

pub mod camera;
#[cfg(feature = "dev")]
pub mod dev;
pub mod environment;
pub mod input;
pub mod player;
pub mod render;
mod run_mode;
pub mod save;
pub mod world;

use bevy::prelude::*;

use camera::CameraPlugin;
use environment::EnvironmentPlugin;
use input::InputPlugin;
use player::PlayerPlugin;
pub use run_mode::RunMode;
use save::SavePlugin;
use world::WorldPlugin;

/// Adds every game feature plugin to the `App`. Window and renderer plugins stay in `main`
/// so headless tests can run this plugin alone.
#[derive(Debug, Default)]
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            EnvironmentPlugin,
            InputPlugin,
            WorldPlugin,
            PlayerPlugin,
            CameraPlugin,
            SavePlugin,
        ))
        .add_systems(Startup, announce_startup);
    }
}

fn announce_startup() {
    info!("voxel game started");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_plugin_runs_a_frame_without_a_window() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::asset::AssetPlugin::default(),
            bevy::input::InputPlugin,
        ))
        .init_asset::<Mesh>()
        .init_asset::<StandardMaterial>()
        .init_asset::<Image>()
        .add_plugins(GamePlugin);

        app.update();
    }
}
