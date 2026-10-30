use serde::{Deserialize, Serialize};

use crate::coords::LocalPos;
use crate::voxel::VoxelId;

/// Edge length of a chunk, in voxels.
pub const CHUNK_EDGE: usize = 32;
/// Number of voxels in a chunk.
pub const CHUNK_VOLUME: usize = CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE;

/// A cube of voxels. A chunk of one kind only (all air, all stone) is stored as that kind,
/// which is most chunks of a world; others keep one id per voxel (ADR 0012).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "Storage", into = "Storage")]
pub struct Chunk {
    storage: Storage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Storage {
    Uniform(VoxelId),
    Voxels(Box<[VoxelId]>),
}

/// Why serialized chunk data was rejected.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("chunk has {found} voxels, expected {CHUNK_VOLUME}")]
pub struct WrongVoxelCount {
    found: usize,
}

impl TryFrom<Storage> for Chunk {
    type Error = WrongVoxelCount;

    fn try_from(storage: Storage) -> Result<Self, Self::Error> {
        match &storage {
            Storage::Voxels(voxels) if voxels.len() != CHUNK_VOLUME => Err(WrongVoxelCount {
                found: voxels.len(),
            }),
            _ => Ok(Self { storage }),
        }
    }
}

impl From<Chunk> for Storage {
    fn from(chunk: Chunk) -> Self {
        chunk.storage
    }
}

impl Chunk {
    #[must_use]
    pub const fn empty() -> Self {
        Self::uniform(VoxelId::AIR)
    }

    /// A chunk filled with one kind of voxel.
    #[must_use]
    pub const fn uniform(voxel: VoxelId) -> Self {
        Self {
            storage: Storage::Uniform(voxel),
        }
    }

    /// Builds a chunk by asking `voxel_at` for every position. Generators use this.
    #[must_use]
    pub fn from_fn(voxel_at: impl FnMut(LocalPos) -> VoxelId) -> Self {
        let voxels: Box<[VoxelId]> = LocalPos::all().map(voxel_at).collect();
        match voxels.first() {
            Some(&first) if voxels.iter().all(|&voxel| voxel == first) => Self::uniform(first),
            _ => Self {
                storage: Storage::Voxels(voxels),
            },
        }
    }

    /// The only kind of voxel in the chunk, when it is stored as one kind.
    #[must_use]
    pub const fn as_uniform(&self) -> Option<VoxelId> {
        match self.storage {
            Storage::Uniform(voxel) => Some(voxel),
            Storage::Voxels(_) => None,
        }
    }

    #[must_use]
    pub fn get(&self, pos: LocalPos) -> VoxelId {
        match &self.storage {
            Storage::Uniform(voxel) => *voxel,
            Storage::Voxels(voxels) => voxels[pos.index()],
        }
    }

    /// Sets one voxel. A uniform chunk expands to one id per voxel on its first differing edit.
    pub fn set(&mut self, pos: LocalPos, voxel: VoxelId) {
        match &mut self.storage {
            Storage::Uniform(current) if *current == voxel => {}
            Storage::Uniform(current) => {
                let mut voxels = vec![*current; CHUNK_VOLUME].into_boxed_slice();
                voxels[pos.index()] = voxel;
                self.storage = Storage::Voxels(voxels);
            }
            Storage::Voxels(voxels) => voxels[pos.index()] = voxel,
        }
    }

    /// Bytes this chunk keeps on the heap, for memory budgets.
    #[must_use]
    pub const fn heap_bytes(&self) -> usize {
        match &self.storage {
            Storage::Uniform(_) => 0,
            Storage::Voxels(voxels) => voxels.len() * size_of::<VoxelId>(),
        }
    }
}

/// Chunks are equal when every voxel is, however they are stored.
impl PartialEq for Chunk {
    fn eq(&self, other: &Self) -> bool {
        match (&self.storage, &other.storage) {
            (Storage::Uniform(a), Storage::Uniform(b)) => a == b,
            (Storage::Voxels(a), Storage::Voxels(b)) => a == b,
            (Storage::Uniform(a), Storage::Voxels(b))
            | (Storage::Voxels(b), Storage::Uniform(a)) => b.iter().all(|voxel| voxel == a),
        }
    }
}

impl Eq for Chunk {}

impl Default for Chunk {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::edit_error::EditError;

    fn local(x: u8, y: u8, z: u8) -> LocalPos {
        LocalPos::new(x, y, z).expect("in bounds")
    }

    #[test]
    fn a_new_chunk_is_all_air() {
        let chunk = Chunk::empty();

        assert!(chunk.get(local(0, 0, 0)).is_air());
        assert!(chunk.get(local(31, 31, 31)).is_air());
    }

    #[test]
    fn from_fn_stores_what_the_function_returns_at_each_position() {
        let chunk = Chunk::from_fn(|pos| VoxelId(u16::from(pos.y())));

        assert_eq!(chunk.get(local(5, 0, 9)), VoxelId(0));
        assert_eq!(chunk.get(local(0, 17, 31)), VoxelId(17));
        assert_eq!(chunk.get(local(31, 31, 0)), VoxelId(31));
    }

    #[test]
    fn a_chunk_of_one_kind_needs_no_voxel_storage() {
        let stone = Chunk::from_fn(|_| VoxelId(3));

        assert_eq!(stone.as_uniform(), Some(VoxelId(3)));
        assert_eq!(stone.heap_bytes(), 0);
        assert_eq!(Chunk::empty().as_uniform(), Some(VoxelId::AIR));
    }

    #[test]
    fn editing_a_uniform_chunk_keeps_every_other_voxel() {
        let mut chunk = Chunk::uniform(VoxelId(3));

        chunk.set(local(4, 5, 6), VoxelId::AIR);

        assert_eq!(chunk.get(local(4, 5, 6)), VoxelId::AIR);
        assert_eq!(chunk.get(local(4, 5, 7)), VoxelId(3));
        assert_eq!(chunk.as_uniform(), None);
    }

    #[test]
    fn equality_ignores_how_the_chunk_is_stored() {
        let mut expanded = Chunk::uniform(VoxelId(3));
        expanded.set(local(0, 0, 0), VoxelId(4));
        expanded.set(local(0, 0, 0), VoxelId(3));

        assert_eq!(expanded, Chunk::uniform(VoxelId(3)));
        assert_ne!(expanded, Chunk::uniform(VoxelId(4)));
    }

    #[test]
    fn round_trips_through_postcard() {
        let mut chunk = Chunk::uniform(VoxelId(2));
        chunk.set(local(1, 2, 3), VoxelId(9));

        for original in [chunk, Chunk::empty()] {
            let bytes = postcard::to_stdvec(&original).expect("serializes");
            let decoded: Chunk = postcard::from_bytes(&bytes).expect("deserializes");
            assert_eq!(decoded, original);
        }
    }

    #[test]
    fn rejects_serialized_voxels_of_the_wrong_length() {
        let bytes = postcard::to_stdvec(&Storage::Voxels(vec![VoxelId(1); 5].into_boxed_slice()))
            .expect("serializes");

        assert!(postcard::from_bytes::<Chunk>(&bytes).is_err());
    }

    #[test]
    fn rejects_a_position_outside_the_chunk() {
        let result = LocalPos::new(32, 0, 0);

        assert!(matches!(result, Err(EditError::OutOfBounds { x: 32, .. })));
    }

    proptest! {
        #[test]
        fn an_edit_is_read_back_and_touches_nothing_else(
            x in 0u8..32, y in 0u8..32, z in 0u8..32, id in 1u16..,
            other_x in 0u8..32, other_y in 0u8..32, other_z in 0u8..32,
        ) {
            let mut chunk = Chunk::empty();
            let edited = local(x, y, z);

            chunk.set(edited, VoxelId(id));

            prop_assert_eq!(chunk.get(edited), VoxelId(id));
            let other = local(other_x, other_y, other_z);
            if other != edited {
                prop_assert!(chunk.get(other).is_air());
            }
        }
    }
}
