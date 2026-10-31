//! Between the running world and its save: what is captured, and how a save is applied.

use bevy::prelude::*;
use voxel_gen::{GENERATOR_VERSION, WorldSeed};

use super::format::{EnvironmentSave, PlayerSave, SaveError, SavedWeather, WorldSave};
use crate::environment::{Environment, Weather};
use crate::player::{Body, Look, PlayerStart};
use crate::world::WorldSettings;

/// The state worth saving now; `None` until the player exists.
#[must_use]
pub fn capture(
    settings: &WorldSettings,
    environment: &Environment,
    player: Option<(&Body, &Look)>,
) -> Option<WorldSave> {
    let (body, look) = player?;
    Some(WorldSave {
        seed: settings.seed.0,
        generator_version: GENERATOR_VERSION,
        player: PlayerSave {
            feet: body.state.feet.to_array(),
            yaw: look.yaw,
            pitch: look.pitch,
        },
        environment: EnvironmentSave {
            time_of_day: environment.time_of_day,
            weather: match environment.weather {
                Weather::Clear => SavedWeather::Clear,
                Weather::Rain => SavedWeather::Rain,
                Weather::Storm => SavedWeather::Storm,
            },
        },
        edited_chunks: Vec::new(),
    })
}

/// Puts a loaded save into the world before it starts: seed, time, weather and where the
/// player reappears.
///
/// # Errors
/// Returns [`SaveError::OtherGenerator`] when the save's terrain cannot be rebuilt by this build;
/// nothing is changed then.
pub fn apply(
    save: &WorldSave,
    settings: &mut WorldSettings,
    environment: &mut Environment,
) -> Result<PlayerStart, SaveError> {
    if save.generator_version != GENERATOR_VERSION {
        return Err(SaveError::OtherGenerator {
            found: save.generator_version,
            supported: GENERATOR_VERSION,
        });
    }
    settings.seed = WorldSeed(save.seed);
    environment.time_of_day = save.environment.time_of_day;
    environment.weather = match save.environment.weather {
        SavedWeather::Clear => Weather::Clear,
        SavedWeather::Rain => Weather::Rain,
        SavedWeather::Storm => Weather::Storm,
    };
    Ok(PlayerStart {
        feet: Vec3::from_array(save.player.feet),
        look: Look {
            yaw: save.player.yaw,
            pitch: save.player.pitch,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::save::format::tests::sample;

    #[test]
    fn applying_a_save_restores_seed_time_weather_and_player() {
        let (mut settings, mut environment) = (WorldSettings::default(), Environment::default());
        let save = WorldSave {
            generator_version: GENERATOR_VERSION,
            ..sample()
        };

        let start = apply(&save, &mut settings, &mut environment).expect("same generator");

        assert_eq!(settings.seed, WorldSeed(save.seed));
        assert!((environment.time_of_day - 0.6).abs() < f32::EPSILON);
        assert_eq!(environment.weather, Weather::Rain);
        assert_eq!(start.feet, Vec3::new(1.5, 64.25, -3.0));
    }

    #[test]
    fn a_save_from_another_generator_changes_nothing() {
        let (mut settings, mut environment) = (WorldSettings::default(), Environment::default());
        let save = WorldSave {
            generator_version: GENERATOR_VERSION + 1,
            ..sample()
        };

        let result = apply(&save, &mut settings, &mut environment);

        assert!(matches!(result, Err(SaveError::OtherGenerator { .. })));
        assert_eq!(settings, WorldSettings::default());
        assert_eq!(environment, Environment::default());
    }
}
