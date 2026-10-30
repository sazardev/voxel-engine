//! The work of meshing one coarse patch, runnable off the main thread: heights straight from
//! the generator, a height-field mesh, no voxel data kept (ADR 0016).

use bevy::prelude::*;
use voxel_core::{CHUNK_EDGE, VOXEL_SIZE_METRES};
use voxel_gen::TerrainGenerator;
use voxel_mesh::{HeightPatch, mesh_heightfield};

use super::chunk_mesh::tinted_mesh;
use super::lod_layout::LodPatch;
use super::voxel_colors::VoxelColors;

/// Columns per side of every patch, whatever its level: a level-1 patch (16 m) has 1 m columns
/// and a level-2 patch (32 m) 2 m columns, about as fine on screen at their distance as 25 cm
/// voxels are at the edge of the detailed area.
const CELLS: usize = CHUNK_EDGE / 2;
/// Skirts reach this many cells below the lower side of a patch edge.
const SKIRT_CELLS: i64 = 4;

/// The patch's mesh and where it sits; `None` when it has no faces.
#[must_use]
pub fn lod_mesh(
    terrain: &TerrainGenerator,
    colors: &VoxelColors,
    patch: LodPatch,
) -> Option<(Mesh, Transform)> {
    let edge = i64::try_from(CHUNK_EDGE).ok()?;
    let cell = i64::from(patch.size()) * edge / i64::try_from(CELLS).ok()?;
    let (first_x, first_z) = patch.first_column();
    let origin = (i64::from(first_x) * edge, i64::from(first_z) * edge);
    // Cubic cells: each surface rounds up to a whole number of cells, so far terrain reads as
    // bigger cubes, merges into fewer faces, and never dips below the detailed terrain.
    let heights: Vec<i64> = terrain
        .cell_heights((origin.0 - cell, origin.1 - cell), cell, CELLS + 2)
        .into_iter()
        .map(|top| -(-(top + 1)).div_euclid(cell) * cell - 1)
        .collect();
    let skirt = SKIRT_CELLS * cell;
    let base = heights.iter().copied().min()? - skirt;
    let palette = terrain.palette();
    let buffers = mesh_heightfield(&HeightPatch {
        cells: CELLS,
        heights: &heights,
        top: palette.grass,
        side: palette.dirt,
        skirt,
        base,
    });
    if buffers.is_empty() {
        return None;
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "cells are a few voxels and positions a few thousand metres, exact enough in f32"
    )]
    let (cell_f32, origin_metres) = (
        cell as f32,
        Vec3::new(origin.0 as f32, base as f32, origin.1 as f32) * VOXEL_SIZE_METRES,
    );
    let scale = [cell_f32, 1.0, cell_f32];
    let mesh = tinted_mesh(buffers, colors, [origin.0, base, origin.1], scale);
    let transform = Transform::from_translation(origin_metres)
        .with_scale(Vec3::from_array(scale) * VOXEL_SIZE_METRES);
    Some((mesh, transform))
}

#[cfg(test)]
mod tests {
    use voxel_gen::{TerrainPalette, WorldSeed};

    use super::*;

    #[test]
    fn a_patch_sits_at_its_columns_and_spans_its_size() {
        let terrain = TerrainGenerator::new(WorldSeed(5), TerrainPalette::default());
        let patch = LodPatch {
            level: 2,
            x: -3,
            z: 1,
        };

        let (mesh, transform) =
            lod_mesh(&terrain, &VoxelColors::default(), patch).expect("terrain has faces");

        // Level 2: 4 chunk columns of 8 m, so 32 m per side starting at (-96 m, 32 m).
        assert!(
            (transform.translation.x - -96.0).abs() < 1e-4
                && (transform.translation.z - 32.0).abs() < 1e-4
        );
        assert!((transform.scale.x * 16.0 - 32.0).abs() < 1e-4);
        assert!(mesh.count_vertices() > 0);
    }

    #[test]
    fn far_cells_are_cubes_that_never_dip_below_the_detail() {
        let terrain = TerrainGenerator::new(WorldSeed(5), TerrainPalette::default());
        let patch = LodPatch {
            level: 1,
            x: 4,
            z: -2,
        };

        let (mesh, transform) =
            lod_mesh(&terrain, &VoxelColors::default(), patch).expect("terrain has faces");

        // Level 1: 16 m patches of 1 m cells. Tops sit on whole metres.
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("positions");
        };
        for position in positions {
            let metres = transform.translation.y + position[1] * transform.scale.y;
            assert!((metres - metres.round()).abs() < 1e-3, "{metres}");
        }
    }
}
