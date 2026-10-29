use serde::{Deserialize, Serialize};

/// Identifies a material in the palette. `VoxelId::AIR` is empty space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct VoxelId(pub u16);

impl VoxelId {
    pub const AIR: Self = Self(0);

    #[must_use]
    pub const fn is_air(self) -> bool {
        self.0 == Self::AIR.0
    }
}
