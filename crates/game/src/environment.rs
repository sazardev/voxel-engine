//! The single source of truth for sun, light, fog, sky and grading. Gameplay changes time and
//! weather only through this state, never by setting a light or fog value directly.

mod atmosphere;
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
            .init_resource::<ClearColor>()
            .add_systems(Startup, sun::spawn_sun)
            .add_systems(
                Update,
                (
                    sun::follow_environment.run_if(resource_changed::<Environment>),
                    atmosphere::follow_environment,
                ),
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
    fn cameras_get_fog_and_the_sky_color_follows_the_time_of_day() {
        let mut app = App::new();
        app.add_plugins(EnvironmentPlugin);
        let camera = app.world_mut().spawn(Camera3d::default()).id();
        app.update();
        let morning_sky = app.world().resource::<ClearColor>().0;

        app.world_mut().resource_mut::<Environment>().time_of_day = 0.0;
        app.update();

        let fog = app
            .world()
            .get::<bevy::pbr::DistanceFog>(camera)
            .expect("fog on the camera");
        let night_sky = app.world().resource::<ClearColor>().0;
        assert_ne!(night_sky, morning_sky);
        assert_eq!(fog.color, night_sky);
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
