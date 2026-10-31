use std::collections::BTreeMap;

use crate::save_store::{SaveFileInfo, SaveStore, SaveStoreError};

/// In-memory `SaveStore` fake. Writes replace whole files at once, like the real contract.
#[derive(Debug, Default, Clone)]
pub struct MemorySaveStore {
    files: BTreeMap<String, Vec<u8>>,
    modified: BTreeMap<String, u64>,
    now_secs: u64,
}

impl MemorySaveStore {
    /// Sets the time stamped on files written from now on, so tests control "last played".
    pub fn set_time(&mut self, now_secs: u64) {
        self.now_secs = now_secs;
    }

    /// Names of the files held, in order; lets tests check what was written.
    #[must_use]
    pub fn names(&self) -> Vec<&str> {
        self.files.keys().map(String::as_str).collect()
    }
}

impl SaveStore for MemorySaveStore {
    fn read(&self, name: &str) -> Result<Option<Vec<u8>>, SaveStoreError> {
        Ok(self.files.get(name).cloned())
    }

    fn write(&mut self, name: &str, bytes: &[u8]) -> Result<(), SaveStoreError> {
        self.files.insert(name.to_owned(), bytes.to_vec());
        self.modified.insert(name.to_owned(), self.now_secs);
        Ok(())
    }

    fn copy(&mut self, from: &str, to: &str) -> Result<(), SaveStoreError> {
        if let Some(bytes) = self.files.get(from).cloned() {
            self.write(to, &bytes)?;
        }
        Ok(())
    }

    fn list(&self) -> Result<Vec<SaveFileInfo>, SaveStoreError> {
        Ok(self
            .files
            .iter()
            .map(|(name, bytes)| SaveFileInfo {
                name: name.clone(),
                size_bytes: bytes.len() as u64,
                modified_secs: self.modified.get(name).copied(),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_nothing_for_a_file_never_written() {
        assert_eq!(MemorySaveStore::default().read("world"), Ok(None));
    }

    #[test]
    fn reads_what_was_written_last() {
        let mut store = MemorySaveStore::default();

        store.write("world", b"one").expect("memory never fails");
        store.write("world", b"two").expect("memory never fails");

        assert_eq!(store.read("world"), Ok(Some(b"two".to_vec())));
    }

    #[test]
    fn copies_a_file_and_ignores_a_missing_source() {
        let mut store = MemorySaveStore::default();
        store.write("world", b"data").expect("memory never fails");

        store
            .copy("world", "world.bak")
            .expect("memory never fails");
        store.copy("missing", "other").expect("memory never fails");

        assert_eq!(store.read("world.bak"), Ok(Some(b"data".to_vec())));
        assert_eq!(store.names(), vec!["world", "world.bak"]);
    }

    #[test]
    fn lists_names_sizes_and_write_times_in_name_order() {
        let mut store = MemorySaveStore::default();
        store.set_time(100);
        store.write("b.save", b"12345").expect("memory never fails");
        store.set_time(200);
        store.write("a.save", b"12").expect("memory never fails");

        assert_eq!(
            store.list(),
            Ok(vec![
                SaveFileInfo {
                    name: "a.save".to_owned(),
                    size_bytes: 2,
                    modified_secs: Some(200)
                },
                SaveFileInfo {
                    name: "b.save".to_owned(),
                    size_bytes: 5,
                    modified_secs: Some(100)
                },
            ])
        );
    }

    #[test]
    fn lists_nothing_for_an_empty_store() {
        assert_eq!(MemorySaveStore::default().list(), Ok(Vec::new()));
    }
}
