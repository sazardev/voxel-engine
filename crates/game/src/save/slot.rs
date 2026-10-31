//! The world save slot: loading with a fallback to the last good copy, saving with a backup.

use voxel_core::SaveStore;

use super::format::{SaveError, WorldSave, decode, encode};

/// The save file and the previous good one, kept until the next successful save.
pub const SAVE_NAME: &str = "world.save";
pub const BACKUP_NAME: &str = "world.save.bak";

/// A save that was read, and whether it came from the backup because the main file was damaged.
#[derive(Debug, Clone, PartialEq)]
pub struct Loaded {
    pub save: WorldSave,
    pub from_backup: bool,
}

/// Reads the world save. `Ok(None)` when there is none. A damaged file falls back to the backup;
/// a file from a newer game is an error and nothing is changed.
///
/// # Errors
/// Returns [`SaveError`] when neither file can be used.
pub fn load(store: &dyn SaveStore) -> Result<Option<Loaded>, SaveError> {
    let Some(bytes) = store.read(SAVE_NAME)? else {
        return Ok(None);
    };
    match decode(&bytes) {
        Ok(save) => Ok(Some(Loaded {
            save,
            from_backup: false,
        })),
        Err(error @ SaveError::NewerVersion { .. }) => Err(error),
        Err(damaged) => match store.read(BACKUP_NAME)? {
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

/// Writes the world save, first keeping the current file as the backup when it is a good save.
/// The store's atomic write means a crash never leaves a broken save (NFR-5).
///
/// # Errors
/// Returns [`SaveError`] when the save cannot be encoded or stored.
pub fn save(store: &mut dyn SaveStore, world: &WorldSave) -> Result<(), SaveError> {
    let bytes = encode(world)?;
    let current_is_good = store
        .read(SAVE_NAME)?
        .is_some_and(|current| decode(&current).is_ok());
    if current_is_good {
        store.copy(SAVE_NAME, BACKUP_NAME)?;
    }
    store.write(SAVE_NAME, &bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use voxel_core::MemorySaveStore;

    use super::*;
    use crate::save::format::tests::sample;

    #[test]
    fn a_new_slot_has_nothing_to_load() {
        assert_eq!(load(&MemorySaveStore::default()), Ok(None));
    }

    #[test]
    fn loads_what_was_saved() {
        let mut store = MemorySaveStore::default();

        save(&mut store, &sample()).expect("saves");

        assert_eq!(
            load(&store),
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

        save(&mut store, &first).expect("saves");
        save(&mut store, &second).expect("saves");

        let backup = store
            .read(BACKUP_NAME)
            .expect("reads")
            .expect("backup exists");
        assert_eq!(decode(&backup), Ok(first));
    }

    #[test]
    fn a_damaged_save_falls_back_to_the_backup() {
        let mut store = MemorySaveStore::default();
        save(&mut store, &sample()).expect("saves");
        save(
            &mut store,
            &WorldSave {
                seed: 7,
                ..sample()
            },
        )
        .expect("saves");

        store
            .write(SAVE_NAME, b"VXWS\x01\x00 broken")
            .expect("memory never fails");

        assert_eq!(
            load(&store),
            Ok(Some(Loaded {
                save: sample(),
                from_backup: true
            }))
        );
    }

    #[test]
    fn a_damaged_save_never_replaces_a_good_backup() {
        let mut store = MemorySaveStore::default();
        save(&mut store, &sample()).expect("saves");
        save(&mut store, &sample()).expect("saves");
        store
            .write(SAVE_NAME, b"garbage")
            .expect("memory never fails");

        save(
            &mut store,
            &WorldSave {
                seed: 9,
                ..sample()
            },
        )
        .expect("saves");

        let backup = store
            .read(BACKUP_NAME)
            .expect("reads")
            .expect("backup exists");
        assert_eq!(decode(&backup), Ok(sample()));
    }

    #[test]
    fn a_save_from_a_newer_game_is_refused_and_left_alone() {
        let mut store = MemorySaveStore::default();
        let mut newer = super::super::format::encode(&sample()).expect("encodes");
        newer[4] = 0xFF;
        store.write(SAVE_NAME, &newer).expect("memory never fails");

        let result = load(&store);

        assert!(matches!(result, Err(SaveError::NewerVersion { .. })));
        assert_eq!(store.read(SAVE_NAME), Ok(Some(newer)));
    }
}
