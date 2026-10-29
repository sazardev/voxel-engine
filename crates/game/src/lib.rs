//! Composition root. Wires adapters into the core; nothing else builds infrastructure (ADR 0002).

pub mod environment;
pub mod render;

use bevy::prelude::*;

use environment::EnvironmentPlugin;

/// Adds every game feature plugin to the `App`. Window and renderer plugins stay in `main`
/// so headless tests can run this plugin alone.
#[derive(Debug, Default)]
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EnvironmentPlugin)
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
        app.add_plugins((MinimalPlugins, GamePlugin));

        app.update();
    }
}
