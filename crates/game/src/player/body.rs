use bevy::prelude::*;

use super::kinematics::BodyState;

/// Marks the player entity.
#[derive(Debug, Component)]
pub struct Player;

/// The player's body: the state of the last fixed step and of the one before, so the camera
/// can interpolate between them at any frame rate.
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct Body {
    pub state: BodyState,
    pub previous_feet: Vec3,
}

impl Body {
    #[must_use]
    pub const fn at(feet: Vec3) -> Self {
        Self {
            state: BodyState {
                feet,
                velocity: Vec3::ZERO,
                on_ground: false,
            },
            previous_feet: feet,
        }
    }

    /// Feet position between the last two steps; `alpha` is how far the frame is into the next.
    #[must_use]
    pub fn interpolated_feet(&self, alpha: f32) -> Vec3 {
        self.previous_feet
            .lerp(self.state.feet, alpha.clamp(0.0, 1.0))
    }
}

/// Where the player looks, in radians. Yaw turns around +y (0 faces -z); pitch is clamped
/// to ±89° so the view never flips.
#[derive(Debug, Clone, Copy, PartialEq, Default, Component)]
pub struct Look {
    pub yaw: f32,
    pub pitch: f32,
}

impl Look {
    pub const MAX_PITCH: f32 = 89.0_f32.to_radians();

    /// Turns by the given angles, keeping pitch within its limits.
    pub fn turn(&mut self, yaw: f32, pitch: f32) {
        self.yaw = (self.yaw + yaw).rem_euclid(std::f32::consts::TAU);
        self.pitch = (self.pitch + pitch).clamp(-Self::MAX_PITCH, Self::MAX_PITCH);
    }

    #[must_use]
    pub fn rotation(&self) -> Quat {
        Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pitch_stops_short_of_straight_up_and_down() {
        let mut look = Look::default();

        look.turn(0.0, 10.0);
        assert!((look.pitch - Look::MAX_PITCH).abs() < 1e-6);
        look.turn(0.0, -20.0);
        assert!((look.pitch + Look::MAX_PITCH).abs() < 1e-6);
    }

    #[test]
    fn yaw_zero_looks_down_negative_z() {
        let forward = Look::default().rotation() * Vec3::NEG_Z;

        assert!(forward.abs_diff_eq(Vec3::NEG_Z, 1e-6));
    }

    #[test]
    fn interpolates_between_the_last_two_steps() {
        let mut body = Body::at(Vec3::ZERO);
        body.state.feet = Vec3::new(2.0, 0.0, 0.0);

        assert_eq!(body.interpolated_feet(0.25), Vec3::new(0.5, 0.0, 0.0));
    }
}
