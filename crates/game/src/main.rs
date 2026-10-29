//! Binary entry point: window, renderer and the game plugin. Developer tools are added here, and
//! only in builds compiled with `--features dev`.

use bevy::prelude::*;
use game::{GamePlugin, RunMode};

fn main() -> AppExit {
    let mode = RunMode::current();
    let mut app = App::new();
    app.insert_resource(mode)
        .add_plugins((DefaultPlugins.set(mode.log_plugin()), GamePlugin));
    #[cfg(feature = "dev")]
    app.add_plugins(game::dev::DevPlugin);
    app.run()
}
