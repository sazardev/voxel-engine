//! Binary entry point: window, renderer and the game plugin. Developer tools are added here, and
//! only in builds compiled with `--features dev`. This is where concrete adapters are chosen.

use bevy::prelude::*;
use game::save::{SaveSlot, fresh_seed};
use game::world::WorldSettings;
use game::{GamePlugin, RunMode};
use platform::{FileSaveStore, save_directory};

fn main() -> AppExit {
    let mode = RunMode::current();
    let mut app = App::new();
    app.insert_resource(mode)
        .add_plugins((DefaultPlugins.set(mode.log_plugin()), GamePlugin));
    #[cfg(feature = "dev")]
    app.add_plugins(game::dev::DevPlugin);
    // A new world gets a fresh seed; a save, when there is one, replaces it at startup.
    app.insert_resource(WorldSettings {
        seed: fresh_seed(),
        ..WorldSettings::default()
    });
    if let Some(slot) = open_save_slot() {
        app.insert_resource(slot);
    }
    app.run()
}

fn open_save_slot() -> Option<SaveSlot> {
    let directory = save_directory()?;
    match FileSaveStore::open(&directory) {
        Ok(store) => Some(SaveSlot::new(store)),
        Err(error) => {
            warn!("saves are off: {error}");
            None
        }
    }
}
