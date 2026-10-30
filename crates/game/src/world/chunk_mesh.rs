//! Turns the pure mesher's buffers into a Bevy `Mesh` with palette colors and AO baked in.

use bevy::asset::RenderAssetUsages;
use bevy::color::ColorToComponents;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use voxel_mesh::MeshBuffers;

use super::voxel_colors::VoxelColors;
use super::voxel_tint::TINT_SIZE;

/// A render-only mesh: the CPU copy is dropped after upload, since voxels stay the source of
/// truth. `origin` is the chunk's corner in world voxels; it aligns the per-voxel tint across
/// chunks.
#[must_use]
pub fn chunk_mesh(buffers: MeshBuffers, colors: &VoxelColors, origin: [i64; 3]) -> Mesh {
    tinted_mesh(buffers, colors, origin, [1.0; 3])
}

/// Like [`chunk_mesh`] for meshes whose units are `scale` voxels per axis (coarse patches).
#[must_use]
pub fn tinted_mesh(
    buffers: MeshBuffers,
    colors: &VoxelColors,
    origin: [i64; 3],
    scale: [f32; 3],
) -> Mesh {
    let vertex_colors: Vec<[f32; 4]> = buffers
        .voxels
        .iter()
        .zip(&buffers.ao)
        .map(|(&voxel, &ao)| colors.shaded(voxel, ao).to_f32_array())
        .collect();
    let uvs = tint_uvs(&buffers.positions, &buffers.normals, origin, scale);
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, buffers.positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, buffers.normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vertex_colors)
    .with_inserted_indices(Indices::U32(buffers.indices))
}

/// Texture coordinates in world voxels over the face's plane, so one tint texel lands on each
/// voxel face whether or not faces were merged.
#[expect(
    clippy::cast_precision_loss,
    reason = "the origin is reduced modulo the tint size (64) before the cast"
)]
fn tint_uvs(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    origin: [i64; 3],
    scale: [f32; 3],
) -> Vec<[f32; 2]> {
    let size = i64::from(TINT_SIZE);
    let offset = origin.map(|axis| axis.rem_euclid(size) as f32);
    positions
        .iter()
        .zip(normals)
        .map(|(position, normal)| {
            let axis = (0..3)
                .max_by(|&a, &b| normal[a].abs().total_cmp(&normal[b].abs()))
                .unwrap_or(1);
            let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
            [
                (position[u] * scale[u] + offset[u]) / size as f32,
                (position[v] * scale[v] + offset[v]) / size as f32,
            ]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use voxel_core::{Chunk, LocalPos};
    use voxel_gen::TerrainPalette;
    use voxel_mesh::{PaddedChunk, mesh_quads};

    use super::*;

    #[test]
    fn a_lone_voxel_becomes_a_colored_cube_mesh() {
        let mut chunk = Chunk::empty();
        let grass = TerrainPalette::default().grass;
        chunk.set(LocalPos::new(1, 1, 1).expect("in bounds"), grass);
        let buffers =
            MeshBuffers::from_quads(&mesh_quads(&PaddedChunk::from_chunks(&chunk, |_| None)));

        let mesh = chunk_mesh(buffers, &VoxelColors::default(), [0, 64, -32]);

        assert_eq!(mesh.count_vertices(), 24);
        assert_eq!(mesh.indices().map(Indices::len), Some(36));
        assert!(mesh.attribute(Mesh::ATTRIBUTE_COLOR).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_some());
    }
}

#[cfg(test)]
mod uv_tests {
    use super::*;

    #[test]
    fn neighbouring_chunks_continue_the_tint_pattern() {
        // The same world point seen from two chunks 32 voxels apart along x, on a +y face.
        let normal = [[0.0, 1.0, 0.0]];
        let here = tint_uvs(&[[32.0, 5.0, 3.0]], &normal, [0, 0, 0], [1.0; 3]);
        let next = tint_uvs(&[[0.0, 5.0, 3.0]], &normal, [32, 0, 0], [1.0; 3]);

        assert_eq!(here, next);
    }
}
