//! Measures meshing. Run with `cargo bench -p voxel-mesh`; report numbers in the change.

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use voxel_core::{Chunk, VoxelId};
use voxel_mesh::{MeshBuffers, PaddedChunk, mesh_quads};

/// Rolling ground with two materials, like a generated surface chunk.
fn hilly_chunk() -> Chunk {
    Chunk::from_fn(|pos| {
        let (x, z) = (u16::from(pos.x()), u16::from(pos.z()));
        let height = 12 + (x * 7 + z * 3) % 9 + (x / 8 + z / 8) % 3;
        match u16::from(pos.y()) {
            y if y < height => VoxelId(3),
            y if y == height => VoxelId(1),
            _ => VoxelId::AIR,
        }
    })
}

fn mesh_hilly_chunk(c: &mut Criterion) {
    let chunk = hilly_chunk();
    let padded = PaddedChunk::from_chunks(&chunk, |_| None);
    c.bench_function("pad a chunk", |b| {
        b.iter(|| PaddedChunk::from_chunks(black_box(&chunk), |_| None));
    });
    c.bench_function("mesh a hilly chunk", |b| {
        b.iter(|| MeshBuffers::from_quads(&mesh_quads(black_box(&padded))));
    });
}

criterion_group!(benches, mesh_hilly_chunk);
criterion_main!(benches);
