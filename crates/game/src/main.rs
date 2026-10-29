//! Binary entry point: window, renderer and the game plugin.

use bevy::prelude::*;
use game::GamePlugin;

fn main() -> AppExit {
    App::new().add_plugins((DefaultPlugins, GamePlugin)).run()
}
