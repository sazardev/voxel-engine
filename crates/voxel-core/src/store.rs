use thiserror::Error;

use crate::chunk::Chunk;
use crate::coords::ChunkPos;

/// Why a chunk could not be loaded or saved.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum StoreError {
    #[error("chunk ({}, {}, {}) is corrupt: {reason}", pos.x, pos.y, pos.z)]
    Corrupt { pos: ChunkPos, reason: String },
    #[error("chunk storage is unavailable: {0}")]
    Unavailable(String),
}

/// Port: where chunks persist. Implemented by adapters in `platform`; faked in tests.
///
/// Contract: `load` returns `Ok(None)` for a chunk never saved, and `Ok(Some(saved))` after
/// `save`. A corrupt chunk is an error, never a panic or a silently empty chunk.
pub trait ChunkStore {
    /// # Errors
    /// Returns [`StoreError`] when the storage is unavailable or the stored data is corrupt.
    fn load(&self, pos: ChunkPos) -> Result<Option<Chunk>, StoreError>;

    /// # Errors
    /// Returns [`StoreError`] when the storage is unavailable.
    fn save(&mut self, pos: ChunkPos, chunk: &Chunk) -> Result<(), StoreError>;
}
