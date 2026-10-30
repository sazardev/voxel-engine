//! Chunk storage, coordinates and edits. Pure Rust: no Bevy types (ADR 0002).
//!
//! Start here when adding voxel logic: a small type per file, errors as enums, ports as traits
//! owned by this crate (`ChunkStore`, `Clock`) with an in-memory fake for tests.

mod chunk;
mod clock;
mod coords;
mod edit_error;
mod memory_store;
mod store;
mod voxel;

pub use chunk::{CHUNK_EDGE, CHUNK_VOLUME, Chunk, WrongVoxelCount};
pub use clock::{Clock, FixedClock};
pub use coords::{ChunkPos, LocalPos};
pub use edit_error::EditError;
pub use memory_store::MemoryChunkStore;
pub use store::{ChunkStore, StoreError};
pub use voxel::VoxelId;
