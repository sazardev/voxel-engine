//! Greedy meshing with per-vertex ambient occlusion. Pure Rust: no Bevy types (ADR 0002, 0011).
//!
//! Pipeline: [`PaddedChunk`] (a chunk plus one voxel of each neighbour) → [`mesh_quads`]
//! (visible faces merged into [`Quad`]s) → [`MeshBuffers`] (vertex data ready to upload).

mod ambient_occlusion;
mod buffers;
mod chunk_edge;
mod face;
mod greedy;
mod padded;
mod quad;
mod solid_columns;

pub use buffers::MeshBuffers;
pub use face::Face;
pub use greedy::mesh_quads;
pub use padded::PaddedChunk;
pub use quad::Quad;
