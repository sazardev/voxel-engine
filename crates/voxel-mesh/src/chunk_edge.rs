//! The chunk edge in the integer types meshing loops and offsets use.

use voxel_core::CHUNK_EDGE;

#[expect(
    clippy::cast_possible_truncation,
    reason = "CHUNK_EDGE is 32, checked by the assertion below"
)]
pub(crate) const EDGE_U8: u8 = CHUNK_EDGE as u8;
pub(crate) const EDGE: i32 = EDGE_U8 as i32;
const _: () = assert!(EDGE_U8 as usize == CHUNK_EDGE);
