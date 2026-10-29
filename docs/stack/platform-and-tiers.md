# Platform and tiers

Capabilities decide what runs, never the platform. Tiers add or remove passes and resolution.

- Probe the device at startup (GPU limits, memory, WebGPU availability, touch) and pick a render tier; the player can override it. The probe result decides what is allowed, never a platform `#[cfg]` scattered in gameplay.
- Art direction is one: the same palette, models and light colors on every tier; tiers only add or remove passes and resolution.
- Stylized light, not realism: strong key light, tinted soft shadows, colored bounce, emissive glow; check each scene in the Low tier first.
- Every pass declares its tier, its cost budget in ms and a fallback; High and Ultra never ship a visual that Low breaks.
- Memory budgets per tier (chunks resident, texture sizes, view distance); mobile and web are the tightest and drive the limits.
- Web has no threads by default: meshing and generation have a single-thread path with a per-frame time budget; use workers and SharedArrayBuffer only when the page is cross-origin isolated.
- Touch controls (move stick, look drag, action buttons) map to the same input actions as keyboard and gamepad.
- Battery and heat matter: a 30 fps cap option on mobile, a frame limiter, and the High tier off by default on phones.
- No network layer; saves and settings are local on every platform.
