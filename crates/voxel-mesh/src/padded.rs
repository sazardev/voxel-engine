use voxel_core::{CHUNK_EDGE, Chunk, LocalPos, VoxelId};

use crate::chunk_edge::{EDGE, EDGE_U8};
use crate::face::Face;

pub(crate) const PADDED_EDGE: usize = CHUNK_EDGE + 2;
const PADDED_VOLUME: usize = PADDED_EDGE * PADDED_EDGE * PADDED_EDGE;
#[expect(
    clippy::cast_possible_wrap,
    reason = "PADDED_EDGE is 34, far below isize::MAX"
)]
const PADDED_EDGE_SIGNED: isize = PADDED_EDGE as isize;
/// Storage distance between neighbouring voxels along x, y and z.
const STRIDES: [isize; 3] = [
    1,
    PADDED_EDGE_SIGNED,
    PADDED_EDGE_SIGNED * PADDED_EDGE_SIGNED,
];

/// A chunk plus a one-voxel border copied from its 26 neighbours: everything meshing reads.
///
/// Coordinates run from -1 to 32 on each axis; 0..32 is the chunk itself.
#[derive(Debug, Clone)]
pub struct PaddedChunk {
    voxels: Vec<VoxelId>,
}

impl PaddedChunk {
    /// `neighbour([dx, dy, dz])` returns the chunk at that offset (each -1, 0 or 1, never all
    /// zero). A missing neighbour reads as air, so its border faces are meshed until it loads.
    #[must_use]
    pub fn from_chunks<'a>(
        center: &'a Chunk,
        neighbour: impl Fn([i32; 3]) -> Option<&'a Chunk>,
    ) -> Self {
        let mut padded = Self {
            voxels: vec![VoxelId::AIR; PADDED_VOLUME],
        };
        for dz in -1..=1 {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let offset = [dx, dy, dz];
                    let source = if offset == [0, 0, 0] {
                        Some(center)
                    } else {
                        neighbour(offset)
                    };
                    if let Some(chunk) = source {
                        padded.copy_border(chunk, offset);
                    }
                }
            }
        }
        padded
    }

    /// The voxel at `pos`; anything outside the padded range reads as air.
    #[must_use]
    pub fn voxel(&self, pos: [i32; 3]) -> VoxelId {
        index(pos).map_or(VoxelId::AIR, |index| self.voxels[index])
    }

    /// Every voxel in storage order: x fastest, then y, then z, each from -1 to 32.
    pub(crate) fn voxels(&self) -> &[VoxelId] {
        &self.voxels
    }

    /// Storage index of the chunk voxel at local `pos`.
    pub(crate) fn interior_index(pos: [u8; 3]) -> usize {
        let [x, y, z] = pos.map(|axis| usize::from(axis) + 1);
        x + PADDED_EDGE * (y + PADDED_EDGE * z)
    }

    /// The voxel `offset` storage steps from `index` (see [`axis_step`]); outside reads as air.
    pub(crate) fn voxel_near(&self, index: usize, offset: isize) -> VoxelId {
        index
            .checked_add_signed(offset)
            .and_then(|index| self.voxels.get(index))
            .copied()
            .unwrap_or(VoxelId::AIR)
    }

    /// Copies the part of the chunk at `offset` that falls inside the padded range.
    fn copy_border(&mut self, chunk: &Chunk, offset: [i32; 3]) {
        let [xs, ys, zs] = offset.map(source_range);
        for z in zs {
            for y in ys.clone() {
                for x in xs.clone() {
                    let Ok(local) = LocalPos::new(x, y, z) else {
                        continue;
                    };
                    let pos = [x, y, z].map(i32::from);
                    let target = [0, 1, 2].map(|axis| pos[axis] + offset[axis] * EDGE);
                    if let Some(index) = index(target) {
                        self.voxels[index] = chunk.get(local);
                    }
                }
            }
        }
    }
}

/// Storage offset of one voxel along `axis` (0 is x, 1 is y, 2 is z).
pub(crate) const fn axis_step(axis: usize) -> isize {
    STRIDES[axis]
}

/// Storage offset from a voxel to the one in front of its `face`.
pub(crate) const fn front_step(face: Face) -> isize {
    if face.is_positive() {
        STRIDES[face.axis()]
    } else {
        -STRIDES[face.axis()]
    }
}

/// Local coordinates of a neighbour, along one axis, that land in the padding.
fn source_range(offset: i32) -> std::ops::Range<u8> {
    match offset {
        -1 => EDGE_U8 - 1..EDGE_U8,
        0 => 0..EDGE_U8,
        _ => 0..1,
    }
}

fn index([x, y, z]: [i32; 3]) -> Option<usize> {
    let shifted = |axis: i32| {
        usize::try_from(axis + 1)
            .ok()
            .filter(|&axis| axis < PADDED_EDGE)
    };
    Some(shifted(x)? + PADDED_EDGE * (shifted(y)? + PADDED_EDGE * shifted(z)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_center_and_the_neighbour_borders() {
        let mut center = Chunk::empty();
        center.set(LocalPos::new(0, 0, 0).expect("in bounds"), VoxelId(5));
        let east = Chunk::from_fn(|pos| VoxelId(u16::from(pos.y()) + 10));

        let padded =
            PaddedChunk::from_chunks(&center, |offset| (offset == [1, 0, 0]).then_some(&east));

        assert_eq!(padded.voxel([0, 0, 0]), VoxelId(5));
        assert_eq!(padded.voxel([32, 7, 3]), VoxelId(17));
        assert_eq!(padded.voxel([-1, 7, 3]), VoxelId::AIR);
        assert_eq!(padded.voxel([33, 0, 0]), VoxelId::AIR);
    }

    #[test]
    fn steps_reach_the_neighbouring_voxels() {
        let center = Chunk::from_fn(|pos| VoxelId(u16::from(pos.x()) + 100 * u16::from(pos.z())));
        let padded = PaddedChunk::from_chunks(&center, |_| None);
        let index = PaddedChunk::interior_index([5, 6, 7]);

        assert_eq!(padded.voxel_near(index, 0), VoxelId(705));
        assert_eq!(
            padded.voxel_near(index, front_step(Face::PosX)),
            VoxelId(706)
        );
        assert_eq!(
            padded.voxel_near(index, front_step(Face::NegZ)),
            VoxelId(605)
        );
        assert_eq!(
            padded.voxel_near(index, axis_step(2) - axis_step(0)),
            VoxelId(804)
        );
    }
}
