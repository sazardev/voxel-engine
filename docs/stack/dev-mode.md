# Dev mode and player mode

One codebase, two builds. Player mode is the game as shipped; dev mode adds tools for testing comfortably. The split is a compile-time Cargo feature, so player builds contain no dev code (ADR 0009).

| Build   | Run                    | Features  | Logs                                | Contents                                    |
| ------- | ---------------------- | --------- | ----------------------------------- | ------------------------------------------- |
| Player  | `cargo play` (release) | none      | `warn`, `game=info`                 | the game only                               |
| Dev     | `cargo dev`            | `dev`     | `info`, `game=debug`, GPU noise off | overlay, debug hotkeys, verbose logs        |
| Profile | `cargo profile`        | `profile` | same as dev                         | dev tools plus Chrome-trace spans (release) |

`cargo dev`, `cargo play` and `cargo profile` are aliases in `.cargo/config.toml`.

## Dev tools today

- **Overlay** (F3 toggles): fps, frame time, time of day and weather.
- **Hotkeys**: F6 cycles weather, hold `]` to fast-forward time. They change the `Environment` resource, never a light or fog value directly.
- **Logs**: `RunMode::log_plugin` picks the filter by mode (`crates/game/src/run_mode.rs`).

## Rules

- Dev code lives only in `crates/game/src/dev/` and is registered in `main.rs` under `#[cfg(feature = "dev")]`, never from `GamePlugin`. A guard test fails if any file other than `lib.rs`, `main.rs`, `run_mode.rs` and `dev/` mentions it.
- Dev code observes and nudges state through the resources gameplay uses; it never contains game rules and never bypasses them.
- Every dev tool has a test; the gates run with `--all-features`, so dev code is linted and tested like the rest.
- New dev tools are a new file plus a registration in `dev.rs` (open/closed); one file per tool.
- Do not branch gameplay on `RunMode`; it only selects logging before the app starts.
- Dev features never become required to play: a player build with no dev code must run the full game.

## Planned dev tools

In-game command console, entity/chunk inspector, wireframe and chunk-border debug draws, tier override, screenshot and replay capture, log to file. Add each as its own file under `dev/`.
