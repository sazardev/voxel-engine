//! Which part of the world is drawn at which detail: a quadtree of chunk columns around the
//! focus, aligned to a fixed grid so blocks never overlap or leave holes (ADR 0016).

/// A square block of `2^level` × `2^level` chunk columns drawn as one coarse height-field mesh.
/// `x` and `z` count blocks of that size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LodPatch {
    pub level: u8,
    pub x: i32,
    pub z: i32,
}

impl LodPatch {
    /// Chunk columns per side.
    #[must_use]
    pub const fn size(self) -> i32 {
        1 << self.level
    }

    /// The chunk column at the patch's minimum corner.
    #[must_use]
    pub const fn first_column(self) -> (i32, i32) {
        (self.x * self.size(), self.z * self.size())
    }

    /// Whether chunk column (`x`, `z`) lies in this patch.
    #[must_use]
    pub const fn contains(self, (x, z): (i32, i32)) -> bool {
        let (first_x, first_z) = self.first_column();
        x >= first_x && x < first_x + self.size() && z >= first_z && z < first_z + self.size()
    }
}

/// Detail columns (full voxels) near the focus and coarser patches further out, nearest first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LodLayout {
    pub detail: Vec<(i32, i32)>,
    pub patches: Vec<LodPatch>,
}

impl LodLayout {
    /// Columns within `detail_radius` chunks stay detailed; level `n` reaches
    /// `detail_radius * 2^n` chunks, up to `levels`.
    #[must_use]
    pub fn around(focus: (i32, i32), detail_radius: u32, levels: u8) -> Self {
        let mut layout = Self {
            detail: Vec::new(),
            patches: Vec::new(),
        };
        let top = LodPatch {
            level: levels,
            x: 0,
            z: 0,
        }
        .size();
        let reach = i32::try_from(detail_radius << levels).unwrap_or(i32::MAX / 4);
        let blocks =
            |center: i32| (center - reach).div_euclid(top)..=(center + reach).div_euclid(top);
        for z in blocks(focus.1) {
            for x in blocks(focus.0) {
                let block = LodPatch {
                    level: levels,
                    x,
                    z,
                };
                if nearest_squared(focus, block) <= radius_squared(detail_radius, levels) {
                    layout.split(focus, detail_radius, block);
                }
            }
        }
        let by_distance =
            |&(x, z): &(i32, i32)| distance_squared(focus, LodPatch { level: 0, x, z });
        layout
            .detail
            .sort_by_key(|column| (by_distance(column), column.1, column.0));
        layout.patches.sort_by_key(|patch| {
            (
                distance_squared(focus, *patch),
                patch.level,
                patch.z,
                patch.x,
            )
        });
        layout
    }

    fn split(&mut self, focus: (i32, i32), detail_radius: u32, block: LodPatch) {
        if block.level == 0 {
            self.detail.push((block.x, block.z));
            return;
        }
        let finer = block.level - 1;
        if nearest_squared(focus, block) < radius_squared(detail_radius, finer) {
            for (dx, dz) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                let child = LodPatch {
                    level: finer,
                    x: block.x * 2 + dx,
                    z: block.z * 2 + dz,
                };
                self.split(focus, detail_radius, child);
            }
        } else {
            self.patches.push(block);
        }
    }
}

/// Squared distance between the centers of the focus column and `block`, in half chunks.
fn distance_squared(focus: (i32, i32), block: LodPatch) -> i64 {
    let (first_x, first_z) = block.first_column();
    let size = i64::from(block.size());
    let dx = 2 * i64::from(first_x) + size - (2 * i64::from(focus.0) + 1);
    let dz = 2 * i64::from(first_z) + size - (2 * i64::from(focus.1) + 1);
    dx * dx + dz * dz
}

/// Squared distance from the center of the focus column to the nearest point of `block`, in
/// half chunks: zero when the focus is inside it, so the block holding the focus always splits.
fn nearest_squared(focus: (i32, i32), block: LodPatch) -> i64 {
    let (first_x, first_z) = block.first_column();
    let size = i64::from(block.size());
    let gap = |first: i32, center: i32| {
        let (low, high, at) = (
            2 * i64::from(first),
            2 * (i64::from(first) + size),
            2 * i64::from(center) + 1,
        );
        (low - at).max(at - high).max(0)
    };
    let (dx, dz) = (gap(first_x, focus.0), gap(first_z, focus.1));
    dx * dx + dz * dz
}

/// Squared reach of `level`, in half chunks.
fn radius_squared(detail_radius: u32, level: u8) -> i64 {
    let reach = 2 * (i64::from(detail_radius) << level);
    reach * reach
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn coverage(layout: &LodLayout) -> HashMap<(i32, i32), usize> {
        let mut covered = HashMap::new();
        for &column in &layout.detail {
            *covered.entry(column).or_default() += 1;
        }
        for patch in &layout.patches {
            let (first_x, first_z) = patch.first_column();
            for z in first_z..first_z + patch.size() {
                for x in first_x..first_x + patch.size() {
                    *covered.entry((x, z)).or_default() += 1;
                }
            }
        }
        covered
    }

    #[test]
    fn covers_every_column_once_without_holes_up_to_the_reach() {
        let focus = (37, -12);
        let layout = LodLayout::around(focus, 6, 2);
        let covered = coverage(&layout);

        assert!(covered.values().all(|&count| count == 1));
        // Everything well inside the reach (24 chunks, minus one top block) is drawn.
        for z in -18..=18 {
            for x in -18..=18 {
                if x * x + z * z <= 18 * 18 {
                    assert!(
                        covered.contains_key(&(focus.0 + x, focus.1 + z)),
                        "hole at {x}, {z}"
                    );
                }
            }
        }
    }

    #[test]
    fn keeps_the_focus_detailed_and_coarser_patches_further_out() {
        let focus = (0, 0);
        let layout = LodLayout::around(focus, 6, 2);
        let distance = |patch: &LodPatch| distance_squared(focus, *patch);

        assert_eq!(layout.detail[0], focus);
        let nearest_coarse = layout
            .patches
            .iter()
            .filter(|p| p.level == 2)
            .map(distance)
            .min();
        let nearest_fine = layout
            .patches
            .iter()
            .filter(|p| p.level == 1)
            .map(distance)
            .min();
        assert!(nearest_fine < nearest_coarse);
    }

    #[test]
    fn a_small_detail_radius_still_details_the_focus() {
        for focus in [(0, 0), (5, -7), (-1, -1)] {
            let layout = LodLayout::around(focus, 1, 2);

            assert!(layout.detail.contains(&focus), "{focus:?}: {layout:?}");
            assert!(coverage(&layout).values().all(|&count| count == 1));
        }
    }

    #[test]
    fn without_levels_everything_is_detail() {
        let layout = LodLayout::around((3, 3), 4, 0);

        assert_eq!(layout.patches, Vec::new());
        assert!(layout.detail.len() > 40);
    }
}
