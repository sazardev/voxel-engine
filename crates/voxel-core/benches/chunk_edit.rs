//! Measures chunk edits. Run with `cargo bench -p voxel-core`; report numbers in the change.

#![expect(
    clippy::expect_used,
    reason = "benchmark setup may panic with a clear message"
)]

use criterion::{Criterion, criterion_group, criterion_main};
use voxel_core::{CHUNK_EDGE, Chunk, LocalPos, VoxelId};

fn fill_chunk(c: &mut Criterion) {
    let edge = u8::try_from(CHUNK_EDGE).expect("edge fits in u8");
    c.bench_function("fill a chunk voxel by voxel", |b| {
        b.iter(|| {
            let mut chunk = Chunk::empty();
            for z in 0..edge {
                for y in 0..edge {
                    for x in 0..edge {
                        let pos = LocalPos::new(x, y, z).expect("in bounds");
                        chunk.set(pos, VoxelId(1));
                    }
                }
            }
            chunk
        });
    });
}

criterion_group!(benches, fill_chunk);
criterion_main!(benches);
