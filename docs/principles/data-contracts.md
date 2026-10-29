# Data contracts

Rules, not suggestions: every change follows them and a review checks them. When one has to be broken, say so in the change and why.

There is no network API. The contracts that matter are the traits between crates (ports), the save format and the settings files. They replace the REST-style API rules.

- **Contract first**: a port (trait) or file format is described and reviewed before its implementation; its doc comment states inputs, outputs, errors and side effects.
- **Ports are small and owned by the consumer**: `ChunkStore`, `Clock`; named in game terms, never after a technology.
- **Versioned formats**: every file on disk (chunk, save, settings) starts with a format version; loading an older version migrates, loading a newer one fails with a clear error.
- **Backward compatibility**: add optional fields; never remove, rename or reinterpret a field within a version. A breaking change is a new version plus a migration and an ADR.
- **Stable wire format**: serialize with `postcard` + `serde` (ADR 0006); never serialize Bevy types or `HashMap` iteration order into a file.
- **Deterministic output**: the same seed and inputs give byte-identical data on every platform (SPEC G-1); fixtures in tests pin it.
- **Consistent naming and units**: one case style per format, SI units, ids are typed newtypes (`VoxelId`, `ChunkPos`), never bare integers.
- **Round trips**: every format has a test that writes, reads and compares, and one that loads a stored fixture of each past version.
- Review check: a changed port or format has its doc, version, migration and tests in the same change; no existing save breaks.
