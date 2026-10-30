//! How the player's body moves in one fixed step: a pure rule over the voxel grid (FR-22, FR-57).

use bevy::math::{Vec2, Vec3};
use voxel_core::Aabb;

/// Collision box size, in metres (one voxel is one metre).
pub const BODY_HALF_WIDTH: f32 = 0.3;
pub const BODY_HEIGHT: f32 = 1.8;
const WALK_SPEED: f32 = 4.3;
const SPRINT_SPEED: f32 = 6.5;
const GRAVITY: f32 = 28.0;
/// Reaches about 1.25 m: one block up, with room to spare.
const JUMP_SPEED: f32 = 8.4;
const TERMINAL_SPEED: f32 = 60.0;

/// The physical state of the player, advanced only in fixed steps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyState {
    /// Center of the bottom of the collision box.
    pub feet: Vec3,
    pub velocity: Vec3,
    pub on_ground: bool,
}

/// What the player asks of the body this step.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MoveIntent {
    /// x is right, y is forward, relative to `yaw`; length at most 1.
    pub direction: Vec2,
    /// Radians around +y; 0 faces -z.
    pub yaw: f32,
    pub sprint: bool,
    pub jump: bool,
}

/// Advances the body by `dt` seconds. Horizontal speed follows the intent at once (no
/// acceleration, so control responds immediately); gravity and jumps act on vertical speed.
#[must_use]
pub fn step(
    body: BodyState,
    intent: MoveIntent,
    dt: f32,
    is_solid: impl Fn([i64; 3]) -> bool,
) -> BodyState {
    let (sin, cos) = intent.yaw.sin_cos();
    let forward = Vec3::new(-sin, 0.0, -cos);
    let right = Vec3::new(cos, 0.0, -sin);
    let speed = if intent.sprint {
        SPRINT_SPEED
    } else {
        WALK_SPEED
    };
    let wish =
        (right * intent.direction.x + forward * intent.direction.y).clamp_length_max(1.0) * speed;
    let mut vertical = body.velocity.y;
    if body.on_ground && intent.jump {
        vertical = JUMP_SPEED;
    }
    vertical = (vertical - GRAVITY * dt).max(-TERMINAL_SPEED);
    let velocity = Vec3::new(wish.x, vertical, wish.z);

    let start = Aabb::standing(body.feet.to_array(), BODY_HALF_WIDTH, BODY_HEIGHT);
    let sweep = start.sweep((velocity * dt).to_array(), is_solid);
    let landed = sweep.blocked[1] && velocity.y < 0.0;
    let mut velocity = velocity;
    if sweep.blocked[1] {
        velocity.y = 0.0;
    }
    for axis in [0, 2] {
        if sweep.blocked[axis] {
            velocity[axis] = 0.0;
        }
    }
    BodyState {
        feet: Vec3::from_array(sweep.aabb.feet()),
        velocity,
        on_ground: landed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 64.0;

    fn ground(voxel: [i64; 3]) -> bool {
        voxel[1] < 0
    }

    fn standing() -> BodyState {
        BodyState {
            feet: Vec3::new(0.5, 0.0, 0.5),
            velocity: Vec3::ZERO,
            on_ground: true,
        }
    }

    fn run(mut body: BodyState, intent: MoveIntent, seconds: f32, dt: f32) -> BodyState {
        let mut elapsed = 0.0;
        while elapsed < seconds - dt / 2.0 {
            body = step(body, intent, dt, ground);
            elapsed += dt;
        }
        body
    }

    #[test]
    fn falls_and_lands_on_the_ground() {
        let airborne = BodyState {
            feet: Vec3::new(0.5, 5.0, 0.5),
            velocity: Vec3::ZERO,
            on_ground: false,
        };

        let body = run(airborne, MoveIntent::default(), 2.0, DT);

        assert!(body.on_ground);
        assert!(body.feet.y.abs() < 0.01);
    }

    #[test]
    fn walks_forward_where_it_faces() {
        let intent = MoveIntent {
            direction: Vec2::Y,
            ..MoveIntent::default()
        };

        let body = run(standing(), intent, 1.0, DT);

        assert!(
            (body.feet.z - (0.5 - WALK_SPEED)).abs() < 0.01,
            "{:?}",
            body.feet
        );
        assert!((body.feet.x - 0.5).abs() < 1e-4);
    }

    #[test]
    fn covers_the_same_distance_at_any_step_rate() {
        let intent = MoveIntent {
            direction: Vec2::new(1.0, 1.0),
            yaw: 0.7,
            sprint: true,
            jump: false,
        };

        let fine = run(standing(), intent, 1.0, 1.0 / 128.0);
        let coarse = run(standing(), intent, 1.0, 1.0 / 32.0);

        assert!((fine.feet - coarse.feet).length() < 0.01);
    }

    #[test]
    fn sprinting_is_faster_than_walking() {
        let walk = MoveIntent {
            direction: Vec2::Y,
            ..MoveIntent::default()
        };
        let sprint = MoveIntent {
            sprint: true,
            ..walk
        };

        let walked = run(standing(), walk, 1.0, DT).feet;
        let sprinted = run(standing(), sprint, 1.0, DT).feet;

        assert!(sprinted.z < walked.z);
    }

    #[test]
    fn jumps_only_from_the_ground_and_clears_one_block() {
        let jump = MoveIntent {
            jump: true,
            ..MoveIntent::default()
        };
        let mut body = step(standing(), jump, DT, ground);
        let mut peak: f32 = 0.0;
        for _ in 0..64 {
            body = step(body, MoveIntent::default(), DT, ground);
            peak = peak.max(body.feet.y);
        }

        let midair = BodyState {
            on_ground: false,
            ..standing()
        };
        assert!(peak > 1.0 && peak < 1.5, "peak {peak}");
        assert!(step(midair, jump, DT, ground).velocity.y <= 0.0);
    }
}
