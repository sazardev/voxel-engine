//! Surface height of every world column: a few octaves of value noise summed in fixed point.

use crate::seed::WorldSeed;
use crate::value_noise::{ONE, ValueNoise};

/// Octaves as (lattice spacing in bits, amplitude), in 25 cm voxels (ADR 0015): from hills
/// 256 m across and 48 m high down to bumps 4 m across and 1 m high.
/// Each one's steepest slope is `1.5 * amplitude / spacing` (0.28 for the first five, 0.375 for
/// the last), so their sum, 1.78, stays under `MAX_SURFACE_STEP`.
const OCTAVES: [(u32, i64); 6] = [(10, 192), (9, 96), (8, 48), (7, 24), (6, 12), (4, 4)];
const TOTAL_AMPLITUDE: i64 = octave_amplitude_sum();

/// The surface is never below this world height, in voxels (16 m).
pub const MIN_SURFACE_HEIGHT: i64 = 64;
/// The surface is never above this world height, in voxels.
pub const MAX_SURFACE_HEIGHT: i64 = MIN_SURFACE_HEIGHT + TOTAL_AMPLITUDE;
/// Neighbouring columns differ in height by at most this many voxels (50 cm): hills the player
/// walks up without jumping, never cliffs.
pub const MAX_SURFACE_STEP: i64 = 2;

const fn octave_amplitude_sum() -> i64 {
    let mut sum = 0;
    let mut index = 0;
    while index < OCTAVES.len() {
        sum += OCTAVES[index].1;
        index += 1;
    }
    sum
}

/// World height of the topmost solid voxel of column (`x`, `z`).
pub(crate) fn surface_height(seed: WorldSeed, x: i64, z: i64) -> i64 {
    let mut salt = 0;
    let weighted_sum: i64 = OCTAVES
        .iter()
        .map(|&(cell_bits, amplitude)| {
            salt += 1;
            ValueNoise {
                seed: seed.0,
                salt,
                cell_bits,
            }
            .sample(x, z)
                * amplitude
        })
        .sum();
    MIN_SURFACE_HEIGHT + weighted_sum / ONE
}

/// Surface heights of the `side × side` columns from `origin`, indexed `z * side + x`.
/// Equal to [`surface_height`] per column, computed a grid at a time.
pub(crate) fn column_heights(seed: WorldSeed, origin: (i64, i64), side: i64) -> Vec<i64> {
    let len = usize::try_from(side * side).unwrap_or(0);
    let mut sums = vec![0; len];
    for (salt, &(cell_bits, amplitude)) in (1..).zip(OCTAVES.iter()) {
        ValueNoise {
            seed: seed.0,
            salt,
            cell_bits,
        }
        .accumulate_grid(origin, side, amplitude, &mut sums);
    }
    sums.into_iter()
        .map(|sum| MIN_SURFACE_HEIGHT + sum / ONE)
        .collect()
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn grid_heights_match_single_columns(seed: u64, x in -1_000_000i64..1_000_000, z in -1_000_000i64..1_000_000) {
            let heights = column_heights(WorldSeed(seed), (x, z), 8);

            for (index, &height) in heights.iter().enumerate() {
                let (dx, dz) = (i64::try_from(index % 8).expect("small"), i64::try_from(index / 8).expect("small"));
                prop_assert_eq!(height, surface_height(WorldSeed(seed), x + dx, z + dz));
            }
        }
    }
}
