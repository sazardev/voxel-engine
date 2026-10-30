# Camera and input

First and third person rig, comfort settings and input actions.

- Mouse look is raw input, not smoothed by default; sensitivity and invert are settings; pitch is clamped to ±89°.
- The held item (view model) renders in its own pass with its own FOV so it does not stretch with the world FOV.
- Head bob, sway and camera shake have a strength slider and go to zero with "reduce motion".
- The arm shortens when voxels block it (ray cast against the grid) and recovers smoothly; the camera never enters a solid block.
- The character fades out (dither) when the camera gets too close instead of popping.
- Aim mode narrows FOV and moves the camera over the shoulder; the crosshair follows the real aim ray.

Current implementation (`crates/game/src/input.rs`, `player/`, `camera/`):

- Actions (`PlayerAction`): `Move` (WASD, left stick), `Look` (raw mouse delta in pixels), `Turn` (right stick as a rate), `Jump` (Space, South), `Sprint` (left Shift, left stick press), `SwitchView` (V, Select). Mouse and stick look are separate actions because one is a distance per frame and the other a speed; both end in the same `Look` (yaw, pitch) on the player.
- Look is applied every frame without smoothing: 0.0022 rad per pixel, 3 rad/s at full stick, pitch clamped to ±89°. The mouse only turns the view while the cursor is captured; the game captures it at start, Escape releases it and a left click captures it again.
- Movement runs in `FixedUpdate` (Bevy's default 64 Hz) through a pure `step` rule: walk 4.3 m/s, sprint 6.5 m/s, gravity 28 m/s², jump 8.4 m/s (about 1.25 m). Horizontal speed follows the input at once, without acceleration, so control responds immediately. Collision is the voxel-grid sweep of ADR 0014; voxels of unloaded chunks count as solid and the body waits until the ground under it is loaded.
- The camera and the player model are placed every frame between the last two fixed steps (`Time<Fixed>::overstep_fraction`), so motion is smooth at any frame rate.
- One `CameraRig` holds only the view (first or third person). First person sits at the eye (1.62 m). Third person sits 0.5 m right of the eye and 3.5 m behind; both offsets are ray cast against the grid and keep 0.2 m from the first solid voxel, so the camera never enters a block. The player model is hidden in first person.
- The world streams around the player: `StreamFocus` follows the body.
