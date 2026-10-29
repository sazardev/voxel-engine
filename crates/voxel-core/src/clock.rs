/// Port: time as the core sees it. Bevy's `Time` is the production adapter; tests use `FixedClock`.
pub trait Clock {
    /// Seconds elapsed since the game started.
    fn seconds_since_start(&self) -> f64;
}

/// A clock that always reports the same instant.
#[derive(Debug, Clone, Copy)]
pub struct FixedClock(pub f64);

impl Clock for FixedClock {
    fn seconds_since_start(&self) -> f64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fixed_clock_never_advances() {
        let clock = FixedClock(12.5);

        assert!((clock.seconds_since_start() - 12.5).abs() < f64::EPSILON);
    }
}
