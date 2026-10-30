# 0013. Chunk jobs on the task pool, inline on devices without threads

- Status: accepted
- Date: 2026-10-30
- Requirements: SPEC FR-26, FR-60, FR-61, NFR-1; STACK.md (Meshing); docs/stack/platform-and-tiers.md (web threads)

## Context

Generation and meshing first ran on the main thread with per-frame counts (8 generations, 4 meshes). Streaming a radius of 6 took 131 frames, and the worst frame spent 4.3 ms of the main thread on chunk work. STACK.md asks for meshing on a task pool with a single-thread fallback on the web, where threads need cross-origin isolation.

## Decision

- A `Job` runs chunk work either on Bevy's `AsyncComputeTaskPool` (`ChunkWorkers::Background`) or immediately on the main thread (`ChunkWorkers::Inline`). The world picks `Background` when `available_parallelism()` reports more than one thread; that is a device capability, not a platform check. Tests force either mode.
- Every queue is bounded by a `StreamBudget` per mode: jobs started per frame, jobs in flight (running plus finished but not uploaded), and meshes uploaded per frame. Background: 32 generation starts, 64 in flight; 16 meshing starts, 32 in flight; 8 uploads. Inline: 8, 8; 4, 4; 4.
- A meshing job receives shared `Arc<Chunk>` handles for the chunk and its 26 neighbours plus the voxel colors, and returns a finished Bevy `Mesh`; the main thread only adds the asset and spawns the entity. Uniform air chunks and uniform solid chunks surrounded by solid face neighbours skip meshing.
- Jobs for chunks that leave the plan are dropped, which cancels them; results are taken only for chunks still wanted.

## Consequences

Measured in release, headless, radius 6: Background streams everything in 65 frames and 35 ms; Inline in 132 frames and 134 ms; the worst frame is about 1.4 ms in both. Voxel memory is 14.7 MB with uniform chunks (ADR 0012). Results arrive in a nondeterministic order in Background mode, so streaming tests check the final set and the budgets, not the frame order. Editing chunks later must re-mesh neighbours through the same queues.

## Alternatives considered

- Main thread only: simple and deterministic, but costs frame time on every device with threads.
- Bevy's `ComputeTaskPool` with a scoped parallel loop per frame: blocks the frame until the batch finishes.
- Dedicated worker threads and channels: more code, and still needs the inline path for the web.
