//! The work of meshing one chunk, runnable off the main thread: neighbourhood in, `Mesh` out.

use std::sync::Arc;

use bevy::mesh::Mesh;
use voxel_core::{CHUNK_EDGE, Chunk, ChunkPos};
use voxel_mesh::{MeshBuffers, PaddedChunk, mesh_quads};

use super::chunk_mesh::chunk_mesh;
use super::voxel_colors::VoxelColors;
use super::voxels::ChunkVoxels;

/// Offsets of the six chunks that share a face with the center.
const FACE_NEIGHBOURS: [[i32; 3]; 6] = [
    [1, 0, 0],
    [-1, 0, 0],
    [0, 1, 0],
    [0, -1, 0],
    [0, 0, 1],
    [0, 0, -1],
];

/// A chunk and its 26 neighbours, shared with the job that meshes them. Index `9z + 3y + x`
/// of offsets shifted by one; the center is index 13.
#[derive(Debug, Clone)]
pub struct Neighbourhood {
    chunks: Vec<Arc<Chunk>>,
}

impl Neighbourhood {
    /// Gathers the chunk at `pos` and its neighbours, or `None` until all 27 are loaded.
    #[must_use]
    pub fn gather(voxels: &ChunkVoxels, pos: ChunkPos) -> Option<Self> {
        let chunks = offsets()
            .map(|[dx, dy, dz]| voxels.shared(ChunkPos::new(pos.x + dx, pos.y + dy, pos.z + dz)))
            .collect::<Option<Vec<_>>>()?;
        Some(Self { chunks })
    }

    fn at(&self, [dx, dy, dz]: [i32; 3]) -> Option<&Chunk> {
        let index = usize::try_from(9 * (dz + 1) + 3 * (dy + 1) + (dx + 1)).ok()?;
        self.chunks.get(index).map(Arc::as_ref)
    }

    /// True when meshing cannot produce a face: all air, or solid with solid on every side.
    fn is_trivially_empty(&self) -> Option<bool> {
        let center = self.at([0, 0, 0])?.as_uniform();
        Some(match center {
            Some(voxel) if voxel.is_air() => true,
            Some(_) => FACE_NEIGHBOURS.iter().all(|&offset| {
                self.at(offset)
                    .and_then(Chunk::as_uniform)
                    .is_some_and(|voxel| !voxel.is_air())
            }),
            None => false,
        })
    }
}

fn offsets() -> impl Iterator<Item = [i32; 3]> {
    (-1..=1).flat_map(|dz| (-1..=1).flat_map(move |dy| (-1..=1).map(move |dx| [dx, dy, dz])))
}

/// The render mesh of the neighbourhood's center chunk at `pos`; `None` when it has no visible
/// face.
#[must_use]
pub fn mesh_chunk(
    pos: ChunkPos,
    neighbourhood: &Neighbourhood,
    colors: &VoxelColors,
) -> Option<Mesh> {
    if neighbourhood.is_trivially_empty().unwrap_or(true) {
        return None;
    }
    let padded = PaddedChunk::from_chunks(neighbourhood.at([0, 0, 0])?, |offset| {
        neighbourhood.at(offset)
    });
    let buffers = MeshBuffers::from_quads(&mesh_quads(&padded));
    let edge = i64::try_from(CHUNK_EDGE).unwrap_or(32);
    let origin = [pos.x, pos.y, pos.z].map(|axis| i64::from(axis) * edge);
    (!buffers.is_empty()).then(|| chunk_mesh(buffers, colors, origin))
}

#[cfg(test)]
mod tests {
    use voxel_core::VoxelId;

    use super::*;

    fn neighbourhood(center: Chunk, around: &Chunk) -> Neighbourhood {
        let chunks = offsets()
            .map(|offset| {
                Arc::new(if offset == [0, 0, 0] {
                    center.clone()
                } else {
                    around.clone()
                })
            })
            .collect();
        Neighbourhood { chunks }
    }

    #[test]
    fn an_air_chunk_has_no_mesh() {
        let around = Chunk::uniform(VoxelId(3));

        assert!(
            mesh_chunk(
                ChunkPos::new(0, 0, 0),
                &neighbourhood(Chunk::empty(), &around),
                &VoxelColors::default()
            )
            .is_none()
        );
    }

    #[test]
    fn a_buried_solid_chunk_has_no_mesh() {
        let stone = Chunk::uniform(VoxelId(3));

        assert!(
            mesh_chunk(
                ChunkPos::new(0, 0, 0),
                &neighbourhood(stone.clone(), &stone),
                &VoxelColors::default()
            )
            .is_none()
        );
    }

    #[test]
    fn a_solid_chunk_next_to_air_is_meshed() {
        let stone = Chunk::uniform(VoxelId(3));

        let mesh = mesh_chunk(
            ChunkPos::new(0, 0, 0),
            &neighbourhood(stone, &Chunk::empty()),
            &VoxelColors::default(),
        );

        assert_eq!(mesh.map(|mesh| mesh.count_vertices()), Some(6 * 4));
    }
}
