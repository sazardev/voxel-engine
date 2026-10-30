//! One camera rig for first and third person (FR-16): same look, same body, a view switch.

use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use voxel_core::{VOXEL_SIZE_METRES, cast_ray};

use super::MainCamera;
use crate::input::PlayerAction;
use crate::player::{Body, EYE_HEIGHT, Look, Player, PlayerModel};
use crate::world::ChunkVoxels;

/// Third person: the camera sits this far behind the eye and this far to the right (DESIGN.md).
const ARM_LENGTH: f32 = 3.5;
const SHOULDER_OFFSET: f32 = 0.5;
/// Space kept between the camera and the voxel that shortened its arm.
const CAMERA_CLEARANCE: f32 = 0.2;

/// Which view the rig shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    FirstPerson,
    ThirdPerson,
}

impl View {
    #[must_use]
    pub const fn switched(self) -> Self {
        match self {
            Self::FirstPerson => Self::ThirdPerson,
            Self::ThirdPerson => Self::FirstPerson,
        }
    }
}

/// The camera's rig: the view it shows. Position and angles always come from the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Component)]
pub struct CameraRig {
    pub view: View,
}

/// How far from `from` along the unit vector `direction` the camera can go, up to `reach`,
/// keeping its clearance from the first solid voxel. Distances in metres; `is_solid` answers
/// for world voxels.
#[must_use]
pub fn arm_length(
    from: Vec3,
    direction: Vec3,
    reach: f32,
    is_solid: impl Fn([i64; 3]) -> bool,
) -> f32 {
    let grid_per_metre = 1.0 / VOXEL_SIZE_METRES;
    let origin = (from * grid_per_metre).to_array();
    let grid_reach = (reach + CAMERA_CLEARANCE) * grid_per_metre;
    cast_ray(origin, direction.to_array(), grid_reach, is_solid).map_or(reach, |hit| {
        (hit.distance / grid_per_metre - CAMERA_CLEARANCE).clamp(0.0, reach)
    })
}

pub(super) fn switch_view(
    players: Query<&ActionState<PlayerAction>, With<Player>>,
    mut rigs: Query<&mut CameraRig, With<MainCamera>>,
) {
    let switch = players
        .iter()
        .any(|actions| actions.just_pressed(&PlayerAction::SwitchView));
    if switch {
        for mut rig in &mut rigs {
            rig.view = rig.view.switched();
        }
    }
}

/// Puts the camera at the eye, or behind the shoulder with its arm shortened by walls, so it
/// never enters a solid block.
pub(super) fn place_camera(
    fixed: Res<Time<Fixed>>,
    voxels: Res<ChunkVoxels>,
    players: Query<(&Body, &Look), With<Player>>,
    mut cameras: Query<(&CameraRig, &mut Transform), With<MainCamera>>,
) {
    let Some((body, look)) = players.iter().next() else {
        return;
    };
    let eye = body.interpolated_feet(fixed.overstep_fraction()) + Vec3::Y * EYE_HEIGHT;
    let rotation = look.rotation();
    let blocks = |voxel| voxels.blocks(voxel);
    for (rig, mut transform) in &mut cameras {
        transform.rotation = rotation;
        transform.translation = match rig.view {
            View::FirstPerson => eye,
            View::ThirdPerson => {
                let right = rotation * Vec3::X;
                let shoulder = eye + right * arm_length(eye, right, SHOULDER_OFFSET, blocks);
                let back = rotation * Vec3::Z;
                shoulder + back * arm_length(shoulder, back, ARM_LENGTH, blocks)
            }
        };
    }
}

/// The player's own model is hidden in first person, where it would fill the view.
pub(super) fn show_model_in_third_person(
    rigs: Query<&CameraRig, (With<MainCamera>, Changed<CameraRig>)>,
    mut models: Query<&mut Visibility, With<PlayerModel>>,
) {
    for rig in &rigs {
        let visibility = match rig.view {
            View::FirstPerson => Visibility::Hidden,
            View::ThirdPerson => Visibility::Inherited,
        };
        for mut model in &mut models {
            *model = visibility;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_arm_reaches_full_length_in_open_air() {
        assert!((arm_length(Vec3::ZERO, Vec3::Z, ARM_LENGTH, |_| false) - ARM_LENGTH).abs() < 1e-6);
    }

    #[test]
    fn a_wall_shortens_the_arm_and_keeps_the_camera_off_it() {
        // Voxels from z = 2 m on are solid.
        let wall_behind = |voxel: [i64; 3]| voxel[2] >= 8;

        let length = arm_length(Vec3::new(0.1, 0.1, 0.1), Vec3::Z, ARM_LENGTH, wall_behind);

        assert!((length - (1.9 - CAMERA_CLEARANCE)).abs() < 1e-4, "{length}");
    }

    #[test]
    fn switching_twice_returns_to_the_first_view() {
        assert_eq!(View::FirstPerson.switched().switched(), View::FirstPerson);
    }
}
