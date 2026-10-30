//! The player: one body moved in fixed steps against the voxel grid (FR-57, ADR 0014), seen
//! through the camera rig (FR-16). The world streams around it.

mod body;
mod kinematics;
mod movement;
mod spawn;

use bevy::prelude::*;

pub use body::{Body, Look, Player};
pub use kinematics::{BODY_HALF_WIDTH, BODY_HEIGHT, BodyState, MoveIntent, step};
pub use spawn::PlayerModel;

use crate::world::Terrain;

/// Eye height above the feet, in metres.
pub const EYE_HEIGHT: f32 = 1.62;

#[derive(Debug, Default)]
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            spawn::spawn_player
                .run_if(resource_exists::<Terrain>)
                .run_if(not(any_with_component::<Player>)),
        )
        .add_systems(FixedUpdate, movement::move_player)
        .add_systems(Update, (movement::follow_player, spawn::place_body));
    }
}
