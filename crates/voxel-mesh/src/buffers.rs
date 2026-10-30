use voxel_core::VoxelId;

use crate::quad::Quad;

/// Triangles of a quad split along its 0-2 or 1-3 diagonal, wound counter-clockwise as seen
/// from the front of a positive face; negative faces use the reversed winding.
const SPLIT_02: [u32; 6] = [0, 1, 2, 0, 2, 3];
const SPLIT_13: [u32; 6] = [1, 2, 3, 1, 3, 0];
const SPLIT_02_REVERSED: [u32; 6] = [0, 2, 1, 0, 3, 2];
const SPLIT_13_REVERSED: [u32; 6] = [1, 3, 2, 1, 0, 3];

/// Vertex data of a chunk mesh, one entry per vertex in each attribute, ready to upload.
///
/// `ao` is the occlusion level (0 darkest to 3 open); turning it into brightness is the
/// renderer's job (DESIGN.md `ao` strength). Front faces wind counter-clockwise.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MeshBuffers {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub ao: Vec<u8>,
    pub voxels: Vec<VoxelId>,
    pub indices: Vec<u32>,
}

impl MeshBuffers {
    #[must_use]
    pub fn from_quads(quads: &[Quad]) -> Self {
        let vertices = quads.len() * 4;
        let mut buffers = Self {
            positions: Vec::with_capacity(vertices),
            normals: Vec::with_capacity(vertices),
            ao: Vec::with_capacity(vertices),
            voxels: Vec::with_capacity(vertices),
            indices: Vec::with_capacity(quads.len() * 6),
        };
        for quad in quads {
            buffers.push_quad(quad);
        }
        buffers
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    #[expect(
        clippy::cast_possible_truncation,
        reason = "a chunk mesh has at most 6 * 32^3 * 4 vertices, far below u32::MAX"
    )]
    fn push_quad(&mut self, quad: &Quad) {
        let base = self.positions.len() as u32;
        let normal = quad.face.normal().map(f32::from);
        self.positions.extend(corners(quad));
        self.normals.extend([normal; 4]);
        self.ao.extend(quad.ao);
        self.voxels.extend([quad.voxel; 4]);
        let [a0, a1, a2, a3] = quad.ao.map(u32::from);
        // Split along the darker diagonal so occlusion does not show a triangle seam.
        let triangles = match (a0 + a2 <= a1 + a3, quad.face.is_positive()) {
            (true, true) => SPLIT_02,
            (false, true) => SPLIT_13,
            (true, false) => SPLIT_02_REVERSED,
            (false, false) => SPLIT_13_REVERSED,
        };
        self.indices.extend(triangles.map(|corner| base + corner));
    }
}

/// Corner positions in quad corner order (u0 v0, u1 v0, u1 v1, u0 v1), on the face's plane.
fn corners(quad: &Quad) -> [[f32; 3]; 4] {
    let (u_axis, v_axis) = quad.face.plane_axes();
    let mut origin = quad.position.map(f32::from);
    if quad.face.is_positive() {
        origin[quad.face.axis()] += 1.0;
    }
    let [width, height] = quad.size.map(f32::from);
    [(0.0, 0.0), (width, 0.0), (width, height), (0.0, height)].map(|(du, dv)| {
        let mut corner = origin;
        corner[u_axis] += du;
        corner[v_axis] += dv;
        corner
    })
}
