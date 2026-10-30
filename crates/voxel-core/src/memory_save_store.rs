use std::collections::BTreeMap;

use crate::save_store::{SaveStore, SaveStoreError};

/// In-memory `SaveStore` fake. Writes replace whole files at once, like the real contract.
#[derive(Debug, Default, Clone)]
pub struct MemorySaveStore {
    files: BTreeMap<String, Vec<u8>>,
}

impl MemorySaveStore {
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
        Ok(())
    }

    fn copy(&mut self, from: &str, to: &str) -> Result<(), SaveStoreError> {
        if let Some(bytes) = self.files.get(from).cloned() {
            self.files.insert(to.to_owned(), bytes);
        }
        Ok(())
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
}
