//! A per-voxel brightness variation, so every 25 cm cube reads on its own even where faces are
//! merged into one quad (DESIGN.md `voxel-tint`). Sampled with UVs in world voxel units.

use bevy::asset::RenderAssetUsages;
use bevy::image::{Image, ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// Texels per side: one per voxel, so the pattern repeats every 64 voxels (16 m).
pub const TINT_SIZE: u32 = 64;
/// Darkest a voxel gets, as a fraction of its palette color; the brightest is 1.0.
const DARKEST: f32 = 0.84;

/// The texel brightness at (`x`, `y`), from a hash of the position: in `DARKEST..=1.0`.
fn brightness(x: u32, y: u32) -> u8 {
    let mut hash = u64::from(x).wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ u64::from(y).wrapping_mul(0xc2b2_ae3d_27d4_eb4f);
    hash = (hash ^ (hash >> 31)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    let [unit, ..] = hash.to_be_bytes();
    let fraction = f32::from(unit) / 255.0;
    to_byte(DARKEST + (1.0 - DARKEST) * fraction)
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the value is clamped to 0..=255 before the cast"
)]
fn to_byte(unit: f32) -> u8 {
    (unit * 255.0).round().clamp(0.0, 255.0) as u8
}

/// Gray RGBA texels of every mip level, largest first; each level averages 2×2 of the one above.
#[must_use]
pub fn tint_texels() -> (Vec<u8>, u32) {
    let mut level: Vec<u8> = (0..TINT_SIZE)
        .flat_map(|y| (0..TINT_SIZE).map(move |x| brightness(x, y)))
        .collect();
    let mut size = TINT_SIZE;
    let mut all = Vec::new();
    let mut levels = 0;
    loop {
        all.extend(level.iter().flat_map(|&gray| [gray, gray, gray, 255]));
        levels += 1;
        if size == 1 {
            return (all, levels);
        }
        level = halve(&level, size);
        size /= 2;
    }
}

/// Averages each 2×2 block of a square level of side `size`.
fn halve(level: &[u8], size: u32) -> Vec<u8> {
    let width = usize::try_from(size).unwrap_or(0);
    let half = width / 2;
    (0..half)
        .flat_map(|y| (0..half).map(move |x| (x, y)))
        .map(|(x, y)| {
            let at = |dx, dy| u16::from(level[(2 * y + dy) * width + 2 * x + dx]);
            let sum = at(0, 0) + at(1, 0) + at(0, 1) + at(1, 1);
            u8::try_from((sum + 2) / 4).unwrap_or(u8::MAX)
        })
        .collect()
}

/// The tint as a repeating, mipmapped texture: crisp up close, no shimmer far away.
#[must_use]
pub fn tint_image() -> Image {
    let (texels, levels) = tint_texels();
    let size = Extent3d {
        width: TINT_SIZE,
        height: TINT_SIZE,
        depth_or_array_layers: 1,
    };
    let mut image = Image::new_uninit(
        size,
        TextureDimension::D2,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.mip_level_count = levels;
    image.data = Some(texels);
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..ImageSamplerDescriptor::default()
    });
    image
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_texel_stays_within_the_palette_ramp() {
        let lowest = to_byte(DARKEST);

        for y in 0..TINT_SIZE {
            for x in 0..TINT_SIZE {
                assert!((lowest..=255).contains(&brightness(x, y)));
            }
        }
    }

    #[test]
    fn neighbouring_voxels_differ() {
        let row: Vec<u8> = (0..TINT_SIZE).map(|x| brightness(x, 7)).collect();

        assert!(row.windows(2).filter(|pair| pair[0] != pair[1]).count() > row.len() / 2);
    }

    #[test]
    fn holds_every_mip_level_down_to_one_texel() {
        let (texels, levels) = tint_texels();

        // 64² + 32² + ... + 1² texels of 4 bytes.
        let texel_count: u32 = (0..levels).map(|level| (TINT_SIZE >> level).pow(2)).sum();
        assert_eq!(levels, 7);
        assert_eq!(
            texels.len(),
            usize::try_from(texel_count * 4).expect("small")
        );
    }
}
