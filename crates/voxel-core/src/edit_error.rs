use thiserror::Error;

/// Why an edit to the voxel grid was rejected. Expected outcomes are values, not panics.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EditError {
    #[error("position ({x}, {y}, {z}) is outside a chunk of edge {edge}")]
    OutOfBounds { x: u8, y: u8, z: u8, edge: usize },
}
