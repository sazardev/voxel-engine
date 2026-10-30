//! Turns the pure mesher's buffers into a Bevy `Mesh` with palette colors and AO baked in.

use bevy::asset::RenderAssetUsages;
use bevy::color::ColorToComponents;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use voxel_mesh::MeshBuffers;

use super::voxel_colors::VoxelColors;

/// A render-only mesh: the CPU copy is dropped after upload, since voxels stay the source of truth.
#[must_use]
pub fn chunk_mesh(buffers: MeshBuffers, colors: &VoxelColors) -> Mesh {
    let vertex_colors: Vec<[f32; 4]> = buffers
        .voxels
        .iter()
        .zip(&buffers.ao)
        .map(|(&voxel, &ao)| colors.shaded(voxel, ao).to_f32_array())
        .collect();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, buffers.positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, buffers.normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vertex_colors)
    .with_inserted_indices(Indices::U32(buffers.indices))
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

        let mesh = chunk_mesh(buffers, &VoxelColors::default());

        assert_eq!(mesh.count_vertices(), 24);
        assert_eq!(mesh.indices().map(Indices::len), Some(36));
        assert!(mesh.attribute(Mesh::ATTRIBUTE_COLOR).is_some());
    }
}
