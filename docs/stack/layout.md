# Project layout

Where code lives. Folders are by feature; dependencies point inward (ADR 0002).

```
Cargo.toml                         workspace; profiles: dev (opt 1, deps 3), release (thin LTO), web (opt "s", lto)
crates/
  voxel-core/                      chunks, palette storage, coordinates, edits (no Bevy)
  voxel-gen/                       seeded generation (pure)
  voxel-mesh/                      greedy meshing, AO bake, LOD meshes
  platform/                        storage, input mapping, capability probe, per-OS glue
  game/
    src/
      main.rs                      App + plugins
      world/                       streaming, edit queue, save
      player/                      first_person.rs, third_person.rs, touch.rs
      render/
        tiers.rs                   Tier::{Low, Mid, High, Ultra}, chosen from the capability probe and the settings
        passes/                    sun_shadows, ssao, voxel_rt (high), fog, bloom, post
      ui/                          HUD, menus (scale for touch targets)
targets/
  android/  ios/  web/             manifests, icons, index.html, service worker
assets/                            shaders (WGSL), .vox models, palettes, LUTs
```

```
.github/workflows/ci.yml       jobs: lint (format + lint + typecheck), test, build; dependency cache; concurrency per branch
```

```
lefthook.yml                   pre-commit: format + lint ({staged_files}); commit-msg: conventional commit check
```

```
renovate.json                  extends config:recommended; schedule; packageRules: groups, automerge patch
```
