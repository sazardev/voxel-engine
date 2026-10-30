//! Keeps the stream plan centred on the focus; recomputed only when the focus changes column.

use std::ops::RangeInclusive;

use bevy::prelude::*;
use voxel_core::{CHUNK_EDGE, VOXEL_SIZE_METRES};
use voxel_gen::{MAX_SURFACE_HEIGHT, MIN_SURFACE_HEIGHT};

use super::settings::WorldSettings;
use std::collections::HashSet;

use super::lod_layout::{LodLayout, LodPatch};
use super::streaming::StreamPlan;

/// Where the world streams around, in metres. The player moves it.
#[derive(Debug, Default, Clone, Copy, PartialEq, Resource)]
pub struct StreamFocus(pub Vec3);

/// The plan the loading systems follow for detailed chunks.
#[derive(Debug, Resource)]
pub struct ActivePlan(pub StreamPlan);

/// The coarse patches drawn beyond the detailed area.
#[derive(Debug, Resource)]
pub struct ActiveLod {
    pub layout: LodLayout,
    wanted: HashSet<LodPatch>,
}

impl ActiveLod {
    #[must_use]
    pub fn wants(&self, patch: LodPatch) -> bool {
        self.wanted.contains(&patch)
    }
}

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

/// The detailed chunk plan and the coarse patches around `focus`.
pub(super) fn plans(settings: &WorldSettings, focus: StreamFocus) -> (ActivePlan, ActiveLod) {
    let column = focus_column(focus.0);
    let layout = LodLayout::around(column, settings.view_radius, settings.lod_levels);
    let plan = StreamPlan::for_columns(column, layout.detail.clone(), surface_layers());
    let wanted = layout.patches.iter().copied().collect();
    (ActivePlan(plan), ActiveLod { layout, wanted })
}

pub(super) fn follow_focus(
    focus: Res<StreamFocus>,
    settings: Res<WorldSettings>,
    mut plan: ResMut<ActivePlan>,
    mut lod: ResMut<ActiveLod>,
) {
    let column = focus_column(focus.0);
    if column != plan.0.focus() || settings.is_changed() {
        (*plan, *lod) = plans(&settings, *focus);
    }
}
