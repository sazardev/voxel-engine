//! Seeded, deterministic world generation. Pure Rust: no Bevy types (ADR 0002).
//!
//! Every step is integer arithmetic so a seed gives byte-identical worlds on every platform
//! (SPEC G-1, ADR 0010). Floats are never used for anything that becomes world data.

mod heightmap;
mod lattice_hash;
mod palette;
mod seed;
mod terrain;
mod value_noise;

pub use heightmap::{MAX_SURFACE_HEIGHT, MAX_SURFACE_STEP, MIN_SURFACE_HEIGHT};
pub use palette::TerrainPalette;
pub use seed::WorldSeed;
pub use terrain::{GENERATOR_VERSION, TerrainGenerator};
