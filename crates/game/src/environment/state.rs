use bevy::prelude::Resource;

/// Current weather. Biome and interior overrides join this state when the world exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Weather {
    #[default]
    Clear,
    Rain,
    Storm,
}

/// Time and weather of the world. `time_of_day` is a fraction of a day in `0.0..1.0`.
#[derive(Debug, Clone, Copy, PartialEq, Resource)]
pub struct Environment {
    pub time_of_day: f32,
    pub weather: Weather,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            time_of_day: 0.25,
            weather: Weather::Clear,
        }
    }
}

impl Environment {
    /// Moves the clock forward and wraps at midnight.
    pub fn advance(&mut self, elapsed_seconds: f32, day_length_seconds: f32) {
        self.time_of_day =
            (self.time_of_day + elapsed_seconds / day_length_seconds).rem_euclid(1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advancing_past_midnight_wraps_to_the_next_day() {
        let mut environment = Environment {
            time_of_day: 0.9,
            ..Environment::default()
        };

        environment.advance(60.0, 600.0);

        assert!((environment.time_of_day - 0.0).abs() < 1e-6);
    }
}
