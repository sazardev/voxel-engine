//! Moves the player in fixed steps and keeps the world streaming around them.

use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use voxel_core::{ChunkPos, VOXEL_SIZE_METRES};

use super::body::{Body, Look, Player};
use super::kinematics::{MoveIntent, step};
use crate::input::PlayerAction;
use crate::world::{ChunkVoxels, StreamFocus};

/// The voxels under and at the feet are loaded, so a step cannot fall into unloaded space.
fn has_ground_loaded(voxels: &ChunkVoxels, feet: Vec3) -> bool {
    let voxel = (feet / VOXEL_SIZE_METRES).floor().as_i64vec3().to_array();
    let below = [voxel[0], voxel[1] - 1, voxel[2]];
    [voxel, below]
        .into_iter()
        .all(|voxel| ChunkPos::split(voxel).is_some_and(|(chunk, _)| voxels.contains(chunk)))
}

pub(super) fn move_player(
    time: Res<Time>,
    voxels: Res<ChunkVoxels>,
    mut players: Query<(&mut Body, &Look, &ActionState<PlayerAction>), With<Player>>,
) {
    for (mut body, look, actions) in &mut players {
        body.previous_feet = body.state.feet;
        if !has_ground_loaded(&voxels, body.state.feet) {
            continue;
        }
        let intent = MoveIntent {
            direction: actions.clamped_axis_pair(&PlayerAction::Move),
            yaw: look.yaw,
            sprint: actions.pressed(&PlayerAction::Sprint),
            jump: actions.pressed(&PlayerAction::Jump),
        };
        body.state = step(body.state, intent, time.delta_secs(), |voxel| {
            voxels.blocks(voxel)
        });
    }
}

pub(super) fn follow_player(players: Query<&Body, With<Player>>, mut focus: ResMut<StreamFocus>) {
    if let Some(body) = players.iter().next() {
        focus.set_if_neq(StreamFocus(body.state.feet));
    }
}
