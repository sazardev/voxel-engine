use std::collections::HashMap;

use crate::chunk::Chunk;
use crate::coords::ChunkPos;
use crate::store::{ChunkStore, StoreError};

/// In-memory `ChunkStore` fake. Use it to test any code that depends on the port.
#[derive(Debug, Default)]
pub struct MemoryChunkStore {
    chunks: HashMap<ChunkPos, Chunk>,
}

impl ChunkStore for MemoryChunkStore {
    fn load(&self, pos: ChunkPos) -> Result<Option<Chunk>, StoreError> {
        Ok(self.chunks.get(&pos).cloned())
    }

    fn save(&mut self, pos: ChunkPos, chunk: &Chunk) -> Result<(), StoreError> {
        self.chunks.insert(pos, chunk.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coords::LocalPos;
    use crate::voxel::VoxelId;

    #[test]
    fn loads_nothing_for_a_chunk_never_saved() {
        let store = MemoryChunkStore::default();

        assert_eq!(store.load(ChunkPos::new(1, 2, 3)), Ok(None));
    }

    #[test]
    fn loads_what_was_saved() {
        let mut store = MemoryChunkStore::default();
        let mut chunk = Chunk::empty();
        chunk.set(LocalPos::new(1, 2, 3).expect("in bounds"), VoxelId(7));

        store
            .save(ChunkPos::new(0, 0, 0), &chunk)
            .expect("memory store never fails");

        assert_eq!(store.load(ChunkPos::new(0, 0, 0)), Ok(Some(chunk)));
    }
}
