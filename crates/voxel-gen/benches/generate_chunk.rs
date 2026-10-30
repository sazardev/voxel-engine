//! Measures terrain generation. Run with `cargo bench -p voxel-gen`; report numbers in the change.

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use voxel_core::ChunkPos;
use voxel_gen::{TerrainGenerator, TerrainPalette, WorldSeed};

fn generate_chunks(c: &mut Criterion) {
    let terrain = TerrainGenerator::new(WorldSeed(42), TerrainPalette::default());
    // The layer holding the surface in the middle of column (3, -7).
    let surface = terrain
        .surface_height(3 * 32 + 16, -7 * 32 + 16)
        .div_euclid(32);
    let surface = i32::try_from(surface).unwrap_or_default();
    c.bench_function("generate a surface chunk", |b| {
        b.iter(|| terrain.generate_chunk(black_box(ChunkPos::new(3, surface, -7))));
    });
    c.bench_function("generate a column of chunks", |b| {
        b.iter(|| {
            (0..16)
                .map(|y| terrain.generate_chunk(black_box(ChunkPos::new(3, y, -7))))
                .collect::<Vec<_>>()
        });
    });
}

criterion_group!(benches, generate_chunks);
criterion_main!(benches);
