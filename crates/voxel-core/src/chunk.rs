use serde::{Deserialize, Serialize};

use crate::coords::LocalPos;
use crate::voxel::VoxelId;

/// Edge length of a chunk, in voxels.
pub const CHUNK_EDGE: usize = 32;
/// Number of voxels in a chunk.
pub const CHUNK_VOLUME: usize = CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE;

/// A cube of voxels. Flat storage for now; palette compression comes later behind this same API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chunk {
    voxels: Vec<VoxelId>,
}

impl Chunk {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            voxels: vec![VoxelId::AIR; CHUNK_VOLUME],
        }
    }

    #[must_use]
    pub fn get(&self, pos: LocalPos) -> VoxelId {
        self.voxels[pos.index()]
    }

    pub fn set(&mut self, pos: LocalPos, voxel: VoxelId) {
        self.voxels[pos.index()] = voxel;
    }
}

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
