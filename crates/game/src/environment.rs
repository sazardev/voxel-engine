//! The single source of truth for sun, light, fog, sky and grading. Gameplay changes time and
//! weather only through this state, never by setting a light or fog value directly.

mod state;

use bevy::prelude::*;

pub use state::{Environment, Weather};

#[derive(Debug, Default)]
pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Environment>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_plugin_provides_the_environment_resource() {
        let mut app = App::new();
        app.add_plugins(EnvironmentPlugin);

        assert!(app.world().get_resource::<Environment>().is_some());
    }
}
