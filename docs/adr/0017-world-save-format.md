# 0017. World save: one versioned file, atomic writes, a backup, no compression yet

- Status: accepted
- Date: 2026-10-31
- Requirements: SPEC FR-19, FR-24, FR-25, FR-29, NFR-5, G-4; ADR 0006 (compression left to this ADR); docs/principles/data-contracts.md

## Context

M1 needs the world state to survive a restart and a crash. Generated terrain is not stored (ADR 0010): a world is its seed and generator version plus what changed. Today that is the player, the time of day and the weather; edited chunks come with editing (FR-27). ADR 0006 left the choice of compressor to the save feature. Hexagonal rules put storage behind a port with the adapter in `platform`, and the main thread must not block on IO.

## Decision

- **Port and adapter.** `voxel-core::SaveStore` reads, writes and copies whole files by name; `write` is atomic by contract. `platform::FileSaveStore` writes a temporary file, syncs it and renames it over the old one, in the per-user data folder (`save_directory`). Mobile and the web get their own adapters later (app storage, IndexedDB) and run without saves until then. `MemorySaveStore` is the fake for tests.
- **Format, version 1.** One file, `world.save`: magic `VXWS`, format version (u16), FNV-1a 64 checksum of the body, then a `postcard` body (`WorldSave`: seed, generator version, player feet and look, time of day, weather, edited chunks). A stored version 1 file is a test fixture that every later build must load.
- **Safety.** Before each save, the current file is copied to `world.save.bak` when it is a good save, so a damaged file never replaces a good backup. Loading a damaged file falls back to the backup. A file from a newer format or another terrain generator is refused with a clear message and the session does not save, so the file is never changed. An older format will be copied to a versioned backup before it is migrated.
- **When.** The world loads before startup. It autosaves every 60 s (SPEC Q15 is still open) on the IO task pool, one save in flight at a time, and saves once more on quit. A new world gets a seed mixed from the clock in the composition root (FR-29), and the seed is logged.
- **No compression in version 1.** The file is under 100 bytes without edits. An edited chunk is stored whole, about 33 KB of `postcard` varints; when editing lands and saves grow, a new format version compresses chunk data with a pure-Rust compressor (for example `lz4_flex`), keeping the web build free of C toolchains (ADR 0006).

## Consequences

Crash safety comes from the atomic write plus the backup, tested with fakes and on disk. Saves cost no main-thread IO except the final save on quit. One file per world keeps loading simple but rewrites everything on each save; when edited chunks make that slow, chunk data moves to region files in a later version. Save slots and the world list (FR-24) build on the same store.

## Alternatives considered

- One file per chunk: matches `ChunkStore`, but there are no edits to store yet and many small files cost more on mobile and the web.
- `zstd` now: needs `clang` for the web (ADR 0006) and saves nothing measurable today.
- `serde_json`: readable, but larger and slower, and the data contracts ask for `postcard`.
