//! Keeps the stream plan centred on the focus; recomputed only when the focus changes column.

use std::ops::RangeInclusive;

use bevy::prelude::*;
use voxel_core::{CHUNK_EDGE, VOXEL_SIZE_METRES};
use voxel_gen::{MAX_SURFACE_HEIGHT, MIN_SURFACE_HEIGHT};

use super::settings::WorldSettings;
use super::streaming::StreamPlan;

/// Where the world streams around, in metres. The player moves it.
#[derive(Debug, Default, Clone, Copy, PartialEq, Resource)]
pub struct StreamFocus(pub Vec3);

/// The plan the loading systems follow.
#[derive(Debug, Resource)]
pub struct ActivePlan(pub StreamPlan);

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    reason = "CHUNK_EDGE is 32 and surface heights are a few hundred voxels"
)]
fn surface_layers() -> RangeInclusive<i32> {
    let edge = CHUNK_EDGE as i64;
    (MIN_SURFACE_HEIGHT.div_euclid(edge) as i32)..=(MAX_SURFACE_HEIGHT.div_euclid(edge) as i32)
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "focus coordinates are far inside i32 chunk range; floor() gives whole chunks"
)]
fn focus_column(focus: Vec3) -> (i32, i32) {
    let edge = CHUNK_EDGE as f32 * VOXEL_SIZE_METRES;
    (
        (focus.x / edge).floor() as i32,
        (focus.z / edge).floor() as i32,
    )
}

pub(super) fn initial_plan(settings: &WorldSettings, focus: StreamFocus) -> ActivePlan {
    ActivePlan(StreamPlan::around(
        focus_column(focus.0),
        settings.view_radius,
        surface_layers(),
    ))
}

pub(super) fn follow_focus(
    focus: Res<StreamFocus>,
    settings: Res<WorldSettings>,
    mut plan: ResMut<ActivePlan>,
) {
    let column = focus_column(focus.0);
    let radius_changed = settings.is_changed();
    if column != plan.0.focus() || radius_changed {
        *plan = initial_plan(&settings, StreamFocus(focus.0));
    }
}
