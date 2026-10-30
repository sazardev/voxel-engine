use crate::chunk::{CHUNK_EDGE, CHUNK_VOLUME};
use crate::edit_error::EditError;

/// `LocalPos` stores axes as `u8` and decodes indices with shifts and masks.
const _: () = assert!(CHUNK_EDGE <= 256 && CHUNK_EDGE.is_power_of_two());

/// Position of a chunk in the world, in chunk units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkPos {
    #[must_use]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// The chunk holding world voxel `voxel`, and the voxel's place in it. `None` beyond the
    /// range of chunk coordinates.
    #[must_use]
    pub fn split(voxel: [i64; 3]) -> Option<(Self, LocalPos)> {
        let edge = i64::try_from(CHUNK_EDGE).ok()?;
        let [x, y, z] = voxel.map(|axis| i32::try_from(axis.div_euclid(edge)).ok());
        let [lx, ly, lz] = voxel.map(|axis| u8::try_from(axis.rem_euclid(edge)).ok());
        Some((Self::new(x?, y?, z?), LocalPos::new(lx?, ly?, lz?).ok()?))
    }
}

/// Position of a voxel inside one chunk. Always in bounds once constructed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalPos {
    x: u8,
    y: u8,
    z: u8,
}

impl LocalPos {
    /// # Errors
    /// Returns [`EditError::OutOfBounds`] when any axis is `CHUNK_EDGE` or more.
    pub fn new(x: u8, y: u8, z: u8) -> Result<Self, EditError> {
        let inside = |axis: u8| usize::from(axis) < CHUNK_EDGE;
        if inside(x) && inside(y) && inside(z) {
            Ok(Self { x, y, z })
        } else {
            Err(EditError::OutOfBounds {
                x,
                y,
                z,
                edge: CHUNK_EDGE,
            })
        }
    }

    /// Every position in a chunk, in storage order (see [`LocalPos::index`]).
    pub fn all() -> impl ExactSizeIterator<Item = Self> {
        (0..CHUNK_VOLUME).map(Self::from_index)
    }

    /// Inverse of [`LocalPos::index`] for indices below `CHUNK_VOLUME`.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "each axis is reduced modulo CHUNK_EDGE (32) before the cast"
    )]
    const fn from_index(index: usize) -> Self {
        Self {
            x: (index % CHUNK_EDGE) as u8,
            y: (index / CHUNK_EDGE % CHUNK_EDGE) as u8,
            z: (index / (CHUNK_EDGE * CHUNK_EDGE) % CHUNK_EDGE) as u8,
        }
    }

    #[must_use]
    pub const fn x(self) -> u8 {
        self.x
    }

    #[must_use]
    pub const fn y(self) -> u8 {
        self.y
    }

    #[must_use]
    pub const fn z(self) -> u8 {
        self.z
    }

    /// Index into a chunk's flat storage (x fastest, then y, then z).
    #[must_use]
    pub fn index(self) -> usize {
        usize::from(self.x)
            + usize::from(self.y) * CHUNK_EDGE
            + usize::from(self.z) * CHUNK_EDGE * CHUNK_EDGE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_visits_every_position_once_in_storage_order() {
        let positions: Vec<LocalPos> = LocalPos::all().collect();

        assert_eq!(positions.len(), CHUNK_VOLUME);
        for (expected_index, pos) in positions.iter().enumerate() {
            assert_eq!(pos.index(), expected_index);
        }
    }

    #[test]
    fn splits_world_voxels_into_chunk_and_local_position() {
        let (chunk, local) = ChunkPos::split([33, -1, -32]).expect("in range");

        assert_eq!(chunk, ChunkPos::new(1, -1, -1));
        assert_eq!((local.x(), local.y(), local.z()), (1, 31, 0));
        assert_eq!(ChunkPos::split([i64::MAX, 0, 0]), None);
    }

    #[test]
    fn exposes_the_axes_it_was_built_with() {
        let pos = LocalPos::new(1, 2, 3).expect("in bounds");

        assert_eq!((pos.x(), pos.y(), pos.z()), (1, 2, 3));
    }
}
