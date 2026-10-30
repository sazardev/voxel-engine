//! Moving a box through the voxel grid without entering solid voxels (ADR 0014).
//!
//! World units: one voxel is one unit, voxel `[x, y, z]` spans `x..x + 1` on each axis.

/// Gap kept between a box and the voxel it stopped against, so it never touches it exactly.
const SKIN: f32 = 1e-3;

/// An axis-aligned box: `min` inclusive, `max` exclusive.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// Where a box ended after a sweep, and on which axes something stopped it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sweep {
    pub aabb: Aabb,
    pub blocked: [bool; 3],
}

impl Aabb {
    /// A box standing on `feet`: `half_width` around it on x and z, `height` up.
    #[must_use]
    pub fn standing(feet: [f32; 3], half_width: f32, height: f32) -> Self {
        Self {
            min: [feet[0] - half_width, feet[1], feet[2] - half_width],
            max: [feet[0] + half_width, feet[1] + height, feet[2] + half_width],
        }
    }

    /// The center of the bottom face.
    #[must_use]
    pub fn feet(&self) -> [f32; 3] {
        [
            f32::midpoint(self.min[0], self.max[0]),
            self.min[1],
            f32::midpoint(self.min[2], self.max[2]),
        ]
    }

    /// True when any voxel the box overlaps is solid.
    #[must_use]
    pub fn overlaps_solid(&self, is_solid: impl Fn([i64; 3]) -> bool) -> bool {
        let [xs, ys, zs] =
            [0, 1, 2].map(|axis| first_voxel(self.min[axis])..=last_voxel(self.max[axis]));
        zs.into_iter()
            .any(|z| ys.clone().any(|y| xs.clone().any(|x| is_solid([x, y, z]))))
    }

    /// Moves the box by `motion`, one axis at a time (y first, so it lands before it slides),
    /// stopping each axis at the first solid voxel. A box that starts clear of solid voxels
    /// ends clear of them.
    #[must_use]
    pub fn sweep(self, motion: [f32; 3], is_solid: impl Fn([i64; 3]) -> bool) -> Sweep {
        let mut sweep = Sweep {
            aabb: self,
            blocked: [false; 3],
        };
        for axis in [1, 0, 2] {
            if motion[axis] != 0.0 {
                let (allowed, blocked) = sweep.aabb.allowed_motion(axis, motion[axis], &is_solid);
                sweep.aabb.min[axis] += allowed;
                sweep.aabb.max[axis] += allowed;
                sweep.blocked[axis] = blocked;
            }
        }
        sweep
    }

    /// How far the box can move along `axis`, up to `distance`, before touching a solid voxel,
    /// and whether a solid voxel stopped it.
    fn allowed_motion(
        &self,
        axis: usize,
        distance: f32,
        is_solid: impl Fn([i64; 3]) -> bool,
    ) -> (f32, bool) {
        let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
        let us = first_voxel(self.min[u])..=last_voxel(self.max[u]);
        let vs = first_voxel(self.min[v])..=last_voxel(self.max[v]);
        let layer_is_solid = |layer: i64| {
            us.clone().any(|a| {
                vs.clone().any(|b| {
                    let mut voxel = [0; 3];
                    (voxel[axis], voxel[u], voxel[v]) = (layer, a, b);
                    is_solid(voxel)
                })
            })
        };
        if distance > 0.0 {
            let entered = last_voxel(self.max[axis]) + 1..=last_voxel(self.max[axis] + distance);
            match entered.into_iter().find(|&layer| layer_is_solid(layer)) {
                Some(layer) => (
                    (to_coordinate(layer) - self.max[axis] - SKIN).clamp(0.0, distance),
                    true,
                ),
                None => (distance, false),
            }
        } else {
            let entered = first_voxel(self.min[axis] + distance)..=first_voxel(self.min[axis]) - 1;
            match entered
                .into_iter()
                .rev()
                .find(|&layer| layer_is_solid(layer))
            {
                Some(layer) => (
                    (to_coordinate(layer + 1) - self.min[axis] + SKIN).clamp(distance, 0.0),
                    true,
                ),
                None => (distance, false),
            }
        }
    }
}

/// The voxel containing `coordinate`.
#[expect(
    clippy::cast_possible_truncation,
    reason = "world coordinates in play are far inside i64; floor() makes them whole"
)]
pub(crate) fn first_voxel(coordinate: f32) -> i64 {
    coordinate.floor() as i64
}

/// The last voxel a half-open span ending at `coordinate` reaches.
#[expect(
    clippy::cast_possible_truncation,
    reason = "world coordinates in play are far inside i64; ceil() makes them whole"
)]
fn last_voxel(coordinate: f32) -> i64 {
    coordinate.ceil() as i64 - 1
}

#[expect(
    clippy::cast_precision_loss,
    reason = "voxel layers near the player are small enough to be exact in f32"
)]
fn to_coordinate(layer: i64) -> f32 {
    layer as f32
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn floor_at_zero(voxel: [i64; 3]) -> bool {
        voxel[1] < 0
    }

    /// Solid voxels scattered by a hash of the position: about one in `density`.
    fn scattered(seed: u64, density: u64) -> impl Fn([i64; 3]) -> bool {
        move |[x, y, z]| {
            let mut hash = seed ^ x.cast_unsigned().wrapping_mul(0x9e37_79b9_7f4a_7c15);
            hash = (hash ^ y.cast_unsigned()).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            hash = (hash ^ z.cast_unsigned()).wrapping_mul(0x94d0_49bb_1331_11eb);
            (hash >> 33).is_multiple_of(density)
        }
    }

    #[test]
    fn falls_onto_the_floor_and_stops_on_it() {
        let player = Aabb::standing([0.5, 3.0, 0.5], 0.3, 1.8);

        let sweep = player.sweep([0.0, -10.0, 0.0], floor_at_zero);

        assert!(sweep.blocked[1]);
        assert!(sweep.aabb.min[1] >= 0.0 && sweep.aabb.min[1] < 0.01);
    }

    #[test]
    fn slides_along_a_wall_it_runs_into() {
        let wall = |voxel: [i64; 3]| voxel[1] < 0 || voxel[0] >= 2;
        let player = Aabb::standing([0.5, 0.0, 0.5], 0.3, 1.8);

        let sweep = player.sweep([5.0, 0.0, 3.0], wall);

        assert!(sweep.blocked[0] && !sweep.blocked[2]);
        assert!(sweep.aabb.max[0] <= 2.0);
        assert!((sweep.aabb.min[2] - 3.2).abs() < 1e-4);
    }

    #[test]
    fn moves_freely_where_nothing_is_solid() {
        let player = Aabb::standing([0.5, 0.0, 0.5], 0.3, 1.8);

        let sweep = player.sweep([1.5, 2.0, -0.75], |_| false);

        assert_eq!(sweep.blocked, [false; 3]);
        assert_eq!(sweep.aabb.feet(), [2.0, 2.0, -0.25]);
    }

    proptest! {
        #[test]
        fn never_ends_inside_a_solid_voxel(
            seed: u64, density in 2u64..8,
            start in prop::array::uniform3(-20.0f32..20.0),
            motion in prop::array::uniform3(-3.0f32..3.0),
        ) {
            let player = Aabb::standing(start, 0.3, 1.8);
            let scatter = scattered(seed, density);
            // Clear the voxels the box starts in, so every case starts free.
            let start_voxels = [0, 1, 2].map(|axis| first_voxel(player.min[axis])..=last_voxel(player.max[axis]));
            let solid = |voxel: [i64; 3]| {
                scatter(voxel) && !(0..3).all(|axis| start_voxels[axis].contains(&voxel[axis]))
            };
            prop_assert!(!player.overlaps_solid(solid));

            let sweep = player.sweep(motion, solid);

            prop_assert!(!sweep.aabb.overlaps_solid(solid), "{:?} -> {:?}", player, sweep);
        }
    }
}
