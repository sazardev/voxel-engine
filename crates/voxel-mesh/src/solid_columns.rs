//! Solid voxels as bit columns, so visible faces are found 32 at a time (ADR 0011).

use crate::chunk_edge::EDGE_U8;
use crate::face::Face;
use crate::padded::{PADDED_EDGE, PaddedChunk};

const SIDE: usize = EDGE_U8 as usize;

/// For each axis, one column per (u, v) of the chunk: bit `p` is set when the voxel at padded
/// coordinate `p - 1` along the axis is solid. Bits 0 and 33 are the neighbours' border.
#[derive(Debug)]
pub(crate) struct SolidColumns {
    columns: [Vec<u64>; 3],
}

impl SolidColumns {
    pub fn new(padded: &PaddedChunk) -> Self {
        let mut columns = [0; 3].map(|_| vec![0_u64; SIDE * SIDE]);
        for (index, voxel) in padded.voxels().iter().enumerate() {
            if voxel.is_air() {
                continue;
            }
            let pos = [
                index % PADDED_EDGE,
                index / PADDED_EDGE % PADDED_EDGE,
                index / (PADDED_EDGE * PADDED_EDGE),
            ];
            for (axis, axis_columns) in columns.iter_mut().enumerate() {
                let (u, v) = (pos[(axis + 1) % 3], pos[(axis + 2) % 3]);
                if (1..=SIDE).contains(&u) && (1..=SIDE).contains(&v) {
                    axis_columns[(v - 1) * SIDE + (u - 1)] |= 1 << pos[axis];
                }
            }
        }
        Self { columns }
    }

    /// Slices of column (`u`, `v`) where `face` is visible: bit `s` is set when the voxel in
    /// slice `s` is solid and the one in front of `face` is not.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "keeps the 32 chunk slices; the border bit above them is dropped on purpose"
    )]
    pub fn visible(&self, face: Face, u: usize, v: usize) -> u32 {
        let column = self.columns[face.axis()][v * SIDE + u];
        let in_front = if face.is_positive() {
            column >> 1
        } else {
            column << 1
        };
        ((column & !in_front) >> 1) as u32
    }
}
