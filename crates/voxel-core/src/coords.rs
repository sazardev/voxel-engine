use crate::chunk::CHUNK_EDGE;
use crate::edit_error::EditError;

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

    /// Index into a chunk's flat storage (x fastest, then y, then z).
    #[must_use]
    pub fn index(self) -> usize {
        usize::from(self.x)
            + usize::from(self.y) * CHUNK_EDGE
            + usize::from(self.z) * CHUNK_EDGE * CHUNK_EDGE
    }
}
