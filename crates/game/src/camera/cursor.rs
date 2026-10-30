//! Captures the cursor for mouse look; Escape releases it and a click captures it again.

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

pub(super) fn capture_on_start(mut windows: Query<&mut CursorOptions, With<PrimaryWindow>>) {
    for mut cursor in &mut windows {
        capture(&mut cursor);
    }
}

pub(super) fn toggle_capture(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut windows: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    for mut cursor in &mut windows {
        if keys.just_pressed(KeyCode::Escape) {
            cursor.grab_mode = CursorGrabMode::None;
            cursor.visible = true;
        } else if buttons.just_pressed(MouseButton::Left) {
            capture(&mut cursor);
        }
    }
}

fn capture(cursor: &mut CursorOptions) {
    cursor.grab_mode = CursorGrabMode::Locked;
    cursor.visible = false;
}
