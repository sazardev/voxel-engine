# Performance

Rules, not suggestions: every change follows them and a review checks them. When one has to be broken, say so in the change and why.

- **Measure first**: optimize only with a profile or benchmark that shows the problem; report before and after, per tier, in the change.
- **Budgets**: frame time per tier, memory and startup are in `docs/stack/quality-gates.md` and in code (`FrameBudget`); a pass declares its cost.
- **Bounded work per frame**: every queue, upload, mesh job and load has a cap (chunks in flight, uploads per frame, ms per frame on web); an unbounded queue is a bug.
- **No per-voxel entities or allocations in hot paths**: chunks are arrays; reuse buffers; no `Vec` growth in a per-frame loop without reserve.
- **Work off the main thread**: meshing and generation run on the task pool, with a single-thread path on web.
- **Cache with an owner**: every cache has a key, a size limit and an invalidation rule; never cache without knowing how it goes stale.
- **No blocking IO on the main thread**: load and save asynchronously.
- **Data layout**: prefer contiguous data and iteration over the access pattern; no pointer-chasing structures for voxel data.
- **No premature optimization**: clear code first; micro-optimizations need a benchmark that justifies the lost readability.
- Review check: no unbounded queue, no allocation per voxel, no blocking call on the main thread; numbers attached to any optimization.
