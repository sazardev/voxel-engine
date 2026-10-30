//! Golden fixture for SPEC G-1: a fixed seed produces byte-identical world data on every platform.
//!
//! The digest covers voxel content (each id, little-endian, in storage order), not how a chunk
//! is stored or serialized, so storage and save formats can evolve without touching it.
//!
//! If this fails, generated worlds changed. That is a breaking change for every existing save:
//! bump `GENERATOR_VERSION`, record why in an ADR, and only then update the fixture.

use voxel_core::{Chunk, ChunkPos, LocalPos};
use voxel_gen::{GENERATOR_VERSION, TerrainGenerator, TerrainPalette, WorldSeed};

const SEED: WorldSeed = WorldSeed(0x5EED_CAFE_F00D_0001);
/// Surface chunks, underground, sky and far negative coordinates.
const CHUNKS: [ChunkPos; 6] = [
    ChunkPos::new(0, 1, 0),
    ChunkPos::new(0, 6, 0),
    ChunkPos::new(0, 8, 0),
    ChunkPos::new(-1, 7, 3),
    ChunkPos::new(7, 14, -5),
    ChunkPos::new(-123_456, 7, 987_654),
];
const EXPECTED_VERSION: u32 = 2;
const EXPECTED_DIGEST: u64 = 0x92d1_0fe8_f8f1_948d;

/// FNV-1a: tiny, stable and specified, so the fixture does not depend on a hasher's version.
fn fnv1a(digest: u64, bytes: &[u8]) -> u64 {
    bytes.iter().fold(digest, |hash, &byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

fn content_bytes(chunk: &Chunk) -> Vec<u8> {
    LocalPos::all()
        .flat_map(|pos| chunk.get(pos).0.to_le_bytes())
        .collect()
}

#[test]
fn a_fixed_seed_generates_byte_identical_chunks() {
    let terrain = TerrainGenerator::new(SEED, TerrainPalette::default());

    let digest = CHUNKS.iter().fold(0xcbf2_9ce4_8422_2325, |digest, &pos| {
        fnv1a(digest, &content_bytes(&terrain.generate_chunk(pos)))
    });

    assert_eq!(GENERATOR_VERSION, EXPECTED_VERSION);
    assert_eq!(
        digest, EXPECTED_DIGEST,
        "generated worlds changed: digest {digest:#018x}"
    );
}
