# DESIGN.md: Voxel art

A world built from visible cubes. Every model and block is made of voxels at one fixed size, so the grid itself is the
style: bold silhouettes, flat-shaded faces with soft ambient occlusion, a small hand-picked palette and warm, clear light.
First or third person, offline, readable at a glance.

## Principles

- **Two voxel sizes, one grid**: the world (terrain, buildings, anything that can be dug or destroyed) is built from 25 cm voxels; characters, items and props from 12.5 cm voxels, exactly half, so every model sits on the world grid. Nothing is scaled off-grid (ADR 0015).
- **Silhouette first**: a character or object is recognizable from its shape in one color before any detail.
- **Palette, not paint**: colors come from a shared palette of 32-64 entries with ramps (shadow → light, hue-shifted), never ad hoc RGB.
- **Light does the work**: simple geometry plus ambient occlusion, a sun and soft shadows reads as rich.
- **Honest cubes**: no fake smoothing; bevels and rounding are part of the model, not a shader trick.
- **References**: Teardown (small cubes everywhere, light doing the work), Minecraft (readability), Dragon Quest Builders 2 (warm, friendly palette), Hytale (stylized voxels with strong lighting), Cube World and Trove (character silhouettes).
- First person: the player sees through the character's eyes; the HUD stays small and mostly near the edges.
- Third person: the character is part of the image; the camera keeps them readable and never clips into walls.

## Color

Palette of ramps (approx.); every material takes a ramp of 4-5 steps, shadows shift toward blue/violet, highlights toward yellow.

| Token | Value | Use | | --- | --- | --- | --- |
| `grass-ramp` | `#8BC34A` → `#3F7D2E` | same, darker ambient | Terrain top |
| `stone-ramp` | `#C9C4BC` → `#5F5B66` | same | Rock, buildings |
| `wood-ramp` | `#C98B4B` → `#5A3A26` | same | Trees, props |
| `water` | `#4FA8E0` at 70% alpha | `#1F5F9A` | Water |
| `sun` | `#FFF1C9` | `#8FA8FF` (moon) | Main light |
| `ambient` | `#9FB7E8` | `#27324F` | Shadow fill |
| `hud-surface` | `#FFFFFF` at 85% | `#1B1F2A` at 85% | HUD panels |
| `hud-text` | `#1B1F2A` | `#F4F1EA` | HUD text |
| `accent` | `#FFC93C` | `#FFC93C` | Selection, loot |
| `danger` | `#E5484D` | `#E5484D` | Health, hits |
| tone-map | ACES or AgX filmic curve | Final image |
| grade-lut | one 32³ LUT per environment, blended 400ms on transitions | Mood per place |
| saturation | 0.9-1.15 around neutral | Taste per scene |

- Every token has a light and a dark value; the app follows `prefers-color-scheme` (or the OS setting) and offers a manual Light / Dark / System switch.
- Dark is not inverted light: surfaces get lighter as they rise, saturated colors are toned down (lower chroma) to avoid glare.
- Glow: text gets a soft halo of its own color (GUI `text-shadow: 0 0 2px currentColor, 0 0 8px color-mix(in oklch, currentColor 40%, transparent)`); TUI keeps the palette's brightest step for focus to imitate it.
- Gradients and images are reduced to the palette with ordered dithering (Bayer 4×4 or 8×8); error diffusion (Floyd-Steinberg) only for photos.
- Neon accents: magenta `#FF2BD6`, cyan `#00F0FF`, violet `#9D4EDD` on near-black `#0B0612`.
- Success/danger pairs never rely on red vs green alone: add icons (`✓`, `✕`), labels or patterns; charts use a color-blind safe set (e.g. Okabe-Ito: `#E69F00`, `#56B4E9`, `#009E73`, `#F0E442`, `#0072B2`, `#D55E00`, `#CC79A7`).

## Typography

| Role      | Size / line height | Font                                                  | Use                    |
| --------- | ------------------ | ----------------------------------------------------- | ---------------------- |
| title     | 48 / 56            | Chunky display (approx. Fredoka)                      | Menus, titles          |
| hud-label | 16 / 20            | Pixel-leaning sans (approx. Silkscreen or Inter bold) | Item names, counts     |
| body      | 16 / 24            | Inter                                                 | Descriptions, settings |
| number    | 20 / 24            | Tabular numerals                                      | Stats, counts          |

- HUD text always has a 1-2px dark outline or panel behind it; world-space labels scale with distance within limits.

## Spacing and layout

- World unit: 1 m = 4 world voxels (25 cm) = 8 model voxels (12.5 cm); the HUD uses a 4px grid scaled by the UI scale setting (75-150%).
- Hotbar and crosshair centered; health and stats in the corners, inside a 5% safe margin.
- Menus are single-column lists with large targets (min 48px) for mouse and gamepad.
- Minimum window 800×600; layouts reflow from one to three panes (sidebar, list, detail) as the window grows; splitters remember their width.
- Respect the OS title bar and window controls (left on macOS, right on Windows/Linux).
- Respect safe areas (notch, home indicator, status bar): content padded by the platform insets, backgrounds may bleed under them.
- Phone first (360-430 pt wide), tablet layouts at ≥ 600 pt use two panes (list-detail).
- Primary actions in the thumb zone: the lower half of the screen.
- Mobile first: design at 360 px, then adapt at `sm 640`, `md 768`, `lg 1024`, `xl 1280`, `2xl 1536` px.
- Fluid type and spacing with `clamp()` between the smallest and largest breakpoint (e.g. body `clamp(1rem, 0.95rem + 0.25vw, 1.125rem)`).
- Components adapt to their container with container queries (`@container`), not to the viewport.
- Content measure 60-75 characters; the page never scrolls sideways at 320 px.
- Crosshair at center; hotbar at the bottom center; stats in the corners; held item in the lower right with its own FOV.
- Camera 3-4 m behind and 0.5 m to the right; the crosshair follows the real aim ray; the character stays in the lower third.
- 8 px grid (4 px for fine adjustments), rows 44-48 px, controls 36-40 px high, body text 15-16 px.
- Sections separated by 24-32 px; cards with 16-24 px padding.

## Shape and elevation

| Token                | Value                                                             | Use                           |
| -------------------- | ----------------------------------------------------------------- | ----------------------------- |
| voxel-size           | 25 cm world, 12.5 cm models                                       | Terrain and buildings; models |
| ao                   | baked per vertex, 4 levels, strength 0.35                         | Soft corner darkening         |
| shadow-color         | tinted by the ambient `fill` color, never pure black              | Depth and mood                |
| hud-radius           | 4px (or 0 for a hard cube look)                                   | HUD panels                    |
| outline              | optional 1px inverted-hull or edge detect                         | Selection, hover              |
| distance-fog         | exponential, color = sky at the horizon, density by view distance | Hides chunk edges, adds scale |
| height-fog           | denser below a height, falls off upward                           | Valleys, caves, mornings      |
| aerial-perspective   | distant objects shift toward the haze color and lose contrast     | Depth                         |
| dof-cinematic        | f/2.8, focus on the subject, hexagonal bokeh                      | Cutscenes, photo mode         |
| tilt-shift           | focus band with blur above and below                              | Miniature look (HD-2D style)  |
| bloom                | HDR threshold 1.0, intensity 0.15                                 | Highlights, emissive          |
| vignette             | 0.2                                                               | Focus toward the center       |
| grain                | 0.03, animated at 24 fps                                          | Film texture                  |
| chromatic-aberration | 0.002 at edges                                                    | Lens character                |
| contact-shadow       | short screen-space ray, 0.3 m                                     | Grounding small objects       |
| volumetrics          | froxel fog, density 0.01-0.08 by environment, anisotropy 0.6      | Light shafts, haze            |
| light-shaft          | sun and 2-4 key lights scatter; others do not                     | God rays, windows, torches    |

- GUI scanlines: a fixed overlay `repeating-linear-gradient(to bottom, rgb(0 0 0 / 0.15) 0 1px, transparent 1px 3px)`, pointer-events none; optional vignette and 1-2% barrel curvature via a shader or SVG filter.
- TUI: no overlays; the effect comes from the phosphor palette, a dim surface one step above the background and generous blank lines.
- Shadows and fills use dither patterns instead of opacity; GUI renders at integer scale with `image-rendering: pixelated`.
- TUI: dithering with block/shade characters `░▒▓` or braille dots for sub-cell detail.
- Glass surfaces: `background: color-mix(in oklch, var(--surface) 60%, transparent)`, `backdrop-filter: blur(16px) saturate(140%)`, 1 px border `rgb(255 255 255 / 0.18)` (dark) or `rgb(0 0 0 / 0.08)` (light).
- Only on floating layers (navigation, sheets, popovers), never stacked more than two deep.
- Grain is a tiled 128×128 noise image (or SVG `feTurbulence`, baseFrequency 0.8) at 4-8% opacity with `mix-blend-mode: overlay`, on backgrounds and large surfaces only.
- Never animated, never over text or icons; disabled in high-contrast and print.
- GUI glow: `box-shadow: 0 0 6px var(--accent), 0 0 18px color-mix(in oklch, var(--accent) 50%, transparent)` on focus and primary elements only.
- TUI: bright ANSI colors (`13`, `14`) and bold for the glowing elements; everything else stays dim.

## Motion

| Token          | Value                                              | Use                    |
| -------------- | -------------------------------------------------- | ---------------------- |
| block-break    | 8 crack frames, 0.4s, particles of the block color | Mining                 |
| block-place    | 120ms scale 0.9 → 1.0                              | Placing                |
| camera-blur    | camera rotation blur at 50% of object blur         | Mouse look             |
| ui-ease        | 150ms ease-out                                     | Menus, hotbar          |
| focus-pull     | 250-400ms ease-in-out                              | Changing subject       |
| motion-blur    | shutter 180°, per-object velocity buffer, max 16px | Fast movement          |
| fov            | 70-110, default 90, slider in settings             | Comfort                |
| head-bob       | 0.3 default, 0 with reduce motion                  | Walking                |
| mouse-look     | raw input, no smoothing by default                 | Aiming                 |
| spring-arm     | collision shortening, recover 200ms                | Walls and tight spaces |
| aim-mode       | FOV -15°, camera over the shoulder, 150ms blend    | Aiming                 |
| character-fade | dither out when the camera is closer than 0.8 m    | No clipping pop        |

- Reduced motion: no head bob, no camera shake, fewer particles.
- Durations: 120 ms (hover, press), 200 ms (enter/leave of small elements), 280 ms (panels, dialogs); exits 30% shorter than entries.
- Easing: `cubic-bezier(0.2, 0, 0, 1)` for entering, `cubic-bezier(0.4, 0, 1, 1)` for leaving; movement ≤ 8 px, mostly opacity.
- Only `opacity` and `transform` animate; never layout properties.

## Components

- **Hotbar**: 9 slots, selected slot enlarged, item icons rendered from the voxel models.
- **Crosshair**: small cross or dot, changes when an interaction is available.
- **Inventory/crafting**: grid with drag and drop and keyboard/gamepad navigation, hover tooltip with the item's voxel preview.
- **Health/stamina**: segmented bars or hearts, always with numbers available in settings.
- **Pause and settings**: graphics tiers, FOV, sensitivity, key bindings, accessibility.
- A menu bar (or app menu) lists every command with its shortcut; context menus on right click.
- Every action has a keyboard shortcut following the platform (`Cmd` on macOS, `Ctrl` elsewhere); a command palette (`Cmd/Ctrl+K`) finds any command.
- Pointer states: hover, pressed, focus ring, and the right cursor (pointer, text, resize).
- Bottom navigation with 3-5 destinations; back follows the platform (swipe on iOS, system back on Android).
- Touch targets at least 44×44 pt (iOS) / 48×48 dp (Android), 8 pt apart.
- Gestures (swipe to delete, pull to refresh) always have a visible alternative.
- Haptics only to confirm meaningful actions (success, selection, error), never on scroll.
- Navigation collapses to a menu button below `md`; tables become stacked cards or scroll inside their own container.
- Hover is an enhancement: every action works with touch and keyboard (`@media (hover: hover)` for hover-only styles).
- **Crosshair**: dot or cross, 4-8px, changes shape on interactable or hit; size and opacity adjustable.
- **Vitals**: health and stamina as bars or segments; low-health feedback by vignette + sound, never only red.
- **Hotbar and held item**: 9 slots, selected one enlarged, durability and counts readable at a glance.
- **Prompts**: "Hold E to open", glyphs match the active device (keyboard or gamepad), fade with distance.
- **Compass / markers**: objectives and waypoints with distance, can be hidden.
- **Damage and pickups**: short directional indicators and stacked pickup messages, never covering the center.
- **Photo mode**: hides the HUD, frees the camera and exposes focus, exposure and grade presets.
- **Main menu**: Continue, New world, Load, Settings, Quit; the last world is one press away; works with keyboard, mouse and gamepad.
- **Pause**: stops the simulation, shows Resume, Settings, Save and quit; never loses unsaved progress silently.
- **Graphics**: presets (Low, Medium, High, Ultra, Custom) and a list of effects (shadows, volumetrics, ray-traced light, depth of field, motion blur, grading, fog) each with off / low / high and its cost in ms; a live preview.
- **Controls**: every action rebindable for keyboard, mouse and gamepad; sensitivity, invert, hold/toggle options; conflicts are shown.
- **Accessibility**: UI scale, subtitles, color-blind modes, reduce motion, photosensitivity mode.
- **Loading**: progress and a tip; the world streams in without a frozen window.
- GUI: group settings into 4-7 sections by what the user is trying to do (Profile, Notifications, Security, Billing), not by technical area; the section list is a left column on wide screens and a list screen on narrow ones.
- GUI: each setting is a row: label (medium weight) and one line of helper text on the left, the control on the right; rows 56 px minimum, dividers between rows, section titles as small headings.
- GUI: toggles and selects apply instantly and confirm with a quiet "Saved" near the control; text inputs and anything with side effects use an explicit "Save" button with "Cancel", shown only when the form is dirty.
- GUI: never mix instant and saved controls inside one card; if a section must, label the saved part with its own button.
- GUI: show the effective value when a setting is inherited ("Using workspace default: English") with a "Reset to default" link per setting.
- GUI: settings search at the top of the screen filters rows by label and helper text, highlights matches, jumps to the section, and shows "No settings match "x"" when empty.
- GUI: danger zone is the last section, separated by a divider, titled "Danger zone", rows with danger-outlined buttons ("Delete workspace"); each action opens a confirmation that requires typing the resource name.
- TUI: a two-pane list: sections on the left, rows on the right as `label  value`; Enter edits, Space toggles, `/` searches, `r` resets the row to default; changes marked with `*` until saved with `w`.

## Accessibility

- Contrast of HUD text on any background ≥ 4.5:1 through panel or outline; do not rely on color alone for health or danger.
- Subtitles and sound cues for important events; UI scale, FOV and sensitivity adjustable; all bindings remappable.
- Reduce-motion and photosensitivity-safe flashes (no more than 3 flashes per second).
- Scanlines and glow are off with reduced motion/transparency preferences or high contrast; body text never sits under the overlay at less than 7:1.
- Depth of field is optional in settings and always off in aim and precision moments.
- Text on glass keeps 4.5:1 against the worst background behind it; with `prefers-reduced-transparency` or no `backdrop-filter` support, surfaces become solid.
- Each effect has its own toggle; all go off with "reduce motion"; bloom never pushes text below contrast limits.
- Off by default in first person; a strength slider and a toggle; always zero with "reduce motion".
- Glow never carries meaning on its own; it is off in high contrast and reduced-transparency modes.
- A toggle and strength slider for volumetrics; the game stays readable with it off.
- FOV, sensitivity, head bob, sway and sprint zoom adjustable; a toggle for crouch and sprint (hold or toggle).
- Camera distance, shoulder side, auto-recenter and shake adjustable; a toggle to switch to first person.
- Every screen is checked with a color-blindness simulator (deuteranopia, protanopia, tritanopia) before it is done.
- With `prefers-reduced-motion: reduce` (or the OS/app setting), movement is replaced by ≤ 150 ms opacity fades; parallax, auto-playing animation and spinners that move across the screen are removed.
- GUI: semantic HTML/native controls first, landmarks, headings in order, every control named, live regions for async updates; tested with VoiceOver and NVDA/TalkBack.
- TUI: an accessible mode (flag or env var) prints linear text without box art, spinners or cursor tricks, one line per state change, readable by screen readers.
- Text contrast at least 7:1 (4.5:1 for large text), targets at least 44×44 px, no timing that cannot be extended, no flashing at all.
- Line height ≥ 1.5, paragraph spacing ≥ 2× font size, measure ≤ 80 characters, text resizable to 200% without loss.
- Context help for every non-obvious input; error messages say how to fix the problem.
- The HUD scales 75-150%, can be reduced to essentials, and every element can be toggled; subtitles and sound cues for key events.
- Toggles are `role="switch"` with the label and state announced; sections are landmarks with headings; the search field is labelled and results count is announced.

## Rules for agents

- Make every model and texture on the voxel grid with the shared palette; never introduce a free color.
- Light with the shared environment (sun, ambient, AO); do not tint materials to fake shading.
- Voxel sizes are constants in one place (`voxel_core::VOXEL_SIZE_METRES`, `MODEL_VOXEL_SIZE_METRES`); never hardcode a scale.
- HUD reads game state and sends intents; HUD tokens come from one theme file.
- Never require the mouse: every flow works with the keyboard alone, with visible focus.
- Persist window size, position and pane widths between sessions.
- Use the platform's text scaling (Dynamic Type / font scale) and test at the largest setting.
- Never place interactive elements under the status bar or the home indicator.
- Test every screen at 360, 768, 1280 and 1920 px wide and at 200% browser zoom.
- Use relative units (`rem`, `%`, `ch`) for sizes; `px` only for borders and hairlines.
- Never write a color that only exists in one theme; test every screen in both before it is done.
- Reuse the duration and easing tokens; no ad-hoc timings.
- Fog color and density come from the environment (time, weather, biome); gameplay code never sets them.
- Grading is a LUT in the post chain; never tint individual materials to change the mood. The HUD is drawn after grading.
- Never blur the HUD or the crosshair area; focus follows a target, not a constant.
- Dither at build time or once at load, never per frame.
- Use `@supports (backdrop-filter: blur(1px))` for the glass styles and keep the solid version as the default.
- One shared grain asset under 10 KB; do not generate noise per component.
- Film effects are the last passes before the HUD and stay subtle; if the image needs them to look good, fix the lighting.
- Motion blur reads the velocity buffer; never fake it by blending previous frames in gameplay code.
- Shadow quality is a tier (map size, cascades, filter); never tune bias per object.
- Volumetrics is one pass with off / low / high tiers; fog parameters come from the environment, never from a material.
- Every animation is behind the motion preference check; test with it on.
- Never convey state only visually; every icon-only control has an accessible name.
- The HUD only reads game state and sends intents; never put game rules in HUD code.
- Settings apply live when possible and are saved locally; a setting that needs a restart says so.
- Every setting states its default and can be reset to it.
- Never apply a destructive or billing change instantly.
- Never hide a setting that is on by default from enterprise or admin users without saying who controls it.
