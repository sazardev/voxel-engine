use bevy::prelude::*;

use super::settings::DevSettings;
use crate::environment::Environment;

const TOGGLE_OVERLAY_KEY: KeyCode = KeyCode::F3;
const CYCLE_WEATHER_KEY: KeyCode = KeyCode::F6;
const FAST_FORWARD_KEY: KeyCode = KeyCode::BracketRight;
const FAST_FORWARD_SPEED: f32 = 60.0;
const DAY_LENGTH_SECONDS: f32 = 600.0;

pub fn toggle_overlay(keys: Res<ButtonInput<KeyCode>>, mut settings: ResMut<DevSettings>) {
    if keys.just_pressed(TOGGLE_OVERLAY_KEY) {
        settings.overlay_visible = !settings.overlay_visible;
    }
}

pub fn cycle_weather(keys: Res<ButtonInput<KeyCode>>, mut environment: ResMut<Environment>) {
    if keys.just_pressed(CYCLE_WEATHER_KEY) {
        environment.weather = environment.weather.next();
    }
}

pub fn fast_forward_time(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut environment: ResMut<Environment>,
) {
    if keys.pressed(FAST_FORWARD_KEY) {
        environment.advance(time.delta_secs() * FAST_FORWARD_SPEED, DAY_LENGTH_SECONDS);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app_with_keys() -> App {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<Environment>()
            .init_resource::<DevSettings>();
        app
    }

    #[test]
    fn f3_hides_the_overlay() {
        let mut app = app_with_keys();
        app.add_systems(Update, toggle_overlay);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(TOGGLE_OVERLAY_KEY);

        app.update();

        assert!(!app.world().resource::<DevSettings>().overlay_visible);
    }

    #[test]
    fn f6_changes_the_weather_through_the_environment_state() {
        let mut app = app_with_keys();
        app.add_systems(Update, cycle_weather);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(CYCLE_WEATHER_KEY);

        app.update();

        assert_ne!(
            app.world().resource::<Environment>().weather,
            Environment::default().weather
        );
    }
}
