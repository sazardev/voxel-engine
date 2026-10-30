//! How the player's body moves in one fixed step: a pure rule over the voxel grid (FR-22, FR-57).

use bevy::math::{Vec2, Vec3};
use voxel_core::{Aabb, Sweep, VOXEL_SIZE_METRES};

/// Collision box size, in metres.
pub const BODY_HALF_WIDTH: f32 = 0.3;
pub const BODY_HEIGHT: f32 = 1.8;
/// Ledges up to this height are climbed by walking into them: two 25 cm voxels, the steepest
/// step generated terrain has (`voxel_gen::MAX_SURFACE_STEP`).
pub const STEP_HEIGHT: f32 = 0.5;
const WALK_SPEED: f32 = 4.3;
const SPRINT_SPEED: f32 = 6.5;
const GRAVITY: f32 = 28.0;
/// Reaches about 1.25 m.
const JUMP_SPEED: f32 = 8.4;
const TERMINAL_SPEED: f32 = 60.0;
/// Grid units (world voxels) per metre.
const GRID_PER_METRE: f32 = 1.0 / VOXEL_SIZE_METRES;

/// The physical state of the player, advanced only in fixed steps. Units are metres.
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
/// `is_solid` answers for world voxels; positions and speeds are in metres.
#[must_use]
pub fn step(
    body: BodyState,
    intent: MoveIntent,
    dt: f32,
    is_solid: impl Fn([i64; 3]) -> bool,
) -> BodyState {
    let velocity = intended_velocity(body, intent, dt);
    let box_at_feet = Aabb::standing(
        (body.feet * GRID_PER_METRE).to_array(),
        BODY_HALF_WIDTH * GRID_PER_METRE,
        BODY_HEIGHT * GRID_PER_METRE,
    );
    let motion = velocity * dt * GRID_PER_METRE;
    let mut sweep = box_at_feet.sweep(motion.to_array(), &is_solid);
    let walked_into_something = sweep.blocked[0] || sweep.blocked[2];
    if body.on_ground
        && walked_into_something
        && let Some(stepped) = step_up(box_at_feet, motion, &sweep, &is_solid)
    {
        sweep = stepped;
    }
    settle(velocity, &sweep)
}

fn intended_velocity(body: BodyState, intent: MoveIntent, dt: f32) -> Vec3 {
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
    Vec3::new(wish.x, vertical, wish.z)
}

/// Tries the move again lifted by up to `STEP_HEIGHT`, then lowered back onto the ground.
/// Returns it when it gets further horizontally than the plain move and ends standing.
fn step_up(
    start: Aabb,
    motion: Vec3,
    plain: &Sweep,
    is_solid: impl Fn([i64; 3]) -> bool,
) -> Option<Sweep> {
    let lift = start.sweep([0.0, STEP_HEIGHT * GRID_PER_METRE, 0.0], &is_solid);
    let lifted_by = lift.aabb.min[1] - start.min[1];
    let across = lift.aabb.sweep([motion.x, 0.0, motion.z], &is_solid);
    let down = across
        .aabb
        .sweep([0.0, -lifted_by + motion.y.min(0.0), 0.0], &is_solid);
    let progress =
        |aabb: &Aabb| Vec2::new(aabb.min[0] - start.min[0], aabb.min[2] - start.min[2]).length();
    let further = progress(&down.aabb) > progress(&plain.aabb) + 1e-4;
    (further && down.blocked[1]).then_some(Sweep {
        aabb: down.aabb,
        blocked: [across.blocked[0], true, across.blocked[2]],
    })
}

/// The body after a sweep: speed is dropped on every axis something stopped.
fn settle(velocity: Vec3, sweep: &Sweep) -> BodyState {
    let landed = sweep.blocked[1] && velocity.y < 0.0;
    let mut velocity = velocity;
    for axis in 0..3 {
        if sweep.blocked[axis] {
            velocity[axis] = 0.0;
        }
    }
    BodyState {
        feet: Vec3::from_array(sweep.aabb.feet()) / GRID_PER_METRE,
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

    /// Flat ground at height 0 with a ledge `voxels` high from z = -1 m onward (ahead when
    /// facing -z).
    fn ledge(voxels: i64) -> impl Fn([i64; 3]) -> bool {
        move |[_, y, z]| y < 0 || (z < -4 && y < voxels)
    }

    fn walk_into_ledge(voxels: i64) -> BodyState {
        let walk = MoveIntent {
            direction: Vec2::Y,
            ..MoveIntent::default()
        };
        let mut body = standing();
        for _ in 0..128 {
            body = step(body, walk, DT, ledge(voxels));
        }
        body
    }

    #[test]
    fn walks_up_ledges_of_one_and_two_voxels_without_jumping() {
        for voxels in [1, 2] {
            let body = walk_into_ledge(voxels);

            #[expect(clippy::cast_precision_loss, reason = "a few voxels")]
            let top = voxels as f32 * VOXEL_SIZE_METRES;
            assert!(
                (body.feet.y - top).abs() < 0.01,
                "{voxels} voxels: {:?}",
                body.feet
            );
            assert!(
                body.feet.z < -2.0,
                "{voxels} voxels: stuck at {:?}",
                body.feet
            );
        }
    }

    #[test]
    fn a_ledge_of_three_voxels_needs_a_jump() {
        let body = walk_into_ledge(3);

        assert!(body.feet.y.abs() < 0.01);
        assert!(
            body.feet.z > -1.0 + BODY_HALF_WIDTH - 0.01,
            "{:?}",
            body.feet
        );
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
