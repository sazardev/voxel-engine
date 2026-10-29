# STACK.md: Voxel 3D: Bevy cross-platform

This file is the index. Detail lives in `docs/stack/` (this project) and `docs/principles/` (how we write code). Agents start at `AGENTS.md`.

## Summary

An open-world voxel game in Rust on Bevy and wgpu that ships to Windows, Linux, macOS, Android, iOS and the web from one
codebase. The look is polished stylized voxel lighting (soft shadows, ambient occlusion, bounce light, fog, glow), not
photorealism. The world and the lighting live in the voxel grid itself, so quality scales by tier: the same art direction
runs on a phone with cheap light and on a desktop GPU with ray-marched light. Everything works offline.

## Commands

| Task              | Command                                                 |
| ----------------- | ------------------------------------------------------- |
| Run (desktop)     | `cargo run -p game`                                     |
| Run (web)         | `trunk serve`                                           |
| Android           | `cargo ndk -t arm64-v8a build -p game --release`        |
| Test              | `cargo test --workspace`                                |
| Lint              | `cargo clippy --workspace --all-targets -- -D warnings` |
| Format            | `cargo fmt --all`                                       |
| Format check      | `cargo fmt --all --check`                               |
| Dependency check  | `cargo deny check`                                      |
| Spell check       | `typos`                                                 |
| Docs/TOML format  | `dprint check` (fix: `dprint fmt`)                      |
| Benchmark         | `cargo bench -p voxel-core`                             |
| Coverage (signal) | `cargo llvm-cov --workspace --summary-only`             |
| Profile           | `cargo run -p game --release --features profile`        |
| Release (desktop) | `cargo build --release -p game`                         |
| Install hooks     | `lefthook install`                                      |

## Project docs

| Topic                           | Read                                     |
| ------------------------------- | ---------------------------------------- |
| What is built with what, why    | `docs/stack/technology.md`               |
| Where code lives                | `docs/stack/layout.md`                   |
| Devices, tiers, offline, web    | `docs/stack/platform-and-tiers.md`       |
| Post chain, color, Environment  | `docs/stack/render-pipeline.md`          |
| Shadows, fog, volumetrics       | `docs/stack/lighting-and-volumetrics.md` |
| Chunk loading, LOD, culling     | `docs/stack/world-streaming.md`          |
| Camera rig, comfort, input      | `docs/stack/camera-and-input.md`         |
| CI, hooks, dependency updates   | `docs/stack/automation.md`               |
| Tests                           | `docs/stack/testing.md`                  |
| Pass/fail criteria, budgets     | `docs/stack/quality-gates.md`            |
| Allowed and banned dependencies | `docs/stack/dependencies.md`             |
| Rules for AI agents             | `docs/stack/rules-for-agents.md`         |
| Engineering principles          | `docs/principles/README.md`              |
| Decisions and their reasons     | `docs/adr/README.md`                     |
