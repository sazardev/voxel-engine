//! Sun, moon and ambient light as a pure function of the `Environment` (DESIGN.md palette).

use std::f32::consts::PI;

use bevy::color::{Color, Mix, Srgba};
use bevy::math::{FloatExt, Vec3};

use super::state::Environment;

/// Fractions of the day when the sun rises and sets. A long day suits play better than realism.
const SUNRISE: f32 = 0.2;
const SUNSET: f32 = 0.8;
/// Sun elevation (sine of the angle) above which it is full day; below zero it is night.
const FULL_DAY_ELEVATION: f32 = 0.2;

const SUN_COLOR: Srgba = Srgba::rgb(1.0, 0.945, 0.788); // #FFF1C9
const MOON_COLOR: Srgba = Srgba::rgb(0.561, 0.659, 1.0); // #8FA8FF
const DAY_AMBIENT: Srgba = Srgba::rgb(0.624, 0.718, 0.910); // #9FB7E8
const NIGHT_AMBIENT: Srgba = Srgba::rgb(0.153, 0.196, 0.310); // #27324F

const SUN_ILLUMINANCE_LUX: f32 = 10_000.0;
const MOON_ILLUMINANCE_LUX: f32 = 400.0;
const DAY_AMBIENT_BRIGHTNESS: f32 = 600.0;
const NIGHT_AMBIENT_BRIGHTNESS: f32 = 120.0;

/// The light the world gets at one moment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Daylight {
    /// Unit vector from the ground toward the main light (sun by day, moon by night).
    pub toward_light: Vec3,
    pub light_color: Color,
    pub light_illuminance_lux: f32,
    pub ambient_color: Color,
    pub ambient_brightness: f32,
    /// Sky at the horizon: the clear color and the fog color (DESIGN.md `distance-fog`). The
    /// palette's `ambient` ramp doubles as the sky until the palette names one.
    pub sky_color: Color,
}

/// How much of the day it is: 0 at night, 1 in full day, blended around sunrise and sunset.
fn day_factor(sun_elevation: f32) -> f32 {
    (sun_elevation / FULL_DAY_ELEVATION).clamp(0.0, 1.0)
}

/// The sun's path: rises in the east (+x), peaks at noon, sets in the west, tilted to the south.
fn toward_sun(time_of_day: f32) -> Vec3 {
    let arc = (time_of_day - SUNRISE) / (SUNSET - SUNRISE) * PI;
    Vec3::new(arc.cos(), arc.sin(), -0.35).normalize()
}

#[must_use]
pub fn daylight(environment: &Environment) -> Daylight {
    let sun = toward_sun(environment.time_of_day);
    let day = day_factor(sun.y);
    // At night the moon takes the sun's place on the opposite side of the sky.
    let toward_light = if sun.y >= 0.0 { sun } else { -sun };
    Daylight {
        toward_light,
        light_color: MOON_COLOR.mix(&SUN_COLOR, day).into(),
        light_illuminance_lux: MOON_ILLUMINANCE_LUX.lerp(SUN_ILLUMINANCE_LUX, day),
        ambient_color: NIGHT_AMBIENT.mix(&DAY_AMBIENT, day).into(),
        ambient_brightness: NIGHT_AMBIENT_BRIGHTNESS.lerp(DAY_AMBIENT_BRIGHTNESS, day),
        sky_color: NIGHT_AMBIENT.mix(&DAY_AMBIENT, day).into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(time_of_day: f32) -> Daylight {
        daylight(&Environment {
            time_of_day,
            ..Environment::default()
        })
    }

    #[test]
    fn the_sun_is_overhead_at_noon_and_in_full_strength() {
        let noon = at(0.5);

        assert!(noon.toward_light.y > 0.9);
        assert!((noon.light_illuminance_lux - SUN_ILLUMINANCE_LUX).abs() < 1.0);
    }

    #[test]
    fn the_light_always_comes_from_above_the_horizon() {
        for step in 0..100 {
            #[expect(clippy::cast_precision_loss, reason = "step is below 100")]
            let time = step as f32 / 100.0;
            assert!(at(time).toward_light.y >= 0.0, "time {time}");
        }
    }

    #[test]
    fn midnight_is_moonlit_and_darker_than_noon() {
        let (noon, midnight) = (at(0.5), at(0.0));

        assert!(midnight.light_illuminance_lux < noon.light_illuminance_lux);
        assert!(midnight.ambient_brightness < noon.ambient_brightness);
        assert_eq!(midnight.light_color, Color::from(MOON_COLOR));
    }

    #[test]
    fn the_sky_darkens_at_night() {
        let (noon, midnight) = (at(0.5).sky_color.to_linear(), at(0.0).sky_color.to_linear());

        assert!(midnight.blue < noon.blue && midnight.green < noon.green);
    }

    #[test]
    fn the_game_starts_in_daylight() {
        assert!(day_factor(at(Environment::default().time_of_day).toward_light.y) > 0.99);
    }
}
