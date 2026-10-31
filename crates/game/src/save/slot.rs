//! One world's save slot: loading with a fallback to the last good copy, saving with a backup.

use voxel_core::SaveStore;

use super::format::{SaveError, WorldSave, decode, encode};
use super::world_id::WorldId;

/// A save that was read, and whether it came from the backup because the main file was damaged.
#[derive(Debug, Clone, PartialEq)]
pub struct Loaded {
    pub save: WorldSave,
    pub from_backup: bool,
}

/// Reads a world's save. `Ok(None)` when there is none. A damaged file falls back to the backup;
/// a file from a newer game is an error and nothing is changed.
///
/// # Errors
/// Returns [`SaveError`] when neither file can be used.
pub fn load(store: &dyn SaveStore, world: &WorldId) -> Result<Option<Loaded>, SaveError> {
    let Some(bytes) = store.read(&world.save_file())? else {
        return Ok(None);
    };
    match decode(&bytes) {
        Ok(save) => Ok(Some(Loaded {
            save,
            from_backup: false,
        })),
        Err(error @ SaveError::NewerVersion { .. }) => Err(error),
        Err(damaged) => match store.read(&world.backup_file())? {
            Some(backup) => decode(&backup).map(|save| {
                Some(Loaded {
                    save,
                    from_backup: true,
                })
            }),
            None => Err(damaged),
        },
    }
}

/// Writes a world's save, first keeping the current file as the backup when it is a good save.
/// The store's atomic write means a crash never leaves a broken save (NFR-5).
///
/// # Errors
/// Returns [`SaveError`] when the save cannot be encoded or stored.
pub fn save(store: &mut dyn SaveStore, id: &WorldId, world: &WorldSave) -> Result<(), SaveError> {
    let bytes = encode(world)?;
    let current_is_good = store
        .read(&id.save_file())?
        .is_some_and(|current| decode(&current).is_ok());
    if current_is_good {
        store.copy(&id.save_file(), &id.backup_file())?;
    }
    store.write(&id.save_file(), &bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use voxel_core::MemorySaveStore;

    use super::*;
    use crate::save::format::tests::sample;

    fn world() -> WorldId {
        WorldId::legacy()
    }

    #[test]
    fn a_new_slot_has_nothing_to_load() {
        assert_eq!(load(&MemorySaveStore::default(), &world()), Ok(None));
    }

    #[test]
    fn loads_what_was_saved() {
        let mut store = MemorySaveStore::default();

        save(&mut store, &world(), &sample()).expect("saves");

        assert_eq!(
            load(&store, &world()),
            Ok(Some(Loaded {
                save: sample(),
                from_backup: false
            }))
        );
    }

    #[test]
    fn keeps_the_previous_save_as_a_backup() {
        let mut store = MemorySaveStore::default();
        let first = sample();
        let second = WorldSave {
            seed: 7,
            ..sample()
        };

        save(&mut store, &world(), &first).expect("saves");
        save(&mut store, &world(), &second).expect("saves");

        let backup = store
            .read(&world().backup_file())
            .expect("reads")
            .expect("backup exists");
        assert_eq!(decode(&backup), Ok(first));
    }

    #[test]
    fn a_damaged_save_falls_back_to_the_backup() {
        let mut store = MemorySaveStore::default();
        save(&mut store, &world(), &sample()).expect("saves");
        save(
            &mut store,
            &world(),
            &WorldSave {
                seed: 7,
                ..sample()
            },
        )
        .expect("saves");

        store
            .write(&world().save_file(), b"VXWS\x01\x00 broken")
            .expect("memory never fails");

        assert_eq!(
            load(&store, &world()),
            Ok(Some(Loaded {
                save: sample(),
                from_backup: true
            }))
        );
    }

    #[test]
    fn a_damaged_save_never_replaces_a_good_backup() {
        let mut store = MemorySaveStore::default();
        save(&mut store, &world(), &sample()).expect("saves");
        save(&mut store, &world(), &sample()).expect("saves");
        store
            .write(&world().save_file(), b"garbage")
            .expect("memory never fails");

        save(
            &mut store,
            &world(),
            &WorldSave {
                seed: 9,
                ..sample()
            },
        )
        .expect("saves");

        let backup = store
            .read(&world().backup_file())
            .expect("reads")
            .expect("backup exists");
        assert_eq!(decode(&backup), Ok(sample()));
    }

    #[test]
    fn two_worlds_never_touch_each_other() {
        let mut store = MemorySaveStore::default();
        let other = WorldId::from_name("island").expect("a valid name");
        let island = WorldSave {
            seed: 3,
            ..sample()
        };

        save(&mut store, &world(), &sample()).expect("saves");
        save(&mut store, &other, &island).expect("saves");

        assert_eq!(
            load(&store, &world()).map(|l| l.map(|l| l.save)),
            Ok(Some(sample()))
        );
        assert_eq!(
            load(&store, &other).map(|l| l.map(|l| l.save)),
            Ok(Some(island))
        );
    }

    #[test]
    fn a_save_from_a_newer_game_is_refused_and_left_alone() {
        let mut store = MemorySaveStore::default();
        let mut newer = super::super::format::encode(&sample()).expect("encodes");
        newer[4] = 0xFF;
        store
            .write(&world().save_file(), &newer)
            .expect("memory never fails");

        let result = load(&store, &world());

        assert!(matches!(result, Err(SaveError::NewerVersion { .. })));
        assert_eq!(store.read(&world().save_file()), Ok(Some(newer)));
    }
}
