//! The color of each voxel kind, from the DESIGN.md palette, and how AO darkens it.

use bevy::color::{LinearRgba, Srgba};
use bevy::prelude::Resource;
use voxel_core::VoxelId;
use voxel_gen::TerrainPalette;

/// DESIGN.md `ao` token: an enclosed corner loses this fraction of its light.
const AO_STRENGTH: f32 = 0.35;
/// Highest AO level: a corner with nothing around it.
const OPEN_CORNER: u8 = 3;

const GRASS: Srgba = Srgba::rgb(0.545, 0.765, 0.290); // grass-ramp #8BC34A
const DIRT: Srgba = Srgba::rgb(0.353, 0.227, 0.149); // wood-ramp #5A3A26 until the palette has dirt
const STONE: Srgba = Srgba::rgb(0.788, 0.769, 0.737); // stone-ramp #C9C4BC
/// Shown for a voxel kind without a color, so the gap is impossible to miss.
const MISSING: Srgba = Srgba::rgb(1.0, 0.0, 1.0);

/// Linear color per `VoxelId`. Replaced by the material registry (FR-28) when it exists.
#[derive(Debug, Clone, Resource)]
pub struct VoxelColors {
    colors: Vec<LinearRgba>,
}

impl VoxelColors {
    #[must_use]
    pub fn for_terrain(palette: &TerrainPalette) -> Self {
        let entries = [
            (palette.grass, GRASS),
            (palette.dirt, DIRT),
            (palette.stone, STONE),
        ];
        let len = entries
            .iter()
            .map(|(id, _)| usize::from(id.0) + 1)
            .max()
            .unwrap_or(0);
        let mut colors = vec![MISSING.into(); len];
        for (id, color) in entries {
            colors[usize::from(id.0)] = color.into();
        }
        Self { colors }
    }

    /// The color of `voxel` at a vertex with AO level `ao` (0 darkest to 3 open).
    #[must_use]
    pub fn shaded(&self, voxel: VoxelId, ao: u8) -> LinearRgba {
        let base = self
            .colors
            .get(usize::from(voxel.0))
            .copied()
            .unwrap_or_else(|| MISSING.into());
        let enclosure = f32::from(OPEN_CORNER - ao.min(OPEN_CORNER)) / f32::from(OPEN_CORNER);
        let light = 1.0 - AO_STRENGTH * enclosure;
        LinearRgba::new(
            base.red * light,
            base.green * light,
            base.blue * light,
            base.alpha,
        )
    }
}

impl Default for VoxelColors {
    fn default() -> Self {
        Self::for_terrain(&TerrainPalette::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_open_corner_shows_the_palette_color() {
        let colors = VoxelColors::default();

        assert_eq!(
            colors.shaded(TerrainPalette::default().grass, 3),
            GRASS.into()
        );
    }

    #[test]
    fn a_closed_corner_loses_the_ao_strength() {
        let colors = VoxelColors::default();
        let stone = TerrainPalette::default().stone;

        let open = colors.shaded(stone, 3);
        let closed = colors.shaded(stone, 0);

        assert!((closed.red - open.red * (1.0 - AO_STRENGTH)).abs() < 1e-6);
    }

    #[test]
    fn an_unknown_voxel_is_loud_magenta() {
        assert_eq!(
            VoxelColors::default().shaded(VoxelId(999), 3),
            MISSING.into()
        );
    }
}
