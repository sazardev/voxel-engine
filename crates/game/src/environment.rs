//! The single source of truth for sun, light, fog, sky and grading. Gameplay changes time and
//! weather only through this state, never by setting a light or fog value directly.

mod daylight;
mod state;
mod sun;

use bevy::light::GlobalAmbientLight;
use bevy::prelude::*;

pub use daylight::{Daylight, daylight};
pub use state::{Environment, Weather};
pub use sun::Sun;

#[derive(Debug, Default)]
pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Environment>()
            .init_resource::<GlobalAmbientLight>()
            .add_systems(Startup, sun::spawn_sun)
            .add_systems(
                Update,
                sun::follow_environment.run_if(resource_changed::<Environment>),
            );
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

    #[test]
    fn one_shadow_casting_sun_follows_the_time_of_day() {
        let mut app = App::new();
        app.add_plugins(EnvironmentPlugin);
        app.update();
        let mut suns = app
            .world_mut()
            .query::<(&DirectionalLight, &Transform, &Sun)>();
        let morning = {
            let (light, transform, _) = suns.single(app.world()).expect("one sun");
            assert!(light.shadow_maps_enabled);
            transform.rotation
        };

        app.world_mut().resource_mut::<Environment>().time_of_day = 0.5;
        app.update();

        let (_, noon, _) = suns.single(app.world()).expect("one sun");
        assert_ne!(noon.rotation, morning);
    }
}
