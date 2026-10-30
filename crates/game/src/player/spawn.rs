use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use super::body::{Body, Look, Player};
use super::kinematics::{BODY_HALF_WIDTH, BODY_HEIGHT};
use crate::input::PlayerAction;
use crate::world::{StreamFocus, Terrain};

/// Placeholder body color until characters exist: wood-ramp `#C98B4B` from DESIGN.md.
const BODY_COLOR: Srgba = Srgba::rgb(0.788, 0.545, 0.294);

/// The visible model of the player, a child of the player entity.
#[derive(Debug, Component)]
pub struct PlayerModel;

/// Spawns the player standing on the terrain at the world origin.
pub(super) fn spawn_player(
    mut commands: Commands,
    terrain: Res<Terrain>,
    mut focus: ResMut<StreamFocus>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (x, z) = (0, 0);
    #[expect(
        clippy::cast_precision_loss,
        reason = "surface heights are a few hundred metres, exact in f32"
    )]
    let ground = terrain.0.surface_height(x, z) as f32 + 1.0;
    let feet = Vec3::new(0.5, ground, 0.5);
    focus.0 = feet;
    let model = (
        PlayerModel,
        Mesh3d(meshes.add(Cuboid::new(
            BODY_HALF_WIDTH * 2.0,
            BODY_HEIGHT,
            BODY_HALF_WIDTH * 2.0,
        ))),
        MeshMaterial3d(materials.add(StandardMaterial::from_color(BODY_COLOR))),
        Transform::from_xyz(0.0, BODY_HEIGHT / 2.0, 0.0),
    );
    commands
        .spawn((
            Player,
            Body::at(feet),
            Look::default(),
            PlayerAction::default_input_map(),
            ActionState::<PlayerAction>::default(),
            Transform::from_translation(feet),
            Visibility::default(),
        ))
        .with_child(model);
}

/// Places the player entity between its last two fixed steps, so it moves smoothly.
pub(super) fn place_body(
    fixed: Res<Time<Fixed>>,
    mut players: Query<(&Body, &Look, &mut Transform), With<Player>>,
) {
    for (body, look, mut transform) in &mut players {
        transform.translation = body.interpolated_feet(fixed.overstep_fraction());
        transform.rotation = Quat::from_rotation_y(look.yaw);
    }
}
