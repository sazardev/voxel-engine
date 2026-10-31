//! Saving and loading the world (FR-19, FR-24, FR-25, NFR-5; ADR 0017). The composition root
//! inserts a [`SaveSlot`]; without one the game plays without saving.

mod autosave;
mod format;
mod slot;
mod snapshot;

use bevy::prelude::*;
use voxel_gen::WorldSeed;

pub use autosave::{AUTOSAVE_SECONDS, Autosave, SaveSlot};
pub use format::{
    EditedChunk, EnvironmentSave, FORMAT_VERSION, PlayerSave, SaveError, SavedWeather, WorldSave,
    decode, encode,
};
pub use slot::{BACKUP_NAME, Loaded, SAVE_NAME, load, save};
pub use snapshot::{apply, capture};

#[derive(Debug, Default)]
pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Autosave>()
            .add_systems(PreStartup, autosave::load_world)
            .add_systems(Update, autosave::autosave)
            .add_systems(Last, autosave::save_on_exit);
    }
}

/// A seed for a new world when the player gives none (FR-29): the clock, mixed. Generation
/// itself never reads the clock; only the composition root picks a seed this way.
#[must_use]
pub fn fresh_seed() -> WorldSeed {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    let mut mixed =
        u64::try_from(nanos % u128::from(u64::MAX)).unwrap_or(0) ^ 0x9e37_79b9_7f4a_7c15;
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    WorldSeed(mixed ^ (mixed >> 31))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::asset::AssetPlugin;
    use bevy::time::TimeUpdateStrategy;
    use voxel_core::{MemorySaveStore, SaveStore};

    use super::*;
    use crate::environment::{Environment, EnvironmentPlugin};
    use crate::player::{Body, Look, Player, PlayerPlugin};
    use crate::world::{ChunkWorkers, WorldPlugin, WorldSettings};

    fn game_app(slot: &SaveSlot, seed: u64) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Mesh>()
            .init_asset::<StandardMaterial>()
            .init_asset::<Image>()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
                1.0 / 64.0,
            )))
            .insert_resource(ChunkWorkers::Inline)
            .insert_resource(WorldSettings {
                seed: WorldSeed(seed),
                view_radius: 1,
                lod_levels: 0,
            })
            .insert_resource(slot.clone())
            .add_plugins((EnvironmentPlugin, WorldPlugin, PlayerPlugin, SavePlugin));
        app
    }

    fn player(app: &mut App) -> Option<(Body, Look)> {
        let mut query = app
            .world_mut()
            .query_filtered::<(&Body, &Look), With<Player>>();
        query
            .iter(app.world())
            .next()
            .map(|(body, look)| (*body, *look))
    }

    fn run_until_player(app: &mut App) -> (Body, Look) {
        for _ in 0..200 {
            app.update();
            if let Some(found) = player(app) {
                return found;
            }
        }
        panic!("the player never appeared");
    }

    #[test]
    fn quitting_saves_the_world_and_the_next_start_resumes_it() {
        let slot = SaveSlot::new(MemorySaveStore::default());
        let mut first = game_app(&slot, 11);
        let (body, _) = run_until_player(&mut first);
        let resumed_feet = body.state.feet + Vec3::new(2.0, 0.0, -1.5);
        {
            let mut query = first
                .world_mut()
                .query_filtered::<(&mut Body, &mut Look), With<Player>>();
            let (mut body, mut look) = query.single_mut(first.world_mut()).expect("one player");
            *body = Body::at(resumed_feet);
            look.yaw = 1.25;
        }
        first.world_mut().resource_mut::<Environment>().time_of_day = 0.7;

        first.world_mut().write_message(AppExit::Success);
        first.update();
        // The next session starts with another seed proposed; the save must win.
        let mut second = game_app(&slot, 99);
        let (body, look) = run_until_player(&mut second);

        assert_eq!(
            second.world().resource::<WorldSettings>().seed,
            WorldSeed(11)
        );
        assert!((second.world().resource::<Environment>().time_of_day - 0.7).abs() < 1e-3);
        assert!(
            body.state.feet.distance(resumed_feet) < 1e-4,
            "{:?}",
            body.state.feet
        );
        assert!((look.yaw - 1.25).abs() < 1e-6);
    }

    #[test]
    fn a_save_from_a_newer_game_is_never_overwritten() {
        let mut store = MemorySaveStore::default();
        let mut newer = encode(&WorldSave {
            seed: 5,
            generator_version: voxel_gen::GENERATOR_VERSION,
            player: PlayerSave {
                feet: [0.0; 3],
                yaw: 0.0,
                pitch: 0.0,
            },
            environment: EnvironmentSave {
                time_of_day: 0.3,
                weather: SavedWeather::Clear,
            },
            edited_chunks: Vec::new(),
        })
        .expect("encodes");
        newer[4..6].copy_from_slice(&(FORMAT_VERSION + 1).to_le_bytes());
        store.write(SAVE_NAME, &newer).expect("memory never fails");
        let slot = SaveSlot::new(store);
        let mut app = game_app(&slot, 11);

        run_until_player(&mut app);
        app.world_mut().write_message(AppExit::Success);
        app.update();

        let stored = slot
            .0
            .lock()
            .expect("not poisoned")
            .read(SAVE_NAME)
            .expect("memory never fails");
        assert_eq!(stored, Some(newer));
    }
}
