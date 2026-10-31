use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use voxel_core::VOXEL_SIZE_METRES;

use super::body::{Body, Look, Player};
use super::kinematics::{BODY_HALF_WIDTH, BODY_HEIGHT};
use crate::input::PlayerAction;
use crate::world::{StreamFocus, Terrain};

/// Placeholder body color until characters exist: wood-ramp `#C98B4B` from DESIGN.md.
const BODY_COLOR: Srgba = Srgba::rgb(0.788, 0.545, 0.294);

/// Where the player appears, when a save says so; otherwise on the terrain at the origin.
#[derive(Debug, Clone, Copy, PartialEq, Resource)]
pub struct PlayerStart {
    pub feet: Vec3,
    pub look: Look,
}

/// The visible model of the player, a child of the player entity.
#[derive(Debug, Component)]
pub struct PlayerModel;

/// Spawns the player where a save left them, or standing on the terrain at the world origin.
#[expect(
    clippy::too_many_arguments,
    reason = "a Bevy system: each parameter is a resource or query it uses"
)]
pub(super) fn spawn_player(
    mut commands: Commands,
    terrain: Res<Terrain>,
    start: Option<Res<PlayerStart>>,
    mut focus: ResMut<StreamFocus>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (x, z) = (0, 0);
    #[expect(
        clippy::cast_precision_loss,
        reason = "surface heights are a few hundred voxels, exact in f32"
    )]
    let ground = (terrain.0.surface_height(x, z) + 1) as f32 * VOXEL_SIZE_METRES;
    // Stand in the middle of voxel (x, z).
    let origin = Vec3::new(0.5 * VOXEL_SIZE_METRES, ground, 0.5 * VOXEL_SIZE_METRES);
    let (feet, look) = start.map_or((origin, Look::default()), |start| (start.feet, start.look));
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
            look,
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
