//! Sky and distance fog. Both follow the `Environment`; fog thickens with the view distance so
//! the edge of the streamed world fades into the sky instead of ending in a line.

use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;

use super::daylight::daylight;
use super::state::Environment;
use crate::world::WorldSettings;

/// Fraction of the view distance where the fog leaves 5% of the terrain's contrast.
const FOG_VISIBILITY: f32 = 0.85;

pub(super) fn follow_environment(
    mut commands: Commands,
    environment: Res<Environment>,
    settings: Option<Res<WorldSettings>>,
    mut clear: ResMut<ClearColor>,
    mut cameras: Query<(Entity, Option<&mut DistanceFog>), With<Camera3d>>,
) {
    let light = daylight(&environment);
    clear.0 = light.sky_color;
    let view_distance = settings.map_or(
        WorldSettings::default().view_distance_metres(),
        |settings| settings.view_distance_metres(),
    );
    let fog = DistanceFog {
        color: light.sky_color,
        falloff: FogFalloff::from_visibility(view_distance * FOG_VISIBILITY),
        ..default()
    };
    for (camera, current) in &mut cameras {
        match current {
            Some(mut current) => *current = fog.clone(),
            None => {
                commands.entity(camera).insert(fog.clone());
            }
        }
    }
}
