# Dependencies

What may and may not be used. Version pins and exceptions are in the ADRs (0005, 0006).

- Allowed: Bevy and ecosystem crates pinned to compatible versions, the crates in the table.
- Do NOT use: an entity or mesh per voxel, blocking IO on the main thread, platform-specific game logic, `unsafe` outside documented modules, features that need the network.
