use bevy::prelude::Resource;

/// Runtime switches for the developer tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource)]
pub struct DevSettings {
    pub overlay_visible: bool,
}

impl Default for DevSettings {
    fn default() -> Self {
        Self {
            overlay_visible: true,
        }
    }
}
