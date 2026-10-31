//! Loading at startup, autosaving off the main thread, and a last save on quit.

use std::sync::{Arc, Mutex, PoisonError};

use bevy::prelude::*;
use bevy::tasks::futures::check_ready;
use bevy::tasks::{IoTaskPool, Task, block_on};
use voxel_core::SaveStore;

use super::format::{SaveError, WorldSave};
use super::slot::{load, save};
use super::snapshot::{apply, capture};
use crate::environment::Environment;
use crate::player::{Body, Look, Player};
use crate::world::WorldSettings;

/// Seconds between autosaves; the settings will choose it (SPEC Q15).
pub const AUTOSAVE_SECONDS: f32 = 60.0;

/// Where this session saves. The composition root inserts it; without it nothing is saved.
#[derive(Clone, Resource)]
pub struct SaveSlot(pub Arc<Mutex<dyn SaveStore + Send>>);

impl std::fmt::Debug for SaveSlot {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SaveSlot")
    }
}

impl SaveSlot {
    pub fn new(store: impl SaveStore + Send + 'static) -> Self {
        Self(Arc::new(Mutex::new(store)))
    }

    fn save(&self, world: &WorldSave) -> Result<(), SaveError> {
        // A panic while saving cannot leave the store half-written (writes are atomic), so a
        // poisoned lock is still safe to use.
        let mut store = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        save(&mut *store, world)
    }
}

/// The autosave clock and the one save that may be in flight at a time.
#[derive(Debug, Default, Resource)]
pub struct Autosave {
    seconds_since_save: f32,
    in_flight: Option<Task<Result<(), SaveError>>>,
}

pub(super) fn load_world(
    mut commands: Commands,
    slot: Option<Res<SaveSlot>>,
    mut settings: ResMut<WorldSettings>,
    mut environment: ResMut<Environment>,
) {
    let Some(slot) = slot else {
        info!("saves are off on this device; seed {:#x}", settings.seed.0);
        return;
    };
    let loaded = {
        let store = slot.0.lock().unwrap_or_else(PoisonError::into_inner);
        load(&*store)
    };
    let result = loaded.and_then(|loaded| match loaded {
        None => Ok(None),
        Some(loaded) => apply(&loaded.save, &mut settings, &mut environment)
            .map(|start| Some((start, loaded.from_backup))),
    });
    match result {
        Ok(None) => info!("new world, seed {:#x}", settings.seed.0),
        Ok(Some((start, from_backup))) => {
            if from_backup {
                warn!("the save was damaged; loaded the previous one");
            }
            info!("loaded world, seed {:#x}", settings.seed.0);
            commands.insert_resource(start);
        }
        Err(error) => {
            // Keep the file untouched: this session plays a new world and does not save over it.
            error!("{error}; starting a new world that will not be saved");
            commands.remove_resource::<SaveSlot>();
        }
    }
}

type PlayerState<'a> = (&'a Body, &'a Look);

#[expect(
    clippy::too_many_arguments,
    reason = "a Bevy system: each parameter is a resource or query it uses"
)]
pub(super) fn autosave(
    time: Res<Time<Real>>,
    slot: Option<Res<SaveSlot>>,
    mut autosave: ResMut<Autosave>,
    settings: Res<WorldSettings>,
    environment: Res<Environment>,
    players: Query<PlayerState, With<Player>>,
) {
    if let Some(done) = autosave.in_flight.as_mut().and_then(check_ready) {
        autosave.in_flight = None;
        if let Err(error) = done {
            error!("autosave failed: {error}");
        }
    }
    let Some(slot) = slot else {
        return;
    };
    autosave.seconds_since_save += time.delta_secs();
    if autosave.seconds_since_save < AUTOSAVE_SECONDS || autosave.in_flight.is_some() {
        return;
    }
    let Some(world) = capture(&settings, &environment, players.iter().next()) else {
        return;
    };
    autosave.seconds_since_save = 0.0;
    let slot = slot.clone();
    autosave.in_flight = Some(IoTaskPool::get().spawn(async move { slot.save(&world) }));
}

#[expect(
    clippy::too_many_arguments,
    reason = "a Bevy system: each parameter is a resource or query it uses"
)]
pub(super) fn save_on_exit(
    mut exits: MessageReader<AppExit>,
    slot: Option<Res<SaveSlot>>,
    mut autosave: ResMut<Autosave>,
    settings: Res<WorldSettings>,
    environment: Res<Environment>,
    players: Query<PlayerState, With<Player>>,
) {
    if exits.read().next().is_none() {
        return;
    }
    let Some(slot) = slot else {
        return;
    };
    if let Some(task) = autosave.in_flight.take() {
        // The last autosave finishes before the final one replaces it.
        let _ = block_on(task);
    }
    if let Some(world) = capture(&settings, &environment, players.iter().next()) {
        match slot.save(&world) {
            Ok(()) => info!("saved the world"),
            Err(error) => error!("saving on quit failed: {error}"),
        }
    }
}
