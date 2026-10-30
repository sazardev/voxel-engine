//! Player actions. Keyboard, mouse and gamepad map to the same actions; touch joins them later
//! (FR-18). Remapping (FR-55) edits these maps, never the systems that read them.

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

/// What the player can ask for, whatever device they use.
#[derive(Actionlike, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum PlayerAction {
    /// Walk: x is right, y is forward, length at most 1.
    #[actionlike(DualAxis)]
    Move,
    /// Mouse look: the raw pointer delta of this frame, in pixels (y down).
    #[actionlike(DualAxis)]
    Look,
    /// Stick look: a turn rate from -1 to 1 on each axis (y up).
    #[actionlike(DualAxis)]
    Turn,
    Jump,
    Sprint,
    SwitchView,
}

impl PlayerAction {
    /// The default bindings for keyboard and mouse and for a gamepad.
    #[must_use]
    pub fn default_input_map() -> InputMap<Self> {
        InputMap::default()
            .with_dual_axis(Self::Move, VirtualDPad::wasd())
            .with_dual_axis(Self::Move, GamepadStick::LEFT)
            .with_dual_axis(Self::Look, MouseMove::default())
            .with_dual_axis(Self::Turn, GamepadStick::RIGHT)
            .with(Self::Jump, KeyCode::Space)
            .with(Self::Jump, GamepadButton::South)
            .with(Self::Sprint, KeyCode::ShiftLeft)
            .with(Self::Sprint, GamepadButton::LeftThumb)
            .with(Self::SwitchView, KeyCode::KeyV)
            .with(Self::SwitchView, GamepadButton::Select)
    }
}

/// Reads devices into `ActionState<PlayerAction>` each frame.
#[derive(Debug, Default)]
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default());
    }
}
