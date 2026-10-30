//! The one main light of the world. It follows the `Environment`; nothing else moves it.

use bevy::light::{CascadeShadowConfigBuilder, GlobalAmbientLight};
use bevy::prelude::*;

use super::daylight::daylight;
use super::state::Environment;

/// Shadow cascades reach this far from the camera, in metres (one voxel is one metre).
const SHADOW_DISTANCE: f32 = 220.0;

/// Marks the directional light driven by the environment.
#[derive(Debug, Component)]
pub struct Sun;

pub(super) fn spawn_sun(mut commands: Commands) {
    commands.spawn((
        Sun,
        DirectionalLight {
            shadow_maps_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            maximum_distance: SHADOW_DISTANCE,
            ..default()
        }
        .build(),
    ));
}

pub(super) fn follow_environment(
    environment: Res<Environment>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut sun: Query<(&mut DirectionalLight, &mut Transform), With<Sun>>,
) {
    let light = daylight(&environment);
    for (mut directional, mut transform) in &mut sun {
        directional.color = light.light_color;
        directional.illuminance = light.light_illuminance_lux;
        *transform = Transform::default().looking_to(-light.toward_light, Vec3::Y);
    }
    ambient.color = light.ambient_color;
    ambient.brightness = light.ambient_brightness;
}
