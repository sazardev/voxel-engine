# 0018. Save slots: one set of files per world, listed from the store

- Status: accepted
- Date: 2026-10-30
- Requirements: SPEC FR-24, FR-19, NFR-5; ADR 0017 (this builds on it, it does not change its format)

## Context

FR-24 asks for several worlds, each shown in a list with its name, last played time and size. ADR 0017 stores one world in `world.save`. The `SaveStore` port cannot list files or tell their size and age, and a flat store has no folders, so the layout has to live in file names. A typed world name must never reach a path unchecked, and a save from before slots must keep working.

## Decision

- **Port.** `SaveStore::list` returns every file with its name, size in bytes and last write time in seconds since the Unix epoch (`None` when the store cannot tell). The fake keeps a settable clock so tests control "last played". `FileSaveStore` reads the folder, and skips folders and the `.tmp` files of an interrupted write.
- **One id per world.** A `WorldId` is a slug: lowercase ASCII letters, digits and single dashes, at most 32 characters. It is the only thing that reaches a file name, so no typed name can escape the save folder.
- **Files of a world.** `<id>.save` (the ADR 0017 format, unchanged), `<id>.save.bak` (its backup) and `<id>.name` (the name the player typed, plain UTF-8, at most 48 characters; the id is shown when it is missing or unreadable). The world from before slots has the id `world`, so its `world.save` is already a slot and nothing is migrated.
- **What the list shows.** A world exists once it has a save. Last played is the write time of `<id>.save`; size is the sum of the three files. Worlds are sorted most recent first. A name file without a save is a world never played: its slot is reused, so starting twice without saving does not pile up worlds.
- **Bounds.** At most 99 worlds. A second world with the same name gets a number (`home-2`).
- **Choosing at startup.** Until the menus exist (`ui/` is planned), `--world <name>` opens that world or starts it, `--list-worlds` prints the list and quits, and no option continues the world played last (or makes `World` on the first start). The rule is a pure function in `game`; only `main` reads arguments and picks the adapter.

## Consequences

The list costs one directory read and one small read per world, with no save parsing. Format version 1 and its fixture are untouched. A menu later calls the same `list_worlds`, `create_world` and `choose_world`. Deleting and renaming a world are not part of this change; they add `delete` to the port. Mobile and web adapters must implement `list` with the same contract. The write time is the file system's, so copying the folder by hand changes "last played".

## Alternatives considered

- A folder per world: needs directories in the port, which the web and mobile stores do not have the same way.
- The name and last played time inside `world.save`: a new format version and a migration for something the store already knows, and the list would have to parse every save.
- A separate index file of worlds: a second source of truth that can disagree with the saves after a crash.
