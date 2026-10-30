//! Deterministic pseudo-random values on an integer lattice: the randomness behind all noise.

/// `SplitMix64` finalizer: a fast bijective mix with good avalanche, defined only by integer ops.
const fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// A 16-bit value for lattice point (`x`, `z`) of the noise layer named by `seed` and `salt`.
pub(crate) fn lattice_value(seed: u64, salt: u64, x: i64, z: i64) -> u16 {
    let hash = mix(mix(mix(seed ^ salt) ^ x.cast_unsigned()) ^ z.cast_unsigned());
    let [high, low, ..] = hash.to_be_bytes();
    u16::from_be_bytes([high, low])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_point_always_gets_the_same_value() {
        assert_eq!(lattice_value(9, 1, -5, 12), lattice_value(9, 1, -5, 12));
    }

    #[test]
    fn neighbouring_points_and_layers_get_different_values() {
        let here = lattice_value(9, 1, 0, 0);

        assert_ne!(here, lattice_value(9, 1, 1, 0));
        assert_ne!(here, lattice_value(9, 1, 0, 1));
        assert_ne!(here, lattice_value(9, 2, 0, 0));
        assert_ne!(here, lattice_value(10, 1, 0, 0));
    }
}
