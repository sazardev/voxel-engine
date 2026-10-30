//! Which chunks should be loaded around a focus, nearest first (docs/stack/world-streaming.md).

use std::collections::HashSet;
use std::ops::RangeInclusive;

use voxel_core::ChunkPos;

/// Chunks wanted around a focus column: meshes within the view radius, and voxel data one
/// chunk further so every meshed chunk has all 26 neighbours. Both lists are nearest first.
#[derive(Debug, Clone)]
pub struct StreamPlan {
    focus: (i32, i32),
    meshes: Vec<ChunkPos>,
    voxels: Vec<ChunkPos>,
    mesh_set: HashSet<ChunkPos>,
    voxel_set: HashSet<ChunkPos>,
}

impl StreamPlan {
    /// Plans the chunks around column `focus` (x, z): columns within `radius` chunks, layers in
    /// `layers` (the chunk heights that can contain the surface).
    #[must_use]
    pub fn around(focus: (i32, i32), radius: u32, layers: RangeInclusive<i32>) -> Self {
        let reach = i32::try_from(radius).unwrap_or(i32::MAX / 2);
        let columns = (-reach..=reach)
            .flat_map(|dz| (-reach..=reach).map(move |dx| (dx, dz)))
            .filter(|&(dx, dz)| {
                dx.unsigned_abs().pow(2) + dz.unsigned_abs().pow(2) <= radius.pow(2)
            })
            .map(|(dx, dz)| (focus.0 + dx, focus.1 + dz))
            .collect();
        Self::for_columns(focus, columns, layers)
    }

    /// Plans exactly the detail `columns` given (absolute chunk columns), with voxel data for
    /// them and every column next to them.
    #[must_use]
    pub fn for_columns(
        focus: (i32, i32),
        columns: Vec<(i32, i32)>,
        layers: RangeInclusive<i32>,
    ) -> Self {
        let mut voxel_columns: Vec<(i32, i32)> = columns
            .iter()
            .flat_map(|&(x, z)| {
                (-1..=1).flat_map(move |dz| (-1..=1).map(move |dx| (x + dx, z + dz)))
            })
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        voxel_columns.sort_unstable();
        let below = layers.start() - 1;
        let above = layers.end() + 1;
        let meshes = nearest_first(focus, columns, &layers);
        let voxels = nearest_first(focus, voxel_columns, &(below..=above));
        Self {
            focus,
            mesh_set: meshes.iter().copied().collect(),
            voxel_set: voxels.iter().copied().collect(),
            meshes,
            voxels,
        }
    }

    #[must_use]
    pub const fn focus(&self) -> (i32, i32) {
        self.focus
    }

    /// Chunks to mesh, nearest first.
    #[must_use]
    pub fn meshes(&self) -> &[ChunkPos] {
        &self.meshes
    }

    /// Chunks whose voxel data must be loaded, nearest first.
    #[must_use]
    pub fn voxels(&self) -> &[ChunkPos] {
        &self.voxels
    }

    #[must_use]
    pub fn wants_mesh(&self, pos: ChunkPos) -> bool {
        self.mesh_set.contains(&pos)
    }

    #[must_use]
    pub fn wants_voxels(&self, pos: ChunkPos) -> bool {
        self.voxel_set.contains(&pos)
    }
}

/// Chunks of the given column offsets and layers, sorted by horizontal distance to the focus.
fn nearest_first(
    focus: (i32, i32),
    columns: Vec<(i32, i32)>,
    layers: &RangeInclusive<i32>,
) -> Vec<ChunkPos> {
    let mut columns = columns;
    columns.sort_by_key(|&(x, z)| {
        let (dx, dz) = (i64::from(x - focus.0), i64::from(z - focus.1));
        (dx * dx + dz * dz, z, x)
    });
    columns
        .into_iter()
        .flat_map(|(x, z)| layers.clone().map(move |y| ChunkPos::new(x, y, z)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn neighbours(pos: ChunkPos) -> impl Iterator<Item = ChunkPos> {
        (-1..=1).flat_map(move |dz| {
            (-1..=1).flat_map(move |dy| {
                (-1..=1).map(move |dx| ChunkPos::new(pos.x + dx, pos.y + dy, pos.z + dz))
            })
        })
    }

    #[test]
    fn meshes_the_columns_within_the_radius_on_every_surface_layer() {
        let plan = StreamPlan::around((10, -4), 2, 0..=3);

        assert_eq!(plan.meshes().len(), 13 * 4);
        assert!(plan.wants_mesh(ChunkPos::new(12, 3, -4)));
        assert!(!plan.wants_mesh(ChunkPos::new(12, 4, -4)));
        assert!(!plan.wants_mesh(ChunkPos::new(12, 0, -2)));
    }

    #[test]
    fn loads_the_voxels_of_every_neighbour_of_a_meshed_chunk() {
        let plan = StreamPlan::around((0, 0), 3, 0..=3);

        for &pos in plan.meshes() {
            for neighbour in neighbours(pos) {
                assert!(
                    plan.wants_voxels(neighbour),
                    "{neighbour:?} next to {pos:?}"
                );
            }
        }
    }

    #[test]
    fn lists_nearer_chunks_first() {
        let plan = StreamPlan::around((5, 5), 4, 0..=1);
        let distance = |pos: &ChunkPos| (pos.x - 5).pow(2) + (pos.z - 5).pow(2);

        let distances: Vec<i32> = plan.meshes().iter().map(distance).collect();

        assert!(distances.is_sorted());
        assert_eq!(plan.meshes()[0], ChunkPos::new(5, 0, 5));
    }
}
