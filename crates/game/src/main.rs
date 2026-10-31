//! Binary entry point: window, renderer and the game plugin. Developer tools are added here, and
//! only in builds compiled with `--features dev`. This is where concrete adapters are chosen.

use bevy::prelude::*;
use game::save::{
    SaveSlot, StartupRequest, choose_world, format_world_list, fresh_seed, list_worlds,
    parse_startup_args,
};
use game::world::WorldSettings;
use game::{GamePlugin, RunMode};
use platform::{FileSaveStore, save_directory};

fn main() -> AppExit {
    let request = match parse_startup_args(std::env::args().skip(1)) {
        Ok(request) => request,
        Err(error) => return fail(&error),
    };
    if request.list_only {
        return print_worlds();
    }
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
    if let Some(slot) = open_save_slot(&request) {
        app.insert_resource(slot);
    }
    app.run()
}

fn open_save_slot(request: &StartupRequest) -> Option<SaveSlot> {
    let directory = save_directory()?;
    let mut store = match FileSaveStore::open(&directory) {
        Ok(store) => store,
        Err(error) => {
            warn!("saves are off: {error}");
            return None;
        }
    };
    match choose_world(&mut store, &request.choice) {
        Ok(world) => {
            info!("playing world {world}");
            Some(SaveSlot::new(store, world))
        }
        Err(error) => {
            warn!("saves are off: {error}");
            None
        }
    }
}

#[expect(
    clippy::print_stdout,
    reason = "--list-worlds exists to print the list for the player"
)]
fn print_worlds() -> AppExit {
    let Some(directory) = save_directory() else {
        return fail(&"this device has no save folder");
    };
    let listed = FileSaveStore::open(&directory)
        .map_err(|error| error.to_string())
        .and_then(|store| list_worlds(&store).map_err(|error| error.to_string()));
    match listed {
        Ok(worlds) => {
            print!("{}", format_world_list(&worlds));
            AppExit::Success
        }
        Err(error) => fail(&error),
    }
}

#[expect(
    clippy::print_stderr,
    reason = "a command line mistake is reported on stderr before any window opens"
)]
fn fail(error: &dyn std::fmt::Display) -> AppExit {
    eprintln!("{error}");
    AppExit::error()
}
