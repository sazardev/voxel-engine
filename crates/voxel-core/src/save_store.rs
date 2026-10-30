use thiserror::Error;

/// Why a save file could not be read or written.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SaveStoreError {
    #[error("save storage is unavailable: {0}")]
    Unavailable(String),
}

/// Port: where save files live, by name. Implemented in `platform` (files on disk); faked in
/// memory for tests.
///
/// Contract: `read` returns `Ok(None)` for a name never written. `write` replaces the whole file
/// atomically: if the process dies during a write, a later `read` returns either the previous
/// bytes or the new ones, never a mix. `copy` duplicates a file (for backups) and does nothing
/// when the source does not exist. No method panics.
pub trait SaveStore {
    /// # Errors
    /// Returns [`SaveStoreError`] when the storage cannot be read.
    fn read(&self, name: &str) -> Result<Option<Vec<u8>>, SaveStoreError>;

    /// # Errors
    /// Returns [`SaveStoreError`] when the storage cannot be written; the previous file is kept.
    fn write(&mut self, name: &str, bytes: &[u8]) -> Result<(), SaveStoreError>;

    /// # Errors
    /// Returns [`SaveStoreError`] when the copy cannot be made.
    fn copy(&mut self, from: &str, to: &str) -> Result<(), SaveStoreError>;
}
