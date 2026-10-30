//! Turns the player's look from mouse and stick input: raw, no smoothing (FR-57).

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use leafwing_input_manager::prelude::ActionState;

use crate::input::PlayerAction;
use crate::player::{Look, Player};

/// Mouse sensitivity until the comfort settings expose it (FR-59).
const MOUSE_RADIANS_PER_PIXEL: f32 = 0.0022;
/// Full stick deflection turns this fast.
const STICK_RADIANS_PER_SECOND: f32 = 3.0;

pub(super) fn apply_look(
    time: Res<Time>,
    windows: Query<&CursorOptions, With<PrimaryWindow>>,
    mut players: Query<(&mut Look, &ActionState<PlayerAction>), With<Player>>,
) {
    // The mouse only looks while the cursor is captured; without a window (tests) it always does.
    let mouse_looks = windows
        .iter()
        .all(|cursor| cursor.grab_mode != CursorGrabMode::None);
    let stick_step = STICK_RADIANS_PER_SECOND * time.delta_secs();
    for (mut look, actions) in &mut players {
        let mouse = if mouse_looks {
            actions.axis_pair(&PlayerAction::Look)
        } else {
            Vec2::ZERO
        };
        let stick = actions.clamped_axis_pair(&PlayerAction::Turn);
        look.turn(
            -mouse.x * MOUSE_RADIANS_PER_PIXEL - stick.x * stick_step,
            -mouse.y * MOUSE_RADIANS_PER_PIXEL + stick.y * stick_step,
        );
    }
}
