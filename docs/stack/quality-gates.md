# Quality gates

Pass/fail criteria for a change and for a release.

- `cargo clippy -D warnings`, `cargo fmt --check` and `cargo test --workspace --all-features` green.
- Frame budget by tier: Low 30 fps minimum on a mid-range phone (33 ms), Mid 60 fps on tablet or integrated GPU, High 60 fps on a desktop GPU; no frame above 2x the budget while streaming.
- Web build under the size budget in the SPEC and starts to the menu within its time budget.
- Memory stays flat over a 30-minute session on mobile.
- The whole chain costs at most 3 ms at the default tier; with every effect off the image is still correct.
- Shadow passes together cost at most 2 ms at the default tier; no visible shimmer when walking at full speed.
- Volumetrics cost at most 1.5 ms at the default tier; the high tier states its own cost.
- Walking at full speed never produces a frame above 33 ms; the view distance setting changes memory predictably.
