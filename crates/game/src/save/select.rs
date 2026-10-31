//! Which world a session plays: the one asked for, or else the one played last.

use voxel_core::SaveStore;

use super::world_id::WorldId;
use super::worlds::{WorldsError, create_world, list_worlds};

/// The name of the world made when the store has none.
pub const FIRST_WORLD_NAME: &str = "World";

/// What the player asked for at startup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldChoice {
    /// Continue where the player left off.
    MostRecent,
    /// Open the world with this name, or start one when it does not exist.
    Named(String),
}

/// The world to play. Opening never changes a file; a new world only reserves its name.
///
/// # Errors
/// Returns [`WorldsError`] for a name that is empty, a store that is full, or a store that fails.
pub fn choose_world(
    store: &mut dyn SaveStore,
    choice: &WorldChoice,
) -> Result<WorldId, WorldsError> {
    match choice {
        WorldChoice::MostRecent => match list_worlds(store)?.into_iter().next() {
            Some(latest) => Ok(latest.id),
            None => create_world(store, FIRST_WORLD_NAME),
        },
        WorldChoice::Named(name) => {
            let wanted = WorldId::from_name(name)?;
            let exists = list_worlds(store)?.iter().any(|world| world.id == wanted);
            if exists {
                Ok(wanted)
            } else {
                create_world(store, name)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use voxel_core::MemorySaveStore;

    use super::*;
    use crate::save::format::tests::sample;
    use crate::save::slot::save;
    use crate::save::world_id::WorldIdError;

    fn played(store: &mut MemorySaveStore, name: &str, at: u64) -> WorldId {
        store.set_time(at);
        let id = create_world(store, name).expect("creates");
        save(store, &id, &sample()).expect("saves");
        id
    }

    #[test]
    fn continues_with_the_world_played_last() {
        let mut store = MemorySaveStore::default();
        played(&mut store, "old", 10);
        let latest = played(&mut store, "latest", 99);

        assert_eq!(
            choose_world(&mut store, &WorldChoice::MostRecent),
            Ok(latest)
        );
    }

    #[test]
    fn the_first_start_makes_the_first_world() {
        let mut store = MemorySaveStore::default();

        let id = choose_world(&mut store, &WorldChoice::MostRecent).expect("chooses");

        assert_eq!(id.as_str(), "world");
        assert_eq!(
            store.read(&id.name_file()),
            Ok(Some(FIRST_WORLD_NAME.as_bytes().to_vec()))
        );
    }

    #[test]
    fn starting_twice_without_saving_does_not_pile_up_worlds() {
        let mut store = MemorySaveStore::default();

        let first = choose_world(&mut store, &WorldChoice::MostRecent).expect("chooses");
        let second = choose_world(&mut store, &WorldChoice::MostRecent).expect("chooses");

        assert_eq!(first, second);
        assert_eq!(store.names().len(), 1);
    }

    #[test]
    fn a_named_world_that_exists_is_opened_not_duplicated() {
        let mut store = MemorySaveStore::default();
        let island = played(&mut store, "Island", 10);

        let chosen = choose_world(&mut store, &WorldChoice::Named("island".to_owned()));

        assert_eq!(chosen, Ok(island));
        assert_eq!(list_worlds(&store).expect("lists").len(), 1);
    }

    #[test]
    fn a_named_world_that_does_not_exist_is_started() {
        let mut store = MemorySaveStore::default();
        played(&mut store, "Island", 10);

        let chosen = choose_world(&mut store, &WorldChoice::Named("Desert".to_owned()));

        assert_eq!(chosen.map(|id| id.to_string()), Ok("desert".to_owned()));
    }

    #[test]
    fn an_unusable_name_is_an_error_and_writes_nothing() {
        let mut store = MemorySaveStore::default();

        let chosen = choose_world(&mut store, &WorldChoice::Named("!!".to_owned()));

        assert_eq!(chosen, Err(WorldsError::InvalidName(WorldIdError::Empty)));
        assert_eq!(store.names(), Vec::<&str>::new());
    }
}
