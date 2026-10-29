# Testing

What is tested and how. General rules are in `../principles/testing.md`.

- `guard` crate: architecture tests over the source tree (see ADR 0007).
- Pure crates: unit and property tests (edits then reads agree, meshing leaves no holes, seeds are deterministic).
- Headless `App` tests for streaming, edits, save/load round trips and tier selection from fake capability probes.
- Golden-image test per tier on a fixed seed and camera; Low and Mid also run in CI.
- Manual device matrix (one low-end phone, one tablet, a desktop GPU, a browser) before a release; record frame time per tier.
