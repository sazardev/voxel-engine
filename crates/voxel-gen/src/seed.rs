/// The number a world grows from. The same seed always generates the same world (FR-1, FR-29).
///
/// Picking a random seed is the composition root's job; generation never invents one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldSeed(pub u64);
