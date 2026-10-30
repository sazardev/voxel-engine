//! Surface height of every world column: a few octaves of value noise summed in fixed point.

use crate::seed::WorldSeed;
use crate::value_noise::{ONE, ValueNoise};

/// Octaves as (lattice spacing in bits, amplitude in voxels), from broad hills to small bumps.
/// Each one's steepest slope is `1.5 * amplitude / spacing` (about 0.28 here), so their sum
/// stays under `MAX_SURFACE_STEP`.
const OCTAVES: [(u32, i64); 5] = [(8, 48), (7, 24), (6, 12), (5, 6), (4, 3)];
const TOTAL_AMPLITUDE: i64 = 48 + 24 + 12 + 6 + 3;
const _: () = assert!(TOTAL_AMPLITUDE == octave_amplitude_sum());

/// The surface is never below this world height.
pub const MIN_SURFACE_HEIGHT: i64 = 16;
/// The surface is never above this world height.
pub const MAX_SURFACE_HEIGHT: i64 = MIN_SURFACE_HEIGHT + TOTAL_AMPLITUDE;
/// Neighbouring columns differ in height by at most this many voxels: hills, never cliffs.
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
