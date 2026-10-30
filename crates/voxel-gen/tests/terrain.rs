//! Terrain behaviour through the public API: determinism (FR-1, FR-29) and seamless chunks (FR-30).

#![expect(
    clippy::expect_used,
    reason = "test helpers may panic with a clear message"
)]
#![expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    reason = "CHUNK_EDGE is 32: it fits in u8 and i64"
)]

use proptest::prelude::*;
use voxel_core::{CHUNK_EDGE, Chunk, ChunkPos, LocalPos};
use voxel_gen::{
    MAX_SURFACE_HEIGHT, MAX_SURFACE_STEP, MIN_SURFACE_HEIGHT, TerrainGenerator, TerrainPalette,
    WorldSeed,
};

const EDGE: i64 = CHUNK_EDGE as i64;
/// Far enough from the origin to catch sign and overflow mistakes, small enough to stay in `i64`.
const CHUNK_RANGE: std::ops::Range<i32> = -1_000_000..1_000_000;

fn generator(seed: u64) -> TerrainGenerator {
    TerrainGenerator::new(WorldSeed(seed), TerrainPalette::default())
}

fn local(x: u8, y: u8, z: u8) -> LocalPos {
    LocalPos::new(x, y, z).expect("in bounds")
}

/// Highest solid voxel of a column, read from voxel data only: generates the chunk stack that
/// spans every possible surface height.
fn column_top(terrain: &TerrainGenerator, chunk_x: i32, chunk_z: i32, x: u8, z: u8) -> Option<i64> {
    let lowest = i32::try_from(MIN_SURFACE_HEIGHT.div_euclid(EDGE)).expect("fits");
    let highest = i32::try_from(MAX_SURFACE_HEIGHT.div_euclid(EDGE)).expect("fits");
    (lowest..=highest).rev().find_map(|chunk_y| {
        let chunk = terrain.generate_chunk(ChunkPos::new(chunk_x, chunk_y, chunk_z));
        (0..CHUNK_EDGE as u8)
            .rev()
            .find(|&y| !chunk.get(local(x, y, z)).is_air())
            .map(|y| i64::from(chunk_y) * EDGE + i64::from(y))
    })
}

#[test]
fn ground_is_solid_below_the_surface_and_air_above() {
    let terrain = generator(42);
    let palette = TerrainPalette::default();
    let height = terrain.surface_height(10, 20);
    let chunk_y = i32::try_from(height.div_euclid(EDGE)).expect("fits");
    let chunk = terrain.generate_chunk(ChunkPos::new(0, chunk_y, 0));
    let surface_y = u8::try_from(height.rem_euclid(EDGE)).expect("fits");

    assert_eq!(chunk.get(local(10, surface_y, 20)), palette.grass);
    if surface_y > 0 {
        assert_eq!(chunk.get(local(10, surface_y - 1, 20)), palette.dirt);
    }
    if surface_y < 31 {
        assert!(chunk.get(local(10, surface_y + 1, 20)).is_air());
    }
}

#[test]
fn deep_ground_is_stone_and_the_sky_is_empty() {
    let terrain = generator(7);
    let palette = TerrainPalette::default();
    let deep = i32::try_from(MIN_SURFACE_HEIGHT.div_euclid(EDGE)).expect("fits") - 1;
    let sky = i32::try_from(MAX_SURFACE_HEIGHT.div_euclid(EDGE)).expect("fits") + 1;

    let underground = terrain.generate_chunk(ChunkPos::new(3, deep, -4));
    let above = terrain.generate_chunk(ChunkPos::new(3, sky, -4));

    assert!(LocalPos::all().all(|pos| underground.get(pos) == palette.stone));
    assert_eq!(above, Chunk::empty());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn the_same_seed_generates_the_same_chunk(
        seed: u64, x in CHUNK_RANGE, y in -4i32..8, z in CHUNK_RANGE,
    ) {
        let pos = ChunkPos::new(x, y, z);

        prop_assert_eq!(generator(seed).generate_chunk(pos), generator(seed).generate_chunk(pos));
    }

    #[test]
    fn different_seeds_generate_different_terrain(seed: u64, x in CHUNK_RANGE, z in CHUNK_RANGE) {
        let (a, b) = (generator(seed), generator(seed.wrapping_add(1)));
        let (origin_x, origin_z) = (i64::from(x) * EDGE, i64::from(z) * EDGE);

        let differs = (0..EDGE).any(|dx| (0..EDGE).any(|dz| {
            a.surface_height(origin_x + dx, origin_z + dz) != b.surface_height(origin_x + dx, origin_z + dz)
        }));

        prop_assert!(differs);
    }

    #[test]
    fn surface_height_stays_within_bounds(seed: u64, x in any::<i32>(), z in any::<i32>()) {
        let height = generator(seed).surface_height(i64::from(x), i64::from(z));

        prop_assert!((MIN_SURFACE_HEIGHT..=MAX_SURFACE_HEIGHT).contains(&height));
    }

    #[test]
    fn adjacent_columns_never_form_a_cliff(seed: u64, x in any::<i32>(), z in any::<i32>()) {
        let terrain = generator(seed);
        let (x, z) = (i64::from(x), i64::from(z));
        let here = terrain.surface_height(x, z);

        prop_assert!((here - terrain.surface_height(x + 1, z)).abs() <= MAX_SURFACE_STEP);
        prop_assert!((here - terrain.surface_height(x, z + 1)).abs() <= MAX_SURFACE_STEP);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn a_coarse_cell_takes_its_highest_column(seed: u64, x in -100_000i64..100_000, z in -100_000i64..100_000, cell in 1i64..5) {
        let terrain = generator(seed);

        let heights = terrain.cell_heights((x, z), cell, 3);

        for (index, &height) in heights.iter().enumerate() {
            let (cx, cz) = (i64::try_from(index % 3).expect("small"), i64::try_from(index / 3).expect("small"));
            let highest = (0..cell)
                .flat_map(|dz| (0..cell).map(move |dx| (dx, dz)))
                .map(|(dx, dz)| terrain.surface_height(x + cx * cell + dx, z + cz * cell + dz))
                .max();
            prop_assert_eq!(Some(height), highest);
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn neighbouring_chunks_join_without_gaps_or_cliffs_at_the_seam(
        seed: u64, x in CHUNK_RANGE, z in CHUNK_RANGE, row in 0u8..32,
    ) {
        let terrain = generator(seed);
        let last = (CHUNK_EDGE - 1) as u8;

        // The east neighbour is generated first: generation order must not matter.
        let east_edge = column_top(&terrain, x + 1, z, 0, row);
        let west_edge = column_top(&terrain, x, z, last, row);
        let south_edge = column_top(&terrain, x, z + 1, row, 0);
        let north_edge = column_top(&terrain, x, z, row, last);

        let (east, west) = (east_edge.expect("east has ground"), west_edge.expect("west has ground"));
        let (south, north) = (south_edge.expect("south has ground"), north_edge.expect("north has ground"));
        let west_world_x = i64::from(x) * EDGE + EDGE - 1;
        prop_assert_eq!(west, terrain.surface_height(west_world_x, i64::from(z) * EDGE + i64::from(row)));
        prop_assert!((east - west).abs() <= MAX_SURFACE_STEP);
        prop_assert!((south - north).abs() <= MAX_SURFACE_STEP);
    }
}
