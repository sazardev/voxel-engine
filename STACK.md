# STACK.md: Voxel 3D: Bevy cross-platform

## Summary

An open-world voxel game in Rust on Bevy and wgpu that ships to Windows, Linux, macOS, Android, iOS and the web from one
codebase. The look is polished stylized voxel lighting (soft shadows, ambient occlusion, bounce light, fog, glow), not
photorealism. The world and the lighting live in the voxel grid itself, so quality scales by tier: the same art direction
runs on a phone with cheap light and on a desktop GPU with ray-marched light. Everything works offline.

## Stack

| Layer                | Choice                                                                                                                                              | Why                                                                   |
| -------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------- |
| Language             | Rust (stable, 2024 edition)                                                                                                                         | One language for every target                                         |
| Engine               | Bevy (latest stable, pin the minor)                                                                                                                 | ECS, render graph, plugins, Android/iOS/WASM targets                  |
| GPU API              | wgpu: Vulkan, Metal, DX12, WebGPU (GL as the web fallback)                                                                                          | One backend API for all platforms                                     |
| Voxel storage        | Chunks of 32³ palette-compressed voxels, crate `voxel-core`                                                                                         | Small memory, fast edits, no engine types                             |
| Meshing              | `block-mesh` greedy quads on a task pool (single-thread fallback on web)                                                                            | Fewer triangles, off the main thread                                  |
| Lighting (all tiers) | Baked per-vertex AO, directional sun with cascaded soft shadows, hemispheric ambient, height and distance fog                                       | Polished look at a mobile cost                                        |
| Lighting (mid tier)  | Screen-space AO, bloom on emissive voxels, voxel-grid contact shadows                                                                               | Depth without ray tracing                                             |
| Lighting (high tier) | Voxel-grid ray marching for shadows, AO and one bounce of GI, temporal accumulation + denoise                                                       | The Teardown-like glow of light in a voxel world                      |
| Physics              | `avian3d` for bodies, voxel DDA for world queries                                                                                                   | World is a grid                                                       |
| Touch and input      | `leafwing-input-manager` + on-screen controls                                                                                                       | Rebindable keys, gamepad and touch share actions                      |
| Save                 | `postcard` (serde) chunk files, compressed (compressor chosen in the save ADR); IndexedDB on web, app storage on mobile                             | Same format, per-platform storage                                     |
| Build                | `cargo`, `trunk` or `wasm-bindgen` (web), `cargo-apk`/`cargo-ndk` (Android), Xcode project (iOS)                                                    | Standard Rust per-target tooling                                      |
| Testing              | `cargo test` headless `App` and pure crates                                                                                                         | Rules without a window                                                |
| Post-processing      | HDR (16-bit float) scene, TAA, bloom, depth of field, per-object motion blur, filmic tone mapping (ACES or AgX), 3D LUT color grading               | Cinematic image from one ordered chain                                |
| Atmosphere           | Precomputed atmospheric scattering LUTs for the sky, volumetric or raymarched clouds, one `Environment` state driving sun, ambient, fog and grading | Time of day and weather change the whole image coherently             |
| Shadows              | Cascaded shadow maps (3-4 cascades), PCF/PCSS filtering, texel-snapped, depth bias by slope                                                         | Soft, stable shadows over a large view distance                       |
| Volumetrics          | Froxel grid (about 160×90×64) ray-marched in a compute pass, temporally reprojected, one scattering pass per frame                                  | Fog and light shafts at a fixed, predictable cost                     |
| Streaming and LOD    | Ring streaming around the camera, LOD by octree (full chunks near, merged chunks far), GPU frustum and Hi-Z occlusion culling, multi-draw indirect  | Long view distance without hitching                                   |
| First person         | First-person rig: yaw on the body, pitch on the head, FOV 80-100 (slider), separate FOV for the held item                                           | The player's eyes are the camera, with control that feels right       |
| Third person         | Third-person orbit rig: spring arm with collision against the voxel grid, over-the-shoulder offset, aim mode, view toggle with first person         | A readable view of the character without clipping through walls       |
| CI                   | GitHub Actions (`.github/workflows/ci.yml`) on every pull request and push to `main`                                                                | Runs the same quality gates as local, next to the code review         |
| Hooks                | Lefthook (`lefthook.yml`): pre-commit format + lint on staged files, commit-msg Conventional Commits check                                          | Problems caught before they reach CI, in parallel and in milliseconds |
| Updates              | Renovate (`renovate.json`): grouped weekly updates, automerge for patch and lockfile maintenance                                                    | Dependencies stay current in small, reviewable steps                  |

## Project layout

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

## Conventions

- Probe the device at startup (GPU limits, memory, WebGPU availability, touch) and pick a render tier; the player can override it. The probe result decides what is allowed, never a platform `#[cfg]` scattered in gameplay.
- Art direction is one: the same palette, models and light colors on every tier; tiers only add or remove passes and resolution.
- Stylized light, not realism: strong key light, tinted soft shadows, colored bounce, emissive glow; check each scene in the Low tier first.
- Every pass declares its tier, its cost budget in ms and a fallback; High and Ultra never ship a visual that Low breaks.
- Memory budgets per tier (chunks resident, texture sizes, view distance); mobile and web are the tightest and drive the limits.
- Web has no threads by default: meshing and generation have a single-thread path with a per-frame time budget; use workers and SharedArrayBuffer only when the page is cross-origin isolated.
- Touch controls (move stick, look drag, action buttons) map to the same input actions as keyboard and gamepad.
- Battery and heat matter: a 30 fps cap option on mobile, a frame limiter, and the High tier off by default on phones.
- No network layer; saves and settings are local on every platform.
- Order of the chain is fixed and documented: TAA → depth of field → motion blur → bloom → tone mapping → color grading → film effects → UI.
- Color grading is a 3D LUT (32³) per environment, blended on transitions; the HUD is drawn after grading and is never graded.
- Depth of field and motion blur are physically motivated (focus distance, shutter fraction) and have a strength slider and an off switch.
- Every effect is a pass with off / low / high tiers; the settings screen exposes the ones that can cause discomfort.
- One `Environment` state (time, weather, biome, interior) is the single source for sun direction, light color, fog, sky and the grading LUT.
- Atmospheric perspective tints distant chunks toward the horizon color; fog distance follows the view distance setting.
- Interiors and caves override the environment smoothly (blend over about 1 second), never as a pop.
- Snap each cascade to the shadow-map texel grid so shadows do not shimmer when the camera moves.
- Penumbra widens with distance from the occluder (PCSS) on the high tier; the low tier uses fixed PCF.
- Chunks are culled per cascade; shadow passes use the same indirect draw data as the main pass.
- Fog density, scattering color and anisotropy are parameters of the environment (day cycle, biome, interior), not constants in shaders.
- Lights opt in to volumetrics with a flag; only the sun and a few key lights scatter.
- Render the froxel grid at reduced resolution and upsample; jitter per frame and blend with history to hide banding.
- Chunks load in rings by priority (in view first), with a per-frame budget for uploads and a cap on chunks in flight.
- Far LOD uses merged, simplified meshes and fades in with dithering, never a visible pop.
- Culling happens on the GPU from a single list of chunk bounds; the CPU does not iterate chunks per frame.
- Mouse look is raw input, not smoothed by default; sensitivity and invert are settings; pitch is clamped to ±89°.
- The held item (view model) renders in its own pass with its own FOV so it does not stretch with the world FOV.
- Head bob, sway and camera shake have a strength slider and go to zero with "reduce motion".
- The arm shortens when voxels block it (ray cast against the grid) and recovers smoothly; the camera never enters a solid block.
- The character fades out (dither) when the camera gets too close instead of popping.
- Aim mode narrows FOV and moves the camera over the shoulder; the crosshair follows the real aim ray.
- CI runs exactly the commands in the Commands table; no CI-only scripts.
- Third-party actions are pinned to a full commit SHA; `permissions:` default to `contents: read`.
- `concurrency` cancels superseded runs on the same branch; `main` is protected and requires a green CI.
- Hooks run only the fast checks on staged files; tests stay in CI.
- Hooks run the same commands as the Commands table.
- Related packages update together (framework + its plugins, test tooling, linters).
- Patch updates and lockfile maintenance automerge only when CI is green; minor/major are reviewed.
- A major update gets its own pull request with the migration notes linked.

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
| Profile           | `cargo run -p game --release --features profile`        |
| Release (desktop) | `cargo build --release -p game`                         |
| Install hooks     | `lefthook install`                                      |

## Testing

- `guard` crate: architecture tests over the source tree (see ADR 0007).
- Pure crates: unit and property tests (edits then reads agree, meshing leaves no holes, seeds are deterministic).
- Headless `App` tests for streaming, edits, save/load round trips and tier selection from fake capability probes.
- Golden-image test per tier on a fixed seed and camera; Low and Mid also run in CI.
- Manual device matrix (one low-end phone, one tablet, a desktop GPU, a browser) before a release; record frame time per tier.

## Quality gates

- `cargo clippy -D warnings`, `cargo fmt --check` and `cargo test --workspace` green.
- Frame budget by tier: Low 30 fps minimum on a mid-range phone (33 ms), Mid 60 fps on tablet or integrated GPU, High 60 fps on a desktop GPU; no frame above 2x the budget while streaming.
- Web build under the size budget in the SPEC and starts to the menu within its time budget.
- Memory stays flat over a 30-minute session on mobile.
- The whole chain costs at most 3 ms at the default tier; with every effect off the image is still correct.
- Shadow passes together cost at most 2 ms at the default tier; no visible shimmer when walking at full speed.
- Volumetrics cost at most 1.5 ms at the default tier; the high tier states its own cost.
- Walking at full speed never produces a frame above 33 ms; the view distance setting changes memory predictably.

## Dependencies

- Allowed: Bevy and ecosystem crates pinned to compatible versions, the crates in the table.
- Do NOT use: an entity or mesh per voxel, blocking IO on the main thread, platform-specific game logic, `unsafe` outside documented modules, features that need the network.

## Rules for agents

- Put voxel logic in `voxel-*` crates with tests and keep Bevy types out of them.
- Add every visual effect as a pass with a tier, a cost budget and a lower-tier fallback; verify the Low tier still looks right.
- Never branch gameplay on the platform; branch on capabilities and tier.
- Measure before optimizing and report frame time per tier in the change.
- Keep touch, keyboard and gamepad on one set of input actions.
- Use the single-thread path on web; never assume threads exist.
- Keep the game fully offline.
- Add effects by inserting a pass in the chain; never apply grading or blur inside gameplay or material code.
- Respect the "reduce motion" setting: motion blur, camera shake and head bob go to zero.
- Change time of day or weather only through the `Environment` state; never set a light or fog value from gameplay code.
- Tune bias and cascade splits in one config; never add per-object shadow hacks.
- Volumetrics is one pass with tiers off / low / high (grid size, step count); never ray-march fog inside a material shader.
- Bound every queue (chunks in flight, uploads per frame); an unbounded queue is a bug.
- Keep movement in the fixed-step simulation and the camera interpolated from it, so look is smooth at any frame rate.
- Keep one rig for first and third person with a mode switch; do not fork the player controller.
- Never weaken or skip a CI step to make a change pass.
- A new quality gate is added to the Commands table and to `ci.yml` in the same change.
- Architecture rules (file size, no `utils`, pure crates, no network, no platform branching in gameplay) are enforced by the `guard` crate inside `cargo test --workspace`; a new rule becomes a new guard test, not a prose-only rule.
- Crate dependency direction is enforced by `cargo deny` bans; a new crate adds its ban in the same change.
- Never put secrets in workflow files; use repository or environment secrets.
- Never bypass hooks with `--no-verify`.
- Commit messages follow Conventional Commits (`feat(scope): …`, `fix(scope): …`).
- Never pin a dependency to an old version to silence Renovate without a comment saying why.
- Never mix a dependency update with feature changes in one commit.

## Engineering principles

Rules, not suggestions: every change follows them, and a review checks them. When one has to be broken, say so in the change and why.

### API design

- **Contract first**: the API is described (OpenAPI, protobuf, GraphQL schema) and reviewed before implementation; code and docs are generated or checked against it.
- **Resources and verbs**: plural nouns (`/orders/{id}/items`), HTTP methods with their semantics, correct status codes (201 + `Location`, 204, 404, 409, 422).
- **Consistent errors** as RFC 9457 problem details (`type`, `title`, `status`, `detail`, field errors); never leak stack traces.
- **Idempotency**: PUT/DELETE are idempotent; POST that creates or charges accepts an `Idempotency-Key` and returns the same result on retry.
- **Pagination** on every list (cursor-based for large or changing sets), with a max page size; filtering and sorting via documented query params.
- **Backward compatibility**: add optional fields, never remove or rename fields or change their meaning within a version; breaking changes get a new version and a deprecation period.
- **Versioning** is explicit (`/v1` or a header), one strategy per API.
- **Consistent naming and formats**: one case style for fields, ISO 8601 UTC timestamps, string ids, money as integer minor units plus currency.
- Review check: the contract changed in the same change as the code, and no existing client breaks.

### Clean Code

- **Names** reveal intent: a reader knows what a variable holds or a function does without reading its body; no abbreviations, no `data`, `info`, `tmp`, `manager`, `helper`.
- **Small functions**: one thing, one level of abstraction; ~30 lines is a smell, ~60 is a split. At most 3-4 parameters; more becomes an options/params type.
- **Guard clauses**: handle invalid cases first and return early; nesting deeper than 3 levels is refactored.
- **No surprises**: a function named like a query does not mutate; side effects are visible in the name (`save`, `send`, `delete`).
- **No magic values**: numbers and strings with meaning become named constants or configuration.
- **Comments explain why**, never what: the code says what. No commented-out code, no journal comments, no stale TODOs.
- **DRY with judgment**: remove duplicated knowledge, not similar-looking code; wait for the third occurrence before abstracting.
- **Boolean flags** that switch a function's behaviour become two functions.
- Review check: every new name reads as a sentence in its call site; no function in the diff exceeds the limits above without a reason in the change.

### Error handling

- **Never swallow**: every error is handled, returned or logged-and-escalated exactly once; empty catch blocks and ignored results are bugs.
- **Wrap with context**: add what was being done (`load order 42: …`) while keeping the original cause inspectable (wrapping, `cause`).
- **Domain errors are types/values** (`ErrOrderNotFound`, `InsufficientFunds`) declared in the core; infrastructure errors stay in adapters and are translated.
- **Map at the boundary, once**: one place turns errors into HTTP status, exit codes or UI messages; no status codes inside business logic.
- **Fail fast at startup**: missing or invalid configuration stops the process with a clear message instead of failing on first use.
- **No exceptions/panics for control flow**: expected outcomes (not found, invalid input) are values; panics only for programmer errors.
- **User-safe messages**: users see what happened and what to do; stack traces, SQL and internal ids go only to logs.
- **Log where handled**, not at every layer: one log entry per failure, with its context and correlation id.
- **Retries** only for transient errors, with backoff, jitter and a limit; never retry non-idempotent operations blindly.
- Review check: no new code path drops an error; every new error type is mapped at the boundary.

### Hexagonal architecture

- **The core** (domain + application) contains business rules and knows nothing about HTTP, databases, queues, frameworks, ORMs or the UI.
- **Ports are owned by the core**: interfaces declared where they are used (`OrderRepository`, `PaymentGateway`, `Clock`), named in business terms, never after a technology.
- **Adapters live at the edge**: driving adapters (HTTP handlers, CLI, consumers) call use cases; driven adapters (Postgres, Stripe, SMTP) implement ports.
- **Dependency rule**: adapters → application → domain. The core never imports an adapter, a driver or a framework package.
- **No framework types in the core**: no request/response objects, ORM entities, annotations or SDK types cross into it; adapters map to and from core types.
- **One composition root** (`main`/bootstrap) wires concrete adapters into the core; nothing else instantiates infrastructure.
- **Tests through ports**: use cases are tested with in-memory fakes of their ports; adapters get their own integration tests against the real technology.
- Review check: grep the core for driver, framework and SDK imports: there are none; every new adapter implements a port declared in the core.

### One file, one responsibility

- **One file = one responsibility**: one entity, one use case, one handler group, one component. If its description needs "and", it is two files.
- **Size limits**: ~300 lines is a soft limit, ~500 a hard one; a file over it is split by responsibility, not by line count.
- **Folders by feature**, not by technical type: `orders/` holds its entity, use cases, adapters and tests; shared kernels stay small and explicit.
- **Explicit public surface**: each module exposes a small API (index/exports/package-level types); everything else is private and may change freely.
- **No dumping grounds**: no `utils`, `helpers`, `common`, `misc` files; a helper lives next to its only user or in a module named for what it does.
- **High cohesion, low coupling**: things that change together live together; modules talk through interfaces or events, never through each other's internals.
- **Replaceable modules**: deleting or swapping a module touches its folder and the composition root, nothing else.
- **No cycles**: module dependencies form a DAG; a cycle is broken by extracting the shared concept or inverting a dependency.
- Review check: each new file can be described in one sentence; no import reaches into another module's private files.

### Performance

- **Measure first**: optimize only with a profile, benchmark or production metric that shows the problem; record the before and after numbers.
- **Budgets**: explicit targets (p95 latency per endpoint, bundle size, startup time, memory) checked in CI or monitoring.
- **No N+1**: data for a list is loaded in one query or a batch (joins, `IN`, dataloaders); queries inside loops are bugs.
- **Bounded work**: every list is paginated, large payloads are streamed, unbounded reads (whole tables, whole files) are not allowed.
- **Indexes** match the queries; every new query is checked with its plan on realistic data.
- **Caching** has an owner, a key, a TTL and an invalidation strategy; never cache without knowing how it becomes stale.
- **Timeouts everywhere**: every network call, query and lock has a timeout and cancellation propagates (context, abort signals).
- **Async for slow work**: emails, reports and third-party calls that the user need not wait for go to a queue.
- **No premature optimization**: clear code first; micro-optimizations need a measurement that justifies the lost readability.
- Review check: no new query in a loop, no unbounded list, no network call without a timeout.

### Security

- **Validate at every boundary**: input from users, APIs, files and queues is parsed into typed values with allowlists, size and range limits; reject, never sanitize-and-hope.
- **Injection**: parameterized queries and safe APIs only; never build SQL, shell commands, paths or HTML by string concatenation with input.
- **Authorization on every request**: check that this user may do this action on this resource (object-level), server-side, deny by default.
- **Authentication**: proven libraries, hashed passwords (argon2id/bcrypt), short-lived tokens, secure cookies (`HttpOnly`, `Secure`, `SameSite`), CSRF protection for cookie sessions.
- **Secrets** never in code, logs, URLs or the repository; they come from the environment or a secret manager and are rotatable.
- **Least privilege**: database users, service accounts, tokens and containers get only the permissions they need.
- **Output encoding** by context (HTML, attribute, URL, JS); security headers (CSP, HSTS) on web apps.
- **No PII or secrets in logs**; personal data is minimized, encrypted in transit (TLS) and at rest where required.
- **Dependencies** are pinned and scanned (Dependabot, `npm audit`, `govulncheck`, `cargo audit`); known-vulnerable versions block the build.
- Review check: every new endpoint has an authorization check and input validation; no new string-built query or command.

### SOLID

- **Single responsibility**: a module, type or function has one reason to change. If describing it needs "and", split it.
- **Open/closed**: add behaviour by adding code (a new implementation, a new strategy, a new handler), not by growing `switch`/`if` chains in existing code.
- **Liskov substitution**: every implementation of an interface honours its full contract (inputs, outputs, errors, side effects); no implementation panics or ignores a method it "does not support".
- **Interface segregation**: interfaces are small and owned by the consumer; a consumer never depends on methods it does not call.
- **Dependency inversion**: business code depends on interfaces it defines; concrete adapters (DB, HTTP, queues, clocks, random) are injected from the composition root.
- Review check: no constructor creates its own infrastructure; no type takes more than ~5 dependencies (a sign it does too much).

### Testing

- **Pyramid**: many fast unit tests, fewer integration tests against real dependencies (containers), a few end-to-end tests of critical flows.
- **Test behaviour, not implementation**: assert on outputs and observable effects through public APIs; refactoring must not break tests.
- **Arrange, Act, Assert**: one behaviour per test, named after it (`rejects_order_without_items`); table-driven for variations.
- **Deterministic**: no `sleep`, no real clock, randomness or network in unit tests; inject clocks, id generators and seeds.
- **Fakes over mocks** for ports you own (in-memory repository); mocks only to assert interactions that are the behaviour (an email was sent).
- **Isolated**: tests do not depend on order or shared state; each creates its data and cleans up (transactions rolled back, temp dirs).
- **Every bug gets a regression test** that fails before the fix and passes after it.
- **Coverage is a signal, not a target**: uncovered critical paths matter; chasing a percentage with trivial tests does not.
- Review check: every changed behaviour has a test in the same change; the suite passes repeatedly and in parallel.
