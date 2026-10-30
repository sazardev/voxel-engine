# 0012. Uniform chunk storage and a content-based golden digest

- Status: accepted
- Date: 2026-10-30
- Requirements: SPEC G-1, FR-61 (loading and memory), NFR-1; docs/stack/quality-gates.md (memory)

## Context

A chunk stored one `u16` per voxel: 64 KiB each. With a view radius of 6 the world keeps about 1000 chunks of voxel data, about 65 MB, and most of them are a single kind of voxel: sky chunks are all air, deep chunks all stone. Phones and the web set the tightest memory budgets.

The golden test for G-1 hashed the `postcard` bytes of generated chunks, so any change to how a chunk is stored or serialized would have changed the digest even with an identical world.

## Decision

- `Chunk` stores a single `VoxelId` when every voxel is the same (`Chunk::uniform`, `as_uniform`), and one id per voxel otherwise. `from_fn` collapses to uniform when it can; the first differing `set` expands. A chunk is not collapsed again after edits; equality compares content, not storage.
- Deserialization validates the voxel count and returns an error instead of a chunk that would panic on read.
- The golden digest hashes voxel content (each id little-endian, in storage order). It was recomputed from the same generator that the previous digest validated. Storage and the save format may change without touching it; only a change to the generated world may.
- Palette compression for mixed chunks stays future work behind the same API.

## Consequences

Uniform chunks cost no heap memory, and meshing can skip them (all air, or solid and buried). Readers match on storage in `get`, a small branch per read. The serialized form of a chunk changed; there are no saves yet, and the save feature defines its own versioned format (FR-25).

## Alternatives considered

- Palette compression now: more code and a bigger change; the uniform case gives most of the win on generated terrain.
- Keep flat storage and serialize uniform chunks specially: saves memory on disk but not in RAM, where the budget bites.
