//! Measures terrain generation. Run with `cargo bench -p voxel-gen`; report numbers in the change.

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use voxel_core::ChunkPos;
use voxel_gen::{TerrainGenerator, TerrainPalette, WorldSeed};

fn generate_surface_chunk(c: &mut Criterion) {
    let terrain = TerrainGenerator::new(WorldSeed(42), TerrainPalette::default());
    c.bench_function("generate a surface chunk", |b| {
        b.iter(|| terrain.generate_chunk(black_box(ChunkPos::new(3, 2, -7))));
    });
}

criterion_group!(benches, generate_surface_chunk);
criterion_main!(benches);
