use voxel_core::VoxelId;

use crate::face::Face;

/// A rectangle of identical voxel faces in one plane of a chunk.
///
/// `position` is the local voxel at the quad's minimum corner; the quad spans `size[0]` voxels
/// along the face's u axis and `size[1]` along its v axis (see [`Face::plane_axes`]).
/// `ao` holds the occlusion level of each corner, from 0 (darkest) to 3 (open), in the order
/// (u0 v0, u1 v0, u1 v1, u0 v1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quad {
    pub face: Face,
    pub position: [u8; 3],
    pub size: [u8; 2],
    pub voxel: VoxelId,
    pub ao: [u8; 4],
}
