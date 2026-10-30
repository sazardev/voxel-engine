//! Far terrain as a height field: one mesh per square patch of coarse columns, with tops and
//! walls only, no voxel storage (ADR 0016).

use voxel_core::VoxelId;

use crate::buffers::MeshBuffers;
use crate::face::Face;

/// Occlusion level of an open corner: far terrain is not occluded per vertex.
const OPEN: [u8; 4] = [3; 4];

/// A square patch of columns to mesh.
#[derive(Debug, Clone, Copy)]
pub struct HeightPatch<'a> {
    /// Columns per side of the patch.
    pub cells: usize,
    /// Height of the top voxel of each column, for the patch and a one-column border around it:
    /// `(cells + 2)²` values, index `(z + 1) * (cells + 2) + (x + 1)`.
    pub heights: &'a [i64],
    pub top: VoxelId,
    pub side: VoxelId,
    /// How far below the lower side the walls on the patch's outer edges reach, in voxels, so
    /// patches of other detail levels never show a gap between them.
    pub skirt: i64,
    /// Subtracted from every height so vertex coordinates stay small.
    pub base: i64,
}

impl HeightPatch<'_> {
    fn height(&self, x: usize, z: usize) -> i64 {
        self.heights[(z + 1) * (self.cells + 2) + x + 1]
    }

    /// Height of the neighbour of (`x`, `z`) one column toward `step`; may be in the border.
    fn neighbour(&self, x: usize, z: usize, (dx, dz): (isize, isize)) -> i64 {
        let column = |axis: usize, step: isize| axis.wrapping_add_signed(step + 1);
        self.heights[column(z, dz) * (self.cells + 2) + column(x, dx)]
    }

    /// Top of column (`x`, `z`) as a mesh coordinate.
    #[expect(
        clippy::cast_precision_loss,
        reason = "heights relative to the base are a few hundred voxels, exact in f32"
    )]
    fn level(&self, height: i64) -> f32 {
        (height + 1 - self.base) as f32
    }
}

/// Meshes the patch in patch units: x and z in columns, y in voxels above `base`.
#[must_use]
pub fn mesh_heightfield(patch: &HeightPatch) -> MeshBuffers {
    let mut buffers = MeshBuffers::default();
    push_tops(patch, &mut buffers);
    push_walls(patch, &mut buffers);
    buffers
}

/// Tops merged greedily: the widest run of equal heights in a row, then as many rows as match.
fn push_tops(patch: &HeightPatch, buffers: &mut MeshBuffers) {
    let cells = patch.cells;
    let mut done = vec![false; cells * cells];
    for z in 0..cells {
        let mut x = 0;
        while x < cells {
            if done[z * cells + x] {
                x += 1;
                continue;
            }
            let height = patch.height(x, z);
            let same =
                |cx: usize, cz: usize| !done[cz * cells + cx] && patch.height(cx, cz) == height;
            let width = (x..cells).take_while(|&end| same(end, z)).count();
            let depth = (z..cells)
                .take_while(|&row| (x..x + width).all(|col| same(col, row)))
                .count();
            for row in z..z + depth {
                done[row * cells + x..row * cells + x + width].fill(true);
            }
            let y = patch.level(height);
            let (x0, x1, z0, z1) = (to_f32(x), to_f32(x + width), to_f32(z), to_f32(z + depth));
            // +y: u is z, v is x.
            let corners = [[x0, y, z0], [x0, y, z1], [x1, y, z1], [x1, y, z0]];
            buffers.push_face(Face::PosY, corners, patch.top, OPEN);
            x += width;
        }
    }
}

/// A wall wherever a column stands above its neighbour, and a skirt on every outer edge. Walls
/// in a line with the same span merge into one face.
fn push_walls(patch: &HeightPatch, buffers: &mut MeshBuffers) {
    let sides = [
        (Face::PosX, (1, 0)),
        (Face::NegX, (-1, 0)),
        (Face::PosZ, (0, 1)),
        (Face::NegZ, (0, -1)),
    ];
    for (face, step) in sides {
        for line in 0..patch.cells {
            let mut run: Option<(usize, (i64, i64))> = None;
            for along in 0..=patch.cells {
                let span = (along < patch.cells)
                    .then(|| wall_span(patch, face, step, line, along))
                    .flatten();
                if run.is_some_and(|(_, current)| Some(current) == span) {
                    continue;
                }
                if let Some((start, (bottom, top))) = run {
                    let span = (patch.level(bottom), patch.level(top));
                    buffers.push_face(
                        face,
                        wall_corners(face, line, (start, along), span),
                        patch.side,
                        OPEN,
                    );
                }
                run = span.map(|span| (along, span));
            }
        }
    }
}

/// The wall of one column toward `step`, as (bottom, top) heights, if it has one. X walls run
/// along z (`line` is x), z walls along x (`line` is z).
fn wall_span(
    patch: &HeightPatch,
    face: Face,
    step: (isize, isize),
    line: usize,
    along: usize,
) -> Option<(i64, i64)> {
    let (x, z) = if face.axis() == 0 {
        (line, along)
    } else {
        (along, line)
    };
    let height = patch.height(x, z);
    let neighbour = patch.neighbour(x, z, step);
    let last = patch.cells - 1;
    let on_edge = match step {
        (1, _) => x == last,
        (-1, _) => x == 0,
        (_, 1) => z == last,
        _ => z == 0,
    };
    let bottom = if on_edge {
        height.min(neighbour) - patch.skirt
    } else {
        neighbour
    };
    (bottom < height).then_some((bottom, height))
}

/// Corners of a wall facing `face` on `line`, spanning columns `from..to` along it and heights
/// `bottom..top`, in quad order on the face's plane axes (x faces: u is y, v is z; z faces: u is
/// x, v is y).
fn wall_corners(
    face: Face,
    line: usize,
    (from, to): (usize, usize),
    (bottom, top): (f32, f32),
) -> [[f32; 3]; 4] {
    let (start, end) = (to_f32(from), to_f32(to));
    let plane = if face.is_positive() {
        to_f32(line + 1)
    } else {
        to_f32(line)
    };
    if face.axis() == 0 {
        [
            [plane, bottom, start],
            [plane, top, start],
            [plane, top, end],
            [plane, bottom, end],
        ]
    } else {
        [
            [start, bottom, plane],
            [end, bottom, plane],
            [end, top, plane],
            [start, top, plane],
        ]
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "patch coordinates are at most a few dozen columns"
)]
fn to_f32(value: usize) -> f32 {
    value as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    const GRASS: VoxelId = VoxelId(1);
    const DIRT: VoxelId = VoxelId(2);

    fn patch(heights: &[i64], cells: usize) -> MeshBuffers {
        mesh_heightfield(&HeightPatch {
            cells,
            heights,
            top: GRASS,
            side: DIRT,
            skirt: 4,
            base: 0,
        })
    }

    fn faces(buffers: &MeshBuffers) -> usize {
        buffers.indices.len() / 6
    }

    fn faces_with_normal(buffers: &MeshBuffers, normal: [f32; 3]) -> usize {
        buffers
            .normals
            .iter()
            .filter(|n| n.iter().zip(normal).all(|(a, b)| (a - b).abs() < 1e-6))
            .count()
            / 4
    }

    #[test]
    fn a_flat_patch_is_one_top_and_one_skirt_per_edge() {
        let buffers = patch(&[10; 16], 2);

        assert_eq!(faces_with_normal(&buffers, [0.0, 1.0, 0.0]), 1);
        assert_eq!(faces(&buffers), 1 + 4);
        let lowest = buffers
            .positions
            .iter()
            .map(|p| p[1])
            .fold(f32::MAX, f32::min);
        assert!((lowest - (10.0 + 1.0 - 4.0)).abs() < 1e-6);
    }

    #[test]
    fn a_higher_column_gets_a_wall_down_to_its_neighbour() {
        // 2×2 patch inside a border of height 10; column (0, 0) is 13.
        let mut heights = [10; 16];
        heights[5] = 13;

        let buffers = patch(&heights, 2);

        let east_walls: Vec<f32> = buffers
            .positions
            .as_chunks::<4>()
            .0
            .iter()
            .zip(buffers.normals.as_chunks::<4>().0)
            .filter(|(_, n)| n[0][0] > 0.5)
            .filter(|(p, _)| (p[0][0] - 1.0).abs() < 1e-6)
            .flat_map(|(p, _)| p.iter().map(|corner| corner[1]))
            .collect();
        assert!(
            east_walls.contains(&11.0) && east_walls.contains(&14.0),
            "{east_walls:?}"
        );
    }

    #[test]
    fn walls_face_out_of_the_column() {
        let mut heights = [10; 16];
        heights[5] = 13;

        let buffers = patch(&heights, 2);

        for triangle in buffers.indices.as_chunks::<3>().0 {
            let [a, b, c] = [0, 1, 2].map(|i| buffers.positions[triangle[i] as usize]);
            let normal = buffers.normals[triangle[0] as usize];
            let (ab, ac) = (
                [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
                [c[0] - a[0], c[1] - a[1], c[2] - a[2]],
            );
            let cross = [
                ab[1] * ac[2] - ab[2] * ac[1],
                ab[2] * ac[0] - ab[0] * ac[2],
                ab[0] * ac[1] - ab[1] * ac[0],
            ];
            let facing: f32 = cross.iter().zip(normal).map(|(a, b)| a * b).sum();
            assert!(facing > 0.0, "{triangle:?} faces inward");
        }
    }
}
