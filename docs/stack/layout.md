# Project layout

Where code lives. Folders are by feature; dependencies point inward (ADR 0002). Items marked _planned_ do not exist yet.

```
Cargo.toml                         workspace, shared dependencies and lints; profiles: dev (opt 1, deps 3), release (thin LTO), web (opt "s", lto)
crates/
  voxel-core/                      chunks, coordinates, edits, grid queries, voxel size, ports (ChunkStore, SaveStore, Clock) and their fakes (no Bevy)
  voxel-gen/                       seeded generation (pure, integer-only: ADR 0010): terrain heightmap, golden fixture
  voxel-mesh/                      greedy meshing with per-vertex AO (ADR 0011), height-field patches for LOD (ADR 0016)
  platform/                        capabilities (probe result), file save store and save directory, input mapping, per-OS glue
  guard/                           architecture and docs tests only (ADR 0007)
  game/
    src/
      lib.rs                       GamePlugin: registers feature plugins
      main.rs                      window + renderer + GamePlugin
      run_mode.rs                  RunMode (Player or Dev) and the log filter per mode
      dev/                         developer tools, compiled only with `--features dev` (overlay, hotkeys)
      camera/                      MainCamera and the one rig for first and third person (FR-16), look, cursor capture
      environment/                 Environment resource: time, weather (single source for light, fog, sky); daylight, sun
      render/
        tiers.rs                   Tier::{Low, Mid, High, Ultra} chosen from Capabilities and the settings
        passes/                    _planned_: sun_shadows, ssao, voxel_rt (high), fog, bloom, post
      world/                       streaming plan, LOD quadtree and patches, voxel data, chunk meshes, colors and tint; _planned_: edit queue
      save/                        world save format, slot with backup, load at startup, autosave and save on quit (ADR 0017)
      input.rs                     PlayerAction and default bindings (keyboard, mouse, gamepad)
      player/                      player body, fixed-step movement against the grid, spawn; _planned_: touch
      ui/                          _planned_: HUD, menus (scale for touch targets)
targets/
  android/  ios/  web/             _planned_: manifests, icons, index.html, service worker
assets/                            shaders (WGSL), .vox models, palettes, LUTs
```

Repository tooling:

```
AGENTS.md  CLAUDE.md               entry point for AI agents (CLAUDE.md imports AGENTS.md)
STACK.md  SPEC.md  DESIGN.md       stack index, product requirements, visual language
docs/stack  docs/principles  docs/adr   topic docs, engineering principles, decisions (each folder has a README index)
.claude/                           settings.json (Stop hook runs the gates), hooks/, commands/ (/check, /new-adr, /new-feature)
.github/workflows/ci.yml           jobs: lint (fmt + clippy), deny, docs (typos + dprint), test, build (native + wasm)
.github/pull_request_template.md   review checklist
lefthook.yml                       pre-commit: fmt, clippy, typos, dprint; pre-push: test, deny; commit-msg: Conventional Commits
renovate.json                      grouped weekly updates; automerge patch
.cargo/config.toml                 aliases: cargo dev, cargo play, cargo profile
rust-toolchain.toml  rustfmt.toml  clippy.toml  deny.toml  dprint.json  _typos.toml
```
