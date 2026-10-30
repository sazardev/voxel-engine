//! Smooth 2D value noise in 16.16 fixed point: one octave of the heightmap.

use crate::lattice_hash::lattice_value;

const FRACTION_BITS: u32 = 16;
/// 1.0 in 16.16 fixed point.
pub(crate) const ONE: i64 = 1 << FRACTION_BITS;

/// One layer of noise: random values on a square lattice, smoothly blended in between.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ValueNoise {
    pub seed: u64,
    pub salt: u64,
    /// The lattice spacing is `1 << cell_bits` voxels; at most 16.
    pub cell_bits: u32,
}

impl ValueNoise {
    /// Noise at world column (`x`, `z`), in `0..ONE`. Continuous with a continuous slope, so
    /// neighbouring columns differ by at most `1.5 * ONE / cell size`.
    pub fn sample(self, x: i64, z: i64) -> i64 {
        let (cell_x, tx) = self.split(x);
        let (cell_z, tz) = self.split(z);
        let corner = |dx, dz| {
            i64::from(lattice_value(
                self.seed,
                self.salt,
                cell_x + dx,
                cell_z + dz,
            ))
        };
        let near = lerp(corner(0, 0), corner(1, 0), tx);
        let far = lerp(corner(0, 1), corner(1, 1), tx);
        lerp(near, far, tz)
    }

    /// Lattice cell containing `coordinate` and the smoothed position inside it, in `0..=ONE`.
    fn split(self, coordinate: i64) -> (i64, i64) {
        let cell = coordinate >> self.cell_bits;
        let offset = coordinate - (cell << self.cell_bits);
        (cell, fade(offset << (FRACTION_BITS - self.cell_bits)))
    }
}

/// Smoothstep `3t² - 2t³`: zero slope at cell edges, so octaves join without creases.
fn fade(t: i64) -> i64 {
    (((t * t) >> FRACTION_BITS) * (3 * ONE - 2 * t)) >> FRACTION_BITS
}

fn lerp(from: i64, to: i64, t: i64) -> i64 {
    from + (((to - from) * t) >> FRACTION_BITS)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn fade_keeps_the_ends_of_the_cell() {
        assert_eq!(fade(0), 0);
        assert_eq!(fade(ONE), ONE);
        assert_eq!(fade(ONE / 2), ONE / 2);
    }

    #[test]
    fn matches_the_lattice_value_at_lattice_points() {
        let noise = ValueNoise {
            seed: 3,
            salt: 0,
            cell_bits: 4,
        };

        assert_eq!(noise.sample(32, -16), i64::from(lattice_value(3, 0, 2, -1)));
    }

    proptest! {
        #[test]
        fn stays_in_range_and_changes_slowly(seed: u64, x in any::<i32>(), z in any::<i32>(), cell_bits in 2u32..=8) {
            let noise = ValueNoise { seed, salt: 0, cell_bits };
            let (x, z) = (i64::from(x), i64::from(z));
            let here = noise.sample(x, z);
            let max_step = 3 * ONE / (2 << cell_bits) + 2;

            prop_assert!((0..ONE).contains(&here));
            prop_assert!((here - noise.sample(x + 1, z)).abs() <= max_step);
            prop_assert!((here - noise.sample(x, z + 1)).abs() <= max_step);
        }
    }
}
