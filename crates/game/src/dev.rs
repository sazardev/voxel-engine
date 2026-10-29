//! Developer tools. Compiled only with `--features dev`; player builds contain none of this.
//! Dev code observes and nudges state through the same resources gameplay uses (never around them)
//! and is registered by `main`, never by `GamePlugin` (see `docs/stack/dev-mode.md`).

mod controls;
mod overlay;
mod settings;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;

pub use settings::DevSettings;

#[derive(Debug, Default)]
pub struct DevPlugin;

impl Plugin for DevPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .init_resource::<DevSettings>()
            .add_systems(Startup, overlay::spawn_overlay)
            .add_systems(
                Update,
                (
                    controls::toggle_overlay,
                    controls::cycle_weather,
                    controls::fast_forward_time,
                    overlay::update_overlay,
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::EnvironmentPlugin;

    #[test]
    fn the_dev_plugin_runs_a_frame_without_a_window() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>().add_plugins((
            MinimalPlugins,
            EnvironmentPlugin,
            DevPlugin,
        ));

        app.update();

        assert!(app.world().get_resource::<DevSettings>().is_some());
    }
}
