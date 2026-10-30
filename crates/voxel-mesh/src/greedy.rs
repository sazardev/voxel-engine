//! Visible faces merged greedily, slice by slice, into as few quads as AO allows (ADR 0011).

use voxel_core::VoxelId;

use crate::ambient_occlusion::face_ao;
use crate::chunk_edge::EDGE_U8;
use crate::face::Face;
use crate::padded::PaddedChunk;
use crate::quad::Quad;
use crate::solid_columns::SolidColumns;

const SIDE: usize = EDGE_U8 as usize;

/// Visible faces of one slice: bit `u` of row `v`.
type SliceRows = [u32; SIDE];

/// What two faces must share to merge: material and the occlusion of all four corners.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FaceKey {
    voxel: VoxelId,
    ao: [u8; 4],
}

/// Every visible face of the chunk (solid voxel next to air), merged into quads.
#[must_use]
pub fn mesh_quads(padded: &PaddedChunk) -> Vec<Quad> {
    let columns = SolidColumns::new(padded);
    let mut quads = Vec::new();
    for face in Face::ALL {
        for (slice, rows) in (0..EDGE_U8).zip(visible_rows(&columns, face)) {
            if rows.iter().any(|&row| row != 0) {
                merge_slice(padded, face, slice, rows, &mut quads);
            }
        }
    }
    quads
}

/// Visible faces of every slice, regrouped from columns into rows.
fn visible_rows(columns: &SolidColumns, face: Face) -> [SliceRows; SIDE] {
    let mut slices = [[0; SIDE]; SIDE];
    let cells = (0..SIDE).flat_map(|v| (0..SIDE).map(move |u| (u, v)));
    for (u, v) in cells {
        let mut bits = columns.visible(face, u, v);
        while bits != 0 {
            slices[bits.trailing_zeros() as usize][v] |= 1 << u;
            bits &= bits - 1;
        }
    }
    slices
}

/// Local position of the voxel at (`u`, `v`) in slice `slice` of `face`.
fn voxel_position(face: Face, slice: u8, u: usize, v: usize) -> [u8; 3] {
    let (u_axis, v_axis) = face.plane_axes();
    let mut pos = [0; 3];
    pos[face.axis()] = slice;
    pos[u_axis] = to_u8(u);
    pos[v_axis] = to_u8(v);
    pos
}

/// Material and occlusion of each visible face of the slice, indexed `v * SIDE + u`.
fn face_keys(
    padded: &PaddedChunk,
    face: Face,
    slice: u8,
    rows: &SliceRows,
) -> Vec<Option<FaceKey>> {
    let mut keys = vec![None; SIDE * SIDE];
    for (v, &row) in rows.iter().enumerate() {
        let mut bits = row;
        while bits != 0 {
            let u = bits.trailing_zeros() as usize;
            let index = PaddedChunk::interior_index(voxel_position(face, slice, u, v));
            keys[v * SIDE + u] = Some(FaceKey {
                voxel: padded.voxel_near(index, 0),
                ao: face_ao(padded, face, index),
            });
            bits &= bits - 1;
        }
    }
    keys
}

/// Emits the widest, then tallest, rectangle of equal faces at each first unmeshed face.
fn merge_slice(
    padded: &PaddedChunk,
    face: Face,
    slice: u8,
    mut rows: SliceRows,
    quads: &mut Vec<Quad>,
) {
    let keys = face_keys(padded, face, slice, &rows);
    let matches = |rows: &SliceRows, u: usize, v: usize, key| {
        rows[v] & (1 << u) != 0 && keys[v * SIDE + u] == key
    };
    for v in 0..SIDE {
        while rows[v] != 0 {
            let u = rows[v].trailing_zeros() as usize;
            let key = keys[v * SIDE + u];
            let width = (u..SIDE)
                .take_while(|&end| matches(&rows, end, v, key))
                .count();
            let height = (v..SIDE)
                .take_while(|&row| (u..u + width).all(|col| matches(&rows, col, row, key)))
                .count();
            let span = (u32::MAX >> (SIDE - width)) << u;
            rows[v..v + height].iter_mut().for_each(|row| *row &= !span);
            if let Some(key) = key {
                quads.push(Quad {
                    face,
                    position: voxel_position(face, slice, u, v),
                    size: [to_u8(width), to_u8(height)],
                    voxel: key.voxel,
                    ao: key.ao,
                });
            }
        }
    }
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "slice coordinates and extents are at most CHUNK_EDGE (32)"
)]
const fn to_u8(value: usize) -> u8 {
    value as u8
}
