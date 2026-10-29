//! Chunk storage, coordinates and edits. Pure Rust: no Bevy types (ADR 0002).

/// Edge length of a chunk, in voxels.
pub const CHUNK_EDGE: usize = 32;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_edge_is_a_power_of_two() {
        assert!(CHUNK_EDGE.is_power_of_two());
    }
}
