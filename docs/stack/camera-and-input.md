# Camera and input

First and third person rig, comfort settings and input actions.

- Mouse look is raw input, not smoothed by default; sensitivity and invert are settings; pitch is clamped to ±89°.
- The held item (view model) renders in its own pass with its own FOV so it does not stretch with the world FOV.
- Head bob, sway and camera shake have a strength slider and go to zero with "reduce motion".
- The arm shortens when voxels block it (ray cast against the grid) and recovers smoothly; the camera never enters a solid block.
- The character fades out (dither) when the camera gets too close instead of popping.
- Aim mode narrows FOV and moves the camera over the shoulder; the crosshair follows the real aim ray.
