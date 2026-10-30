//! Per-vertex ambient occlusion: how enclosed each corner of a face is (DESIGN.md `ao`).

use crate::face::Face;
use crate::padded::{PaddedChunk, axis_step, front_step};

/// Occlusion of each corner of `face` on the voxel stored at `index`, in [`crate::Quad`]
/// corner order.
pub(crate) fn face_ao(padded: &PaddedChunk, face: Face, index: usize) -> [u8; 4] {
    let (u, v) = face.plane_axes();
    let (front, u_step, v_step) = (front_step(face), axis_step(u), axis_step(v));
    let solid = |du: isize, dv: isize| {
        !padded
            .voxel_near(index, front + du * u_step + dv * v_step)
            .is_air()
    };
    let corner = |du, dv| vertex_ao(solid(du, 0), solid(0, dv), solid(du, dv));
    [corner(-1, -1), corner(1, -1), corner(1, 1), corner(-1, 1)]
}

/// 3 when open, one less per touching voxel; two sides already close the corner completely.
fn vertex_ao(side_u: bool, side_v: bool, diagonal: bool) -> u8 {
    if side_u && side_v {
        0
    } else {
        3 - u8::from(side_u) - u8::from(side_v) - u8::from(diagonal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_touching_voxels_and_closes_a_corner_between_two_sides() {
        assert_eq!(vertex_ao(false, false, false), 3);
        assert_eq!(vertex_ao(false, false, true), 2);
        assert_eq!(vertex_ao(true, false, true), 1);
        assert_eq!(vertex_ao(true, true, false), 0);
    }
}
