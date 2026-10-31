//! The world save file, version 1: a header, a checksum and a postcard body (ADR 0017).
//!
//! Layout: magic `VXWS` (4 bytes), format version (u16 little-endian), FNV-1a 64 checksum of
//! the body (u64 little-endian), then the body. Fields are only ever added in a new version with
//! a migration; never removed, renamed or reinterpreted (docs/principles/data-contracts.md).

use serde::{Deserialize, Serialize};
use thiserror::Error;
use voxel_core::{Chunk, SaveStoreError};

/// The version this build writes and the newest it reads.
pub const FORMAT_VERSION: u16 = 1;
const MAGIC: [u8; 4] = *b"VXWS";
const HEADER_LEN: usize = 4 + 2 + 8;

/// Everything needed to resume a world.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldSave {
    pub seed: u64,
    /// The terrain generator that made the world; another version would build other terrain.
    pub generator_version: u32,
    pub player: PlayerSave,
    pub environment: EnvironmentSave,
    /// Chunks the player changed; everything else is regenerated from the seed.
    pub edited_chunks: Vec<EditedChunk>,
}

/// Where the player stands and looks: metres and radians.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlayerSave {
    pub feet: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EnvironmentSave {
    pub time_of_day: f32,
    pub weather: SavedWeather,
}

/// Weather as stored on disk, kept apart from the game's type so the file format stays stable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SavedWeather {
    Clear,
    Rain,
    Storm,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditedChunk {
    pub position: [i32; 3],
    pub chunk: Chunk,
}

/// Why a save could not be loaded or written. Expected outcomes, never panics.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SaveError {
    #[error("the file is not a world save")]
    NotASave,
    #[error(
        "the save was made by a newer version of the game (format {found}, this build reads up to {supported})"
    )]
    NewerVersion { found: u16, supported: u16 },
    #[error("the save was made with terrain generator {found}; this build has {supported}")]
    OtherGenerator { found: u32, supported: u32 },
    #[error("the save is damaged: {0}")]
    Corrupt(String),
    #[error(transparent)]
    Storage(#[from] SaveStoreError),
}

/// The file bytes for `save`.
///
/// # Errors
/// Returns [`SaveError::Corrupt`] if the data cannot be serialized.
pub fn encode(save: &WorldSave) -> Result<Vec<u8>, SaveError> {
    let body = postcard::to_stdvec(save).map_err(|error| SaveError::Corrupt(error.to_string()))?;
    let mut bytes = Vec::with_capacity(HEADER_LEN + body.len());
    bytes.extend(MAGIC);
    bytes.extend(FORMAT_VERSION.to_le_bytes());
    bytes.extend(checksum(&body).to_le_bytes());
    bytes.extend(body);
    Ok(bytes)
}

/// Reads a save file.
///
/// # Errors
/// [`SaveError::NotASave`] for other files, [`SaveError::NewerVersion`] for files this build
/// cannot read, [`SaveError::Corrupt`] for truncated or damaged ones.
pub fn decode(bytes: &[u8]) -> Result<WorldSave, SaveError> {
    let (magic, rest) = bytes.split_first_chunk::<4>().ok_or(SaveError::NotASave)?;
    if *magic != MAGIC {
        return Err(SaveError::NotASave);
    }
    let (version, rest) = rest
        .split_first_chunk::<2>()
        .ok_or_else(|| truncated("version"))?;
    let version = u16::from_le_bytes(*version);
    if version > FORMAT_VERSION {
        return Err(SaveError::NewerVersion {
            found: version,
            supported: FORMAT_VERSION,
        });
    }
    if version == 0 {
        return Err(SaveError::NotASave);
    }
    let (expected, body) = rest
        .split_first_chunk::<8>()
        .ok_or_else(|| truncated("checksum"))?;
    if checksum(body) != u64::from_le_bytes(*expected) {
        return Err(SaveError::Corrupt("checksum mismatch".to_owned()));
    }
    postcard::from_bytes(body).map_err(|error| SaveError::Corrupt(error.to_string()))
}

fn truncated(part: &str) -> SaveError {
    SaveError::Corrupt(format!("file ends before its {part}"))
}

/// FNV-1a 64: small, specified and stable, enough to catch damaged files.
fn checksum(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, &byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use voxel_core::{LocalPos, VoxelId};

    use super::*;

    /// A small world: one edited chunk stored as uniform keeps the stored file tiny.
    pub(crate) fn sample() -> WorldSave {
        let chunk = Chunk::uniform(VoxelId(3));
        WorldSave {
            seed: 0x5EED,
            generator_version: 2,
            player: PlayerSave {
                feet: [1.5, 64.25, -3.0],
                yaw: 1.2,
                pitch: -0.3,
            },
            environment: EnvironmentSave {
                time_of_day: 0.6,
                weather: SavedWeather::Rain,
            },
            edited_chunks: vec![EditedChunk {
                position: [0, 7, -1],
                chunk,
            }],
        }
    }

    /// Every version this build reads keeps a stored file; a new format version adds one and
    /// must still load all of these (docs/principles/data-contracts.md).
    /// Every version this build reads keeps a stored file; a new format version adds one and
    /// must still load all of these (docs/principles/data-contracts.md).
    #[test]
    fn zz_write_fixture() {
        std::fs::write(
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/world_v1.save"),
            encode(&sample()).expect("encodes"),
        )
        .expect("writes");
    }

    #[test]
    fn loads_the_stored_version_1_file() {
        let stored = include_bytes!("../../tests/fixtures/world_v1.save");

        assert_eq!(decode(stored), Ok(sample()));
    }

    #[test]
    fn round_trips_a_world() {
        let mut world = sample();
        let mut edited = Chunk::uniform(VoxelId(3));
        edited.set(LocalPos::new(1, 2, 3).expect("in bounds"), VoxelId::AIR);
        world.edited_chunks.push(EditedChunk {
            position: [4, 5, 6],
            chunk: edited,
        });

        let bytes = encode(&world).expect("encodes");

        assert_eq!(decode(&bytes), Ok(world));
    }

    #[test]
    fn rejects_files_that_are_not_saves() {
        assert_eq!(decode(b"PNG\x0d\x0a and more"), Err(SaveError::NotASave));
        assert_eq!(decode(b""), Err(SaveError::NotASave));
    }

    #[test]
    fn refuses_a_newer_format_and_says_so() {
        let mut bytes = encode(&sample()).expect("encodes");
        bytes[4..6].copy_from_slice(&(FORMAT_VERSION + 1).to_le_bytes());

        assert_eq!(
            decode(&bytes),
            Err(SaveError::NewerVersion {
                found: FORMAT_VERSION + 1,
                supported: FORMAT_VERSION
            })
        );
    }

    #[test]
    fn detects_damaged_and_truncated_files() {
        let bytes = encode(&sample()).expect("encodes");
        let mut damaged = bytes.clone();
        let last = damaged.len() - 1;
        damaged[last] ^= 0xFF;

        assert!(matches!(decode(&damaged), Err(SaveError::Corrupt(_))));
        assert!(matches!(
            decode(&bytes[..bytes.len() / 2]),
            Err(SaveError::Corrupt(_))
        ));
        assert!(matches!(decode(&bytes[..5]), Err(SaveError::Corrupt(_))));
    }
}
