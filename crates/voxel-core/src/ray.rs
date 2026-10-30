//! The first solid voxel along a ray, by walking the grid one voxel boundary at a time.

use crate::aabb::first_voxel;

/// Where a ray met a solid voxel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayHit {
    pub voxel: [i64; 3],
    /// Distance from the origin to the face that was hit, along the ray.
    pub distance: f32,
    /// Outward normal of the face that was hit; zero when the ray started inside the voxel.
    pub normal: [i8; 3],
}

/// Casts a ray from `origin` along the unit vector `direction` and returns the first solid voxel
/// within `max_distance`.
#[must_use]
pub fn cast_ray(
    origin: [f32; 3],
    direction: [f32; 3],
    max_distance: f32,
    is_solid: impl Fn([i64; 3]) -> bool,
) -> Option<RayHit> {
    let mut voxel = origin.map(first_voxel);
    if is_solid(voxel) {
        return Some(RayHit {
            voxel,
            distance: 0.0,
            normal: [0; 3],
        });
    }
    let step = direction.map(|d| {
        if d > 0.0 {
            1
        } else if d < 0.0 {
            -1
        } else {
            0
        }
    });
    let t_delta = direction.map(|d| {
        if d == 0.0 {
            f32::INFINITY
        } else {
            1.0 / d.abs()
        }
    });
    let mut t_next = [0, 1, 2].map(|axis| {
        let boundary = if step[axis] > 0 {
            origin[axis].floor() + 1.0
        } else {
            origin[axis].floor()
        };
        if step[axis] == 0 {
            f32::INFINITY
        } else {
            (boundary - origin[axis]) / direction[axis]
        }
    });
    loop {
        let axis = (0..3).min_by(|&a, &b| t_next[a].total_cmp(&t_next[b]))?;
        let distance = t_next[axis];
        if distance > max_distance {
            return None;
        }
        voxel[axis] += step[axis];
        t_next[axis] += t_delta[axis];
        if is_solid(voxel) {
            let mut normal = [0; 3];
            normal[axis] = if step[axis] > 0 { -1 } else { 1 };
            return Some(RayHit {
                voxel,
                distance,
                normal,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ground(voxel: [i64; 3]) -> bool {
        voxel[1] < 0
    }

    #[test]
    fn hits_the_ground_below() {
        let hit = cast_ray([0.5, 5.5, 0.5], [0.0, -1.0, 0.0], 20.0, ground).expect("hits");

        assert_eq!(hit.voxel, [0, -1, 0]);
        assert!((hit.distance - 5.5).abs() < 1e-5);
        assert_eq!(hit.normal, [0, 1, 0]);
    }

    #[test]
    fn misses_what_is_out_of_reach() {
        assert_eq!(
            cast_ray([0.5, 5.5, 0.5], [0.0, -1.0, 0.0], 3.0, ground),
            None
        );
    }

    #[test]
    fn follows_a_diagonal_to_a_wall() {
        let wall = |voxel: [i64; 3]| voxel[0] >= 3;
        let diagonal = [
            std::f32::consts::FRAC_1_SQRT_2,
            0.0,
            std::f32::consts::FRAC_1_SQRT_2,
        ];

        let hit = cast_ray([0.5, 0.5, 0.5], diagonal, 10.0, wall).expect("hits");

        assert_eq!(hit.voxel[0], 3);
        assert_eq!(hit.normal, [-1, 0, 0]);
        assert!((hit.distance - 2.5 * std::f32::consts::SQRT_2).abs() < 1e-4);
    }

    #[test]
    fn starting_inside_a_solid_voxel_hits_at_once() {
        let hit = cast_ray([0.5, -0.5, 0.5], [1.0, 0.0, 0.0], 10.0, ground).expect("hits");

        assert_eq!(hit.distance, 0.0);
    }
}
