//! The list of worlds in a save store and the creation of a new one (FR-24).

use thiserror::Error;
use voxel_core::{SaveFileInfo, SaveStore, SaveStoreError};

use super::world_id::{WorldId, WorldIdError};

/// How many worlds a store may hold; the list and the search for a free id stay bounded.
pub const MAX_WORLDS: u32 = 99;
/// The longest display name kept; a longer one is cut.
const MAX_NAME_CHARS: usize = 48;

/// What the world list shows for one world.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldSummary {
    pub id: WorldId,
    pub name: String,
    /// When the world was last saved, seconds since the Unix epoch.
    pub last_played_secs: Option<u64>,
    /// Everything the world keeps on disk: save, backup and name.
    pub size_bytes: u64,
}

/// Why the worlds could not be listed or a world could not be created.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorldsError {
    #[error(transparent)]
    InvalidName(#[from] WorldIdError),
    #[error("there are already {MAX_WORLDS} worlds; delete one to make room")]
    TooManyWorlds,
    #[error(transparent)]
    Storage(#[from] SaveStoreError),
}

/// Every world in the store, the most recently played first.
///
/// # Errors
/// Returns [`WorldsError::Storage`] when the store cannot be listed or read.
pub fn list_worlds(store: &dyn SaveStore) -> Result<Vec<WorldSummary>, WorldsError> {
    let files = store.list()?;
    let mut worlds = Vec::new();
    for file in &files {
        let Some(id) = file.name.strip_suffix(".save").and_then(WorldId::from_slug) else {
            continue;
        };
        let name = read_name(store, &id)?;
        worlds.push(WorldSummary {
            last_played_secs: file.modified_secs,
            size_bytes: size_of_world(&files, &id),
            name,
            id,
        });
    }
    worlds.sort_by(|a, b| {
        b.last_played_secs
            .cmp(&a.last_played_secs)
            .then_with(|| a.id.as_str().cmp(b.id.as_str()))
    });
    worlds.truncate(MAX_WORLDS as usize);
    Ok(worlds)
}

/// Reserves a slot for a new world called `name` and returns its id. A name already taken gets a
/// number after it. The world joins the list with its first save.
///
/// # Errors
/// Returns [`WorldsError`] for an empty name, a full store, or a store that cannot be written.
pub fn create_world(store: &mut dyn SaveStore, name: &str) -> Result<WorldId, WorldsError> {
    let wanted = WorldId::from_name(name)?;
    let taken: Vec<String> = store.list()?.into_iter().map(|file| file.name).collect();
    if count_worlds(&taken) >= MAX_WORLDS as usize {
        return Err(WorldsError::TooManyWorlds);
    }
    // A name file without a save is a world that was never played; its slot is free again.
    let is_free = |id: &WorldId| !taken.contains(&id.save_file());
    let id = std::iter::once(wanted.clone())
        .chain((2..=MAX_WORLDS).map(|number| wanted.numbered(number)))
        .find(is_free)
        .ok_or(WorldsError::TooManyWorlds)?;
    let shown: String = name.trim().chars().take(MAX_NAME_CHARS).collect();
    store.write(&id.name_file(), shown.as_bytes())?;
    Ok(id)
}

/// How many worlds the files hold: a world exists once it has a save.
fn count_worlds(names: &[String]) -> usize {
    names
        .iter()
        .filter(|name| name.strip_suffix(".save").is_some())
        .count()
}

fn read_name(store: &dyn SaveStore, id: &WorldId) -> Result<String, SaveStoreError> {
    let stored = store
        .read(&id.name_file())?
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty());
    Ok(stored.unwrap_or_else(|| id.to_string()))
}

fn size_of_world(files: &[SaveFileInfo], id: &WorldId) -> u64 {
    let own = [id.save_file(), id.backup_file(), id.name_file()];
    files
        .iter()
        .filter(|file| own.contains(&file.name))
        .map(|file| file.size_bytes)
        .sum()
}

#[cfg(test)]
mod tests {
    use voxel_core::MemorySaveStore;

    use super::*;
    use crate::save::format::tests::sample;
    use crate::save::slot::save;

    fn saved_world(store: &mut MemorySaveStore, name: &str, at: u64) -> WorldId {
        store.set_time(at);
        let id = create_world(store, name).expect("creates");
        save(store, &id, &sample()).expect("saves");
        id
    }

    #[test]
    fn an_empty_store_has_no_worlds() {
        assert_eq!(list_worlds(&MemorySaveStore::default()), Ok(Vec::new()));
    }

    #[test]
    fn each_world_shows_its_name_last_played_time_and_size() {
        let mut store = MemorySaveStore::default();
        let id = saved_world(&mut store, "Island Home", 500);

        let worlds = list_worlds(&store).expect("lists");

        let own_bytes = ["island-home.save", "island-home.name"]
            .iter()
            .map(|name| store.read(name).expect("reads").expect("exists").len() as u64)
            .sum::<u64>();
        assert_eq!(
            worlds,
            vec![WorldSummary {
                id,
                name: "Island Home".to_owned(),
                last_played_secs: Some(500),
                size_bytes: own_bytes,
            }]
        );
    }

    #[test]
    fn the_most_recently_played_world_comes_first() {
        let mut store = MemorySaveStore::default();
        saved_world(&mut store, "old", 100);
        saved_world(&mut store, "new", 900);
        saved_world(&mut store, "middle", 400);

        let names: Vec<_> = list_worlds(&store)
            .expect("lists")
            .into_iter()
            .map(|world| world.name)
            .collect();

        assert_eq!(names, ["new", "middle", "old"]);
    }

    #[test]
    fn the_backup_counts_toward_size_but_is_not_another_world() {
        let mut store = MemorySaveStore::default();
        let id = saved_world(&mut store, "twice", 100);
        let before = list_worlds(&store).expect("lists")[0].size_bytes;

        save(&mut store, &id, &sample()).expect("saves");
        let worlds = list_worlds(&store).expect("lists");

        assert_eq!(worlds.len(), 1);
        assert!(worlds[0].size_bytes > before);
    }

    #[test]
    fn the_world_from_before_slots_is_listed_under_its_id() {
        let mut store = MemorySaveStore::default();
        save(&mut store, &WorldId::legacy(), &sample()).expect("saves");

        let worlds = list_worlds(&store).expect("lists");

        assert_eq!(worlds.len(), 1);
        assert_eq!(worlds[0].id, WorldId::legacy());
        assert_eq!(worlds[0].name, "world");
    }

    #[test]
    fn a_missing_or_unreadable_name_falls_back_to_the_id() {
        let mut store = MemorySaveStore::default();
        let id = saved_world(&mut store, "named", 1);
        store
            .write(&id.name_file(), &[0xFF, 0xFE])
            .expect("memory never fails");

        assert_eq!(list_worlds(&store).expect("lists")[0].name, "named");
    }

    #[test]
    fn a_taken_name_gets_a_number_instead_of_overwriting() {
        let mut store = MemorySaveStore::default();
        let first = saved_world(&mut store, "Home", 1);

        let second = create_world(&mut store, "home").expect("creates");

        assert_eq!(first.as_str(), "home");
        assert_eq!(second.as_str(), "home-2");
    }

    #[test]
    fn a_name_without_letters_is_rejected_and_nothing_is_written() {
        let mut store = MemorySaveStore::default();

        assert_eq!(
            create_world(&mut store, "***"),
            Err(WorldsError::InvalidName(WorldIdError::Empty))
        );
        assert_eq!(store.names(), Vec::<&str>::new());
    }

    #[test]
    fn a_full_store_refuses_a_new_world() {
        let mut store = MemorySaveStore::default();
        for number in 0..MAX_WORLDS {
            let id = create_world(&mut store, &format!("w{number}")).expect("room left");
            save(&mut store, &id, &sample()).expect("saves");
        }

        assert_eq!(
            create_world(&mut store, "one more"),
            Err(WorldsError::TooManyWorlds)
        );
    }

    #[test]
    fn a_long_display_name_is_cut() {
        let mut store = MemorySaveStore::default();
        let id = saved_world(&mut store, &"x".repeat(100), 1);

        let name = &list_worlds(&store).expect("lists")[0].name;

        assert_eq!(name.chars().count(), MAX_NAME_CHARS);
        assert_eq!(id.as_str().len(), 32);
    }

    #[test]
    fn a_world_never_saved_gives_its_slot_back() {
        let mut store = MemorySaveStore::default();
        let first = create_world(&mut store, "Home").expect("creates");

        let again = create_world(&mut store, "Home").expect("creates");

        assert_eq!(first, again);
        assert_eq!(store.names(), vec!["home.name"]);
    }
}
