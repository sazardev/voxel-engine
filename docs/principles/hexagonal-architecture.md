# Hexagonal architecture

Rules, not suggestions: every change follows them and a review checks them. When one has to be broken, say so in the change and why.

Mapping: **core** = `voxel-core`, `voxel-gen`, `voxel-mesh` (pure); **adapters** = `platform`; **driving side and composition root** = `game` (Bevy). See ADR 0002.

- **The core** contains the rules and knows nothing about Bevy, files, the OS, the clock or randomness.
- **Ports are owned by the core**: traits declared where they are used (`ChunkStore`, `Clock`), named in game terms, never after a technology.
- **Adapters live at the edge**: `platform` implements storage, input mapping and the capability probe; Bevy plugins in `game` drive the core.
- **Dependency rule**: `game` → `platform`, core crates. Core crates never import `platform` or Bevy (enforced by `cargo deny` and the `guard` crate).
- **No engine types in the core**: Bevy components, resources and handles do not cross into `voxel-*`; `game` maps to and from core types.
- **One composition root**: `game/src/lib.rs` and `main.rs` wire concrete adapters; nothing else builds infrastructure.
- **Tests through ports**: core logic is tested with in-memory fakes (`MemoryChunkStore`, `FixedClock`); adapters get their own integration tests against the real file system or browser storage.
- Review check: `guard` and `cargo deny` pass; every new adapter implements a port declared in the core.
