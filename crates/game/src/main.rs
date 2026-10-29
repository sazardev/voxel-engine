//! Binary entry point: window, renderer and the game plugin.

use bevy::prelude::*;
use game::GamePlugin;

fn main() -> AppExit {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, GamePlugin));
    #[cfg(feature = "profile")]
    app.add_plugins((
        bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
        bevy::diagnostic::LogDiagnosticsPlugin::default(),
    ));
    app.run()
}
