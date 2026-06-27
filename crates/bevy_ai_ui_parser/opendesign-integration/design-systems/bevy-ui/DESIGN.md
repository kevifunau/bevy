# Bevy UI Design System

Derived from: Bevy BUI IR parser source code (`crates/bevy_ai_ui_parser`)
Purpose: Provide an authoritative CSS capability model and an asset-first authoring contract so design agents produce UI that the Bevy ECS renderer can reproduce faithfully.

Compatibility target: Bevy UI authoring follows Unity UI Toolkit UXML/USS as the supported syntax surface because that is the closest mature game-UI HTML/CSS-like model. If Unity UI Toolkit supports a UXML/USS feature, OD may preserve and emit that intent in the Bevy UI output contract; the Bevy engine/parser can then continue implementing support against that stable target.

---

## Machine-Readable Capability Contracts

The following JSON files are the **authoritative constraint sources** for AI-generated Bevy UI. They supersede any conflicting prose guidance in this document.

| Contract | Path | Purpose |
|----------|------|---------|
| `bevy_capability_contract.json` | `design-systems/bevy-ui/bevy_capability_contract.json` | CSS property support levels (P0 Native / P1 HelperLayer / P2 Approximation / Forbidden). Generated from `css_metadata.rs`. |
| `bevy_html_tag_map.json` | `design-systems/bevy-ui/bevy_html_tag_map.json` | HTML tag → BuiNodeType mapping. Supported tags, forbidden tags, required attributes. Generated from `ir.rs` and `spawn.rs`. |

**OD UI Programmer must:**
1. Load both contracts before generating HTML/CSS.
2. Never emit forbidden tags or CSS properties.
3. Always include `data-action` on interactive elements mapped to `Button`, `Toggle`, `Slider`.
4. Always include `id` on elements with `data-binding`.
5. Run `run_self_check.sh` (Phase 2) before declaring output complete.

---

## 1 Visual Theme & Atmosphere

Bevy UI is a **game HUD and menu system**. It runs inside an ECS-driven rendering pipeline, not a browser.

That means visual fidelity should come from **real game art assets first** and CSS second. HTML/CSS is the assembly language for layout, text, semantic hooks, and lightweight styling. It is not the primary painter for ornate game UI chrome.

Every CSS property is classified into one of four tiers:

| Tier | Name | Meaning |
|------|------|---------|
| P0 | **Native** | The Bevy UI renderer reproduces the property exactly. Use freely. |
| P1 | **HelperLayer** | Achieved by overlaying extra helper nodes (solid bands, box_shadow children, absolute-positioned border strips). Prefer PNGs when possible. |
| P2 | **Approximation** | Rough visual mimicry through composite node strategies. Accept visible imperfections. Use only as last resort. |
| — | **Forbidden** | The renderer has no equivalent. Must never appear in output. |

**Dark mode only.** Prefer Unity USS transitions and ECS state changes over browser-only keyframe animation.

## 1.1 Asset-First Workflow Contract

The preferred Open Design export for Bevy UI is:

```text
project-root/
  index.html
  bevy-ui.assets.json
  assets/
    backgrounds/
    panels/
    buttons/
    icons/
    sliders/
    scrollbars/
    atlas/
```

`index.html` and its CSS/JS are the **source of truth** for UI structure,
layout, text, browser prototype behavior, image URLs, and semantic hooks such as
`id` and `data-action`.

`bevy-ui.assets.json` is an optional **OD-authored asset manifest**. It is not a
Bevy-generated IR file and it must not replace information already present in
HTML/CSS/JS. Treat it as parser hints plus a validation contract for game-UI
asset assembly. A Bevy parser should be able to parse the HTML without it, but
may use it to improve image semantics, 9-slice setup, state textures, atlas
mapping, and handoff validation.

Use this contract when the UI contains any of the following:
- painted panel frames
- stylized button skins
- logos
- skill or equipment icons
- slider skins
- scrollbar skins
- animated atlas art
- decorative borders or fantasy/sci-fi chrome

Guiding rule:
- layout and semantics belong to HTML/CSS
- game art belongs to image assets
- 9-slice, atlas, state textures, and semantic icon bindings should be
  mirrored into `bevy-ui.assets.json` when they are useful parser hints or
  validation data

If a visual treatment would normally be created by an artist in a shipped game, assume it should be an asset, not a CSS trick.

### `bevy-ui.assets.json` Responsibilities

Use the manifest for:

- **Stable asset inventory** — `id`, `kind`, `path`, usage, dimensions when known. Schema: [`bevy_ui_assets_schema.json`](bevy_ui_assets_schema.json).
- **Node-to-asset hints** — `nodeSelector` (CSS selector), `role`, binding to HTML elements.
- **State texture mapping** — per-state overrides: `idle`, `hover`, `pressed`, `disabled`, `focused`, `checked`, `unchecked`.
- **9-slice / slicer configuration** — `slice9.left/right/top/bottom` + `scaleMode: "nine-slice"` for panels, buttons, bars.
- **Sprite atlas metadata** — `atlas.columns/rows/frameWidth/frameHeight/padding/animationFps` for animated icons.
- **Semantic icon mappings** — explicit `kind: "icon"` entries where CSS alone cannot convey meaning.
- **Validation checkpoints** — `validationCheckpoints` array with expected bounds for browser-vs-engine parity checks.
- **Delivery lifecycle** — `adoptionStatus`: `candidate` → `adopted` → `deprecated`. Only `adopted` assets ship to engine.
- **Source provenance** — `sourceRef.wireframeRegion`, `sourceRef.artPrompt`, `sourceRef.svgSource`, `sourceRef.comparisonPath`.

**Asset kind taxonomy:**

| Kind | Description | Typical scaleMode |
|------|-------------|-------------------|
| `background` | Full-screen or panel background image | `cover` |
| `panel` | Panel shell / frame | `nine-slice` |
| `button` | Button face (idle state) | `nine-slice` or `stretch` |
| `button-state` | Hover/pressed/disabled button variant | `nine-slice` or `stretch` |
| `icon` | Standalone icon (play, settings, etc.) | `contain` |
| `slider-track` / `slider-thumb` | Slider component parts | `stretch` / `contain` |
| `scrollbar-track` / `scrollbar-thumb` | Scrollbar component parts | `stretch` / `contain` |
| `progress-fill` | Progress bar fill overlay | `stretch` |
| `border` / `divider` | Decorative line elements | `tile-x` / `tile-y` / `stretch` |
| `decoration` | Non-functional ornamental art | `contain` |
| `atlas` | Multi-frame sprite sheet | `none` (parsed via atlas config) |
| `rasterized-svg` | SVG converted to PNG via pipeline | Varies by role |
| `ai-generated` | MiniMax/ComfyUI produced asset | Varies by role |
| `hand-painted` | Artist-created high-fidelity asset | Varies by role |
| `reference-only` | Mood board or style reference, NOT shipped | N/A |

Do not use the manifest for:

- Changing layout that already exists in HTML/CSS.
- Inventing controls not present in HTML.
- Hiding required images from HTML/CSS so the browser preview no longer works.
- Representing Bevy's generated BUI/IR output.


---

## 2 Color Palette & Roles

### Surface (Background)
| Token | Hex | Role |
|-------|-----|------|
| `--bg` | `#111111` | Page / viewport background |
| `--surface` | `#1c1c1c` | Card / panel / HUD panel fill |
| `--surface-warm` | `var(--surface)` | Warm-tinted surface (identical in Bevy; reserved for future warmth shift) |

### Foreground
| Token | Hex | Role |
|-------|-----|------|
| `--fg` | `#ffffff` | Primary text, high-emphasis icons |
| `--fg-2` | `var(--fg)` | Secondary text (alias; renderer treats same as fg) |
| `--muted` | `#888888` | Disabled / low-emphasis labels |
| `--meta` | `var(--muted)` | Metadata / captions |

### Border
| Token | Hex | Role |
|-------|-----|------|
| `--border` | `#333333` | Panel outlines, dividers |
| `--border-soft` | `var(--border)` | Soft separator (alias) |

### Accent (Interactive)
| Token | Hex | Role |
|-------|-----|------|
| `--accent` | `#f97316` | Buttons, active tabs, focus rings |
| `--accent-on` | `#000000` | Text on accent background |
| `--accent-hover` | `#e8790e` | Hover state |
| `--accent-active` | `#d97006` | Active / pressed state |

### Semantic
| Token | Hex | Role |
|-------|-----|------|
| `--success` | `#22c55e` | Positive status |
| `--warn` | `#fbbf24` | Warning |
| `--danger` | `#ef4444` | Error / health-critical |

### Supported Color Formats
- Hex: `#rrggbb`, `#rgb`
- Named: CSS named colors (`red`, `blue`, `transparent`, etc.)
- `oklch()` — Bevy UI parser accepts oklch values and converts internally
- `color-mix()` — Parser resolves at parse time; output is a single resolved value

Unsupported: `rgb()`, `hsl()`, `rgba()`, `hsla()` (parser does not handle these; convert to hex or oklch first).

---

## 3 Typography Rules

### Type Scale
| Level | Token | Size | Weight | Tracking | Leading | Use |
|-------|-------|------|--------|----------|---------|-----|
| Display | `--text-4xl` | 48px | 700 | 0 | 1.0 | Splash titles |
| Display-sm | `--text-3xl` | 36px | 700 | 0 | 1.0 | Section hero |
| Heading | `--text-2xl` | 24px | 600 | 0 | 1.2 | Panel titles |
| Subheading | `--text-xl` | 20px | 600 | 0 | 1.2 | Group labels |
| Body-lg | `--text-lg` | 18px | 400 | 0 | 1.4 | Emphasized body |
| Body | `--text-base` | 16px | 400 | 0 | 1.4 | Default text |
| Body-sm | `--text-sm` | 14px | 400 | 0 | 1.4 | Compact text |
| Label | `--text-xs` | 12px | 600 | 0 | 1.0 | Tags, badges |
| Micro | `--text-2xs` | 10px | 700 | 0 | 1.0 | Tiny indicators |

### Font Family Mapping
The Bevy UI parser maps CSS font-family names to system font files:

| CSS name | System font file |
|----------|-----------------|
| `menlo` | `Menlo.ttc` |
| `palatino` | `Palatino.ttc` |
| `songti` | `Songti.ttc` |
| `heiti` | `Heiti.ttc` |
| `monaco` | `Monaco.ttf` |
| `courier` | `Courier.ttf` |
| `stfangsong` | `STFangsong.ttf` |
| `stheiti` | `STHeiti.ttf` |
| `stkaiti` | `STKaiti.ttf` |
| `stsong` | `STSong.ttf` |
| `futura` | `Futura.ttf` |
| `helvetica` | `Helvetica.ttf` |
| `arial` | `Arial.ttf` |
| `inter` | `Inter.ttf` (bundled) |
| `roboto` | `Roboto.ttf` (bundled) |

Generic families: `sans-serif` → `Inter.ttf`, `monospace` → `Menlo.ttc`, `serif` → `Palatino.ttc`

### Font Weight Mapping
| CSS value | Rendered weight |
|-----------|----------------|
| `normal` / `400` | 400 |
| `bold` / `700` | 700 |
| `100`–`300` | 400 (fallback) |
| `500`–`600` | 700 (fallback) |
| `800`–`900` | 700 (fallback) |

Bevy UI only renders 400 and 700. All other weights collapse to the nearest tier.

---

## 4 Component Stylings

### Interactive Node (Button / Toggle)
``<div class="btn" data-action="action_name">Label</div>``
- Use `data-action` attribute to bind to ECS interaction handler.
- Styling: `background-color: var(--accent); color: var(--accent-on); border-radius: var(--radius-md); padding: var(--space-2) var(--space-4);`
- States: `:hover` → `background-color: var(--accent-hover)`, `:active` → `background-color: var(--accent-active)`
- Event aliases: `data-action` and `data-action-press` emit press actions; `data-action-hover-enter` and `data-action-hover-exit` emit hover enter/exit actions.
- Disabled controls: use `disabled="true"` or `aria-disabled="true"`; the runtime maps this to the `disabled` state and blocks action dispatch.
- Toggle controls: use `role="switch"`, `role="checkbox"`, `data-toggle="true"`, or `input type="checkbox"` with `checked="true"` / `aria-checked="true"` for initial checked state.
- Runtime behavior must be mirrored in a BUI action contract: `<script type="application/json" data-bui-actions>`, `window.BUI_ACTIONS = {...}`, or `Bui.registerActions({...})`. Browser JS may drive the OD prototype, but Bevy executes the declared action model through ECS.

### Toggle / Checkbox

```html
<input
  id="audio_toggle"
  type="checkbox"
  checked="true"
  data-binding="settings.audio_enabled"
  data-action-change="audio.changed"
/>
```

- `input type="checkbox"`, `role="switch"`, `role="checkbox"`, and `data-toggle="true"` compile to Bevy toggle widgets.
- `checked="true"` and `aria-checked="true"` mark the initial checked state.
- `data-binding` maps the checked boolean into `BuiStateStore`.
- Pressing the toggle emits `value_changed` through `data-action-change` / `data-action-value-changed`.
- Runtime writes both `<id>.checked` and the bound source key as boolean values.

### Text Input / InputField

```html
<input
  id="account_input"
  type="text"
  value="Racer"
  placeholder="Account"
  data-binding="login.account"
  data-action-change="account.changed"
  data-action-submit="account.submit"
  data-action-focus="account.focus"
  data-action-blur="account.blur"
/>
```

- `input type="text|password|search|email|url|tel|number"` and `textarea` compile to Bevy `text_input`.
- `value` becomes the initial editable value; `placeholder` is shown when empty and unfocused.
- `data-binding` maps the edited value into `BuiStateStore`.
- `data-action-change` / `data-action-value-changed` emits `value_changed`.
- `data-action-submit` emits `submit` when the focused input receives Enter.
- `data-action-focus` and `data-action-blur` emit focus lifecycle actions.
- `textarea` enables multi-line text input; single-line `input` disables newlines.

### Slider

```html
<input
  id="volume_slider"
  type="range"
  min="0"
  max="100"
  value="65"
  step="5"
  data-binding="settings.volume"
  data-action-change="volume.changed"
/>
```

- `input type="range"`, `role="slider"`, and `data-slider="true"` compile to Bevy `slider`.
- `min`, `max`, `value`, and `step` map to slider range/value semantics.
- `aria-valuemin`, `aria-valuemax`, `aria-valuenow`, and `aria-orientation` are accepted aliases.
- `data-binding` maps the numeric value into `BuiStateStore`.
- `data-action-change` / `data-action-value-changed` emits `value_changed`.
- Current runtime support covers the headless value contract (`SliderValue` changes -> state/action). Full pointer drag / keyboard adjustment should use Bevy UI Widgets `SliderPlugin` integration as the next runtime layer.

### DropDown / DropdownField

```html
<select
  id="difficulty_select"
  name="difficulty"
  data-binding="settings.difficulty"
>
  <option value="easy">Easy</option>
  <option value="hard" selected="true" data-action-select="difficulty.changed">Hard</option>
</select>
```

- `select`, `role="combobox"`, and `data-dropdown="group-name"` compile to a dropdown group.
- `option` and `role="option"` compile to selectable option items.
- `name`, `data-dropdown`, or the select node id becomes the dropdown group name.
- `data-binding` maps the selected option value into `BuiStateStore`.
- `selected="true"` and `aria-selected="true"` mark the initial selected visual state.
- `data-action-selection-changed` / `data-action-select` emits `selection_changed`.
- Current runtime support covers the headless selection contract (option press -> state/action/selected visual state). Popup open/close, keyboard navigation, and gamepad navigation should layer on this contract.

### ScrollView / Scroller

```html
<section
  id="inventory_scroll"
  class="inventory-scroll"
  data-scroll-view="true"
  data-scroll-binding="inventory.list"
  data-action-scroll="inventory.scrolled"
>
  ...
</section>
```

- `data-scroll-view="true"`, scroll-like classes such as `scroll-view` / `inventory-scroll`, or CSS `overflow: auto|scroll` compile to Bevy scroll-view semantics.
- `overflow-x` and `overflow-y` map to the matching Bevy scroll axes. Use `overflow-y: auto` for vertical lists and `overflow-x: auto` for horizontal carousels.
- `data-scroll-binding` maps runtime scroll offsets into `BuiStateStore` as `<source>.scroll_x` and `<source>.scroll_y`.
- The node id also writes `<id>.scroll_x` and `<id>.scroll_y`.
- `data-action-scroll` emits the `scroll` action trigger when `ScrollPosition` changes.
- Current runtime support covers mouse wheel input plus `ScrollPosition` state/action dispatch. Drag thumb and gamepad scroll input should layer on the same contract.

### Tab Navigation
```
<div class="tab-bar">
  <div class="tab" data-tab="tab-1" data-tab-group="settings">Tab 1</div>
  <div class="tab" data-tab="tab-2" data-tab-group="settings">Tab 2</div>
</div>
<div class="tab-panel" data-tab-panel="tab-1">...</div>
<div class="tab-panel" data-tab-panel="tab-2">...</div>
```
- `data-tab` identifies the tab key; `data-tab-group` groups tabs in the same bar.
- `data-tab-panel` links a panel to its tab key.
- Active tab: `border-bottom: 2px solid var(--accent); color: var(--fg);`
- Inactive tab: `border-bottom: 2px solid transparent; color: var(--muted);`

### Semantic Icon Markers
- `data-skill="fireball"` — skill icon, expects `aria-label` for accessibility.
- `data-equip="sword"` — equipment icon, expects `aria-label` for accessibility.
- Both use `background-image: url(...)` with 24×24 or 32×32 PNG sprites.
- Rendered as square nodes with `aspect-ratio: 1; border-radius: var(--radius-sm);`

### Panel / Card
``div.panel`` → `background-color: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-lg); padding: var(--space-4);`

### Data Readout (Health Bar)
```
<div class="health-bar" aria-label="Health">
  <div class="health-bar-track" style="background-color: var(--border); border-radius: var(--radius-pill);">
    <div class="health-bar-fill" style="background-color: var(--success); width: 75%; border-radius: var(--radius-pill);"></div>
  </div>
</div>
```
- Fill width driven by `progress_binding_source` in BUI IR.
- Color switches to `var(--danger)` when value < 25%.

---

## 5 Layout Principles

- **Flexbox AND Grid** are both P0 Native. Use whichever suits the layout.
- `position: absolute` + `position: fixed` are P0 Native — standard for HUD overlays.
- No `float`, `inline-block`, or `inline` layouts. These are Forbidden.
- `display: block` is accepted as a simple stack/root fallback, but prefer explicit `flex` or `grid` for authored game UI.
- Default approach: `display: flex; flex-direction: column;` for vertical stacks, `display: flex; flex-direction: row;` for horizontal rows.
- Use `gap` instead of margins for spacing between flex/grid children.
- Use `padding` for internal spacing within a node.

### Layout Anti-Patterns
- Do not nest more than 4 levels deep. Flatten the DOM.
- Do not use `float: left/right` for layout (Forbidden).
- Do not use `display: inline/inline-block` (Forbidden).
- Avoid `display: block` for complex layout. It is supported as a simple fallback, but it carries less layout intent than `flex` or `grid`.
- Do not use `margin: auto` for centering — use `justify-content: center; align-items: center;` on the parent.

---

## 6 Depth & Elevation

Bevy UI has **no native `box-shadow`** rendering. Elevation is achieved via borders only.

| Level | Token | CSS |
|-------|-------|-----|
| Flat | `--elev-flat` | `none` — no border treatment |
| Ring | `--elev-ring` | `0 0 0 1px var(--border)` — 1px ring outline |
| Raised | `--elev-raised` | `0 0 0 1px var(--border)` — same ring; visual "raise" via background-color shift |

**Focus indication** uses `border-color` change, not shadow blur:
```css
:focus-visible {
  outline: none;
  border-color: var(--accent);
  /* OR for ring elevation: */
  /* box-shadow: 0 0 0 2px var(--accent); */  /* P1 HelperLayer — ring, no blur */
}
```

The `--focus-ring` token is `0 0 0 2px var(--accent)` — this is a **ring outline** (zero blur radius), which is P1 HelperLayer. For P0-only outputs, use `border-color: var(--accent)` instead.

---

## 7 CSS Property Tier Classification

This is the most critical section. Every CSS property that may appear in agent output is classified here.

### P0 Native (Exact Rendering)

**Layout:**
- `display`: `flex`, `grid`, `block`, `none`
- `position`: `relative`, `absolute`, `fixed`
- `top`, `right`, `bottom`, `left`
- `inset`
- `width`, `height`, `min-width`, `min-height`, `max-width`, `max-height`
- `aspect-ratio`
- `flex-direction`, `flex-wrap`, `flex-grow`, `flex-shrink`, `flex-basis`
- `justify-content`
- `align-items`, `align-self`, `align-content`
- `justify-items`, `justify-self`
- `place-items: center`
- `gap`, `row-gap`, `column-gap`
- `margin`, `padding`, `padding-inline`, `padding-block`
- `grid-template-columns`, `grid-template-rows`
- `overflow`, `overflow-x`, `overflow-y`
- `z-index`

**Visuals:**
- `background-color`
- `background-image: url()`
- `background-size`, `background-position`
- `border-radius`
- `border-width`
- `border-color`
- `border` shorthand
- `opacity`
- `transform`: `translate()`, `rotate()`, `scale()`

**Typography:**
- `color`
- `font-size`
- `font-family`
- `font-weight`
- `line-height`
- `letter-spacing`
- `text-align`
- `white-space`
- `text-shadow`

**State Selectors:**
- `:hover`
- `:active` / `:pressed`
- `:focus` / `:focus-visible`
- `:checked`
- `:disabled`

**State Attribute Aliases:**
- `aria-current="page"` / `aria-selected="true"` → initial `selected`
- `disabled="true"` / `aria-disabled="true"` → initial `disabled`
- `checked="true"` / `aria-checked="true"` → initial `checked`

Prefer explicit XML-safe boolean attributes (`disabled="true"`, `checked="true"`) over naked HTML boolean attributes (`disabled`, `checked`) because the current parser path is XML-like.

**State can override these P0 properties:**
- `background-color`
- `border-color`
- `color`
- `opacity`
- `transform`
- `filter: brightness()`, `filter: contrast()`, `filter: saturate()`

### P1 HelperLayer (Achieved via Extra Nodes)

| Desired CSS | HelperLayer Strategy |
|-------------|---------------------|
| `background: linear-gradient(...)` | Solid-color overlay bands — stack absolute-positioned children with decreasing `opacity` or `z-index` |
| `background: radial-gradient(...)` | Same overlay-band approach, or use a radial PNG as `background-image` |
| `box-shadow: Xpx Ypx Bpx Spx color` | Single shadow → emit a `box_shadow` helper node (background node behind target) + optionally `text_shadow` for text nodes |
| `border-top/left/right/bottom` (per-edge) | Emit absolute-positioned child nodes sized to the edge width/height |
| `filter: drop-shadow(...)` | `box_shadow` layer child + `text_shadow` on contained text nodes |
| `filter: brightness() / contrast() / saturate()` | Adjust the `color` channels of the node directly (P0 when used as state override) |

**P1 recommendation:** When a gradient or shadow is needed, **prefer a PNG image** over the helper-layer approach. PNGs are P0 via `background-image: url()`.

### P2 Approximation (Visible Imperfections)

| Desired CSS | Approximation Strategy |
|-------------|------------------------|
| `filter: blur()` | `box_shadow` with spread — produces a blurred rectangle behind the node; not true gaussian blur |
| `mask-image: linear-gradient(...)` | 20 fade layers — 20 overlapping absolute children with stepped opacity values |
| `clip-path: polygon(...)` | Contour/fill/accent child nodes positioned to approximate the clipped shape |
| `mix-blend-mode: multiply` | Darken color channels of the node and its children by the blend factor |

P2 strategies produce visually imperfect results. Use only when no P0 or P1 alternative exists.

### Unity UXML/USS Supported Surface

These concepts are valid in Unity UI Toolkit UXML/USS and are supported by the Bevy UI OD contract. The current Bevy parser/runtime may still trail some of them, but OD should keep the output aligned with this target instead of shrinking the syntax to a temporary implementation minimum.

| Unity UI Toolkit concept | Bevy UI target handling |
|--------------------------|-------------------------|
| UXML element tree (`VisualElement`, `Button`, `Label`, `Toggle`, `Slider`, `ScrollView`) | Author with conservative HTML tags (`div`, `section`, `button`) plus `data-*` semantics; preserve the UXML role in ids/classes/manifest metadata. |
| USS flex layout, position, spacing, size, overflow | P0 Native when expressed with the P0 layout properties above. |
| USS selectors and pseudo states (`:hover`, `:active`, `:focus`, `:checked`, `:disabled`) | P0 for supported state selectors and P0/P1 state property overrides. |
| USS 2D transforms (`translate`, `rotate`, `scale`) | P0 for supported transform functions. Treat unsupported transform functions as parser gaps. |
| USS `transition-*` | Supported target. OD may emit transition intent; mirror important timing/state intent into manifest/ECS metadata when useful. |
| USS `cursor` | Supported target. OD may emit cursor intent for interactive affordances. |
| Unity 9-slice image properties (`-unity-slice-*`) | Supported target. Also represent slicer / 9-slice metadata in `bevy-ui.assets.json` so the engine has a clear asset contract. |
| Unity font and text extensions (`-unity-font`, `-unity-font-definition`, `-unity-text-align`) | Supported target. Also map to font asset metadata when an external font file is required. |

### Forbidden (Must Never Appear)

These properties have **no renderer equivalent** and must never appear in output HTML:

- `float`, `clear`
- `display: inline`, `display: inline-block`
- `animation`, `@keyframes`
- `pointer-events`
- `content`
- `isolation`
- `-webkit-tap-highlight-color`

Any agent output containing a Forbidden property fails validation immediately.

---

## 8 Responsive Behavior

Bevy UI is **viewport-relative**. The renderer uses the game window size as the viewport.

- Use `vw`, `vh`, `vmin`, `vmax` units for viewport-relative sizing.
- `@media (min-width: ...)` and `@media (max-width: ...)` are P0 Native — the parser evaluates them at parse time.
- Use `--section-y-desktop`, `--section-y-tablet`, `--section-y-phone` tokens for responsive vertical padding.
- Use `--container-gutter-desktop`, `--container-gutter-tablet`, `--container-gutter-phone` tokens for responsive gutters.
- No `@media` queries for print or orientation — only `min-width` / `max-width`.

---

## 9 BUI IR Data Binding Contract

Each node in the parsed output can carry a `bindings` map that links CSS properties to ECS data sources.

### Binding Structure (per node)
```
{
  "target": "text.content",    // CSS property / attribute being bound
  "source": "player.health",   // ECS resource / component path
  "transform": null             // optional value transform function
}
```

### Binding Targets
| Target | Maps to | Type |
|--------|---------|------|
| `text.content` | Text content string | `String` |
| `background_color` | `background-color` | `Color` |
| `border_color` | `border-color` | `Color` |
| `text.color` | `color` (text) | `Color` |
| `image.tint` | `background-image` tint overlay | `Color` |
| `display` | `display` | `String` (`flex`/`grid`/`none`) |
| `visibility` | `visibility` | `String` (`visible`/`hidden`) |
| `border_width` | `border-width` | `f32` |
| `font_size` | `font-size` | `f32` |
| `text_bounds` | Width/Height of text container | `Vec2` |
| `justify` | `justify-content` | `JustifyText` |
| `line_height` | `line-height` | `f32` |
| `letter_spacing` | `letter-spacing` | `f32` |
| `text_shadow` | `text-shadow` | `TextShadow` |
| `ui_rotation` | `transform: rotate()` | `f32` |
| `ui_scale` | `transform: scale()` | `Vec2` |
| `ui_translation` | `transform: translate()` | `Vec2` |

### Semantic Binding Sources
| Source | Type | Use |
|--------|------|-----|
| `tab_binding_source` | `String` | Drives `data-tab` active state |
| `progress_binding_source` | `f32` (0–1) | Drives width/height of progress bar fills |
| `list_binding_source` | `Vec<Entity>` | Repeater: clones template node for each entity |

### State Model Defaults
When no binding is active, nodes use their static CSS values. Bindings override at runtime.

---

## 10 Interaction Contract

Bevy UI interaction follows the same separation of concerns as Unity UI Toolkit and web apps:

- HTML declares controls and stable action names.
- CSS / USS declares visual states.
- `BUI_ACTIONS` declares engine-readable prototype behavior.
- Bevy ECS systems own business logic through `BuiActionTriggered` or `BuiActionRegistry`.

### Control Classes

| UGUI mental model | Unity UI Toolkit target | Bevy / OD authoring contract |
|-------------------|-------------------------|-------------------------------|
| Button | `Button` + `ClickEvent` | `<button data-action="start-race">` or `role="button"` |
| Toggle | `Toggle` + `ChangeEvent<bool>` | `input type="checkbox"` / `role="switch"` + `data-binding` + `data-action-change` |
| InputField | `TextField` + `ChangeEvent<string>` | `input` / `textarea`, `data-binding="player.name"`, `data-action-change` |
| Slider | `Slider` + `ChangeEvent<float>` | `input type="range"` / `role="slider"` + `data-binding="volume"` + `data-action-change` |
| Scroller | `Scroller` | Scrollbar/thumb assets plus `data-action-scroll`; runtime controls target `ScrollPosition` |
| ScrollView | `ScrollView` | `data-scroll-view`, `overflow-x/y: auto|scroll`, `data-scroll-binding`, `data-action-scroll` |
| DropDown | `DropdownField` + value change | `select` / `role="combobox"` / `data-dropdown` + `option` items + `data-action-select` |
| Event Trigger | `RegisterCallback<TEvent>()` | `data-action-*`, `BUI_ACTIONS`, or ECS-side `BuiActionRegistry` |
| Event System | UI Toolkit dispatcher | Bevy input -> `BuiActionTriggered` / value events -> ECS systems |

### Event Names

Current supported action triggers:

- `press` / `pressed`
- `hover_enter` / `hovered`
- `hover_exit` / `unhovered`
- `value_changed` / `change`
- `submit`
- `focus`
- `blur`
- `scroll`
- `selection_changed` / `select`

Target event vocabulary for the parser/runtime:

- `pointer_enter`, `pointer_exit`
- `pointer_down`, `pointer_up`
- `click` / `press`
- `focus`, `blur`
- `submit`, `cancel`
- `scroll`
- `drag_start`, `drag`, `drag_end`
- `value_changed`
- `selection_changed`

OD should emit stable `data-action` names for business actions, not inline business logic. For example:

```html
<button id="play_button" data-action="start-race">PLAY</button>
```

Prototype behavior may be mirrored in `BUI_ACTIONS`:

```html
<script>
window.BUI_ACTIONS = {
  "actions": {
    "start-race": [
      { "op": "set-text", "node": "race_status_text_text_1", "value": "MATCHMAKING..." },
      { "op": "delay", "ms": 900 },
      { "op": "run-action", "target": "game.start_matchmaking" }
    ]
  }
};
</script>
```

Game business logic should be registered in Rust, like a UGUI listener or backend API route:

```rust
app.add_bui_action_handler("game.start_matchmaking", |world, event| {
    // update ECS resources, change scene, play sound, start matchmaking, etc.
});
```

Do not require Bevy to infer business behavior from arbitrary browser JavaScript. If a browser prototype uses DOM JS, also export the engine-readable `BUI_ACTIONS` contract.

## 11 Agent Prompt Guide

### Rules
1. **Use P0 properties freely.** They render exactly.
2. **Use P1 only when needed.** Prefer PNGs (`background-image: url()`) over helper-layer strategies for gradients and shadows.
3. **Use P2 only as last resort.** Accept visible imperfections.
4. **Never use Forbidden properties.** Validation will reject them.
5. **Add `data-action`** on every interactive node (buttons, toggles, links).
6. **Add `data-tab`** on tab navigation items.
7. **Add `data-skill` / `data-equip`** on icon nodes with `aria-label`.
8. **Flatten the DOM** — no more than 4 nesting levels.
9. **Use Unity USS `transition-*` for simple state transitions.** Do not use browser `animation` or `@keyframes`; timeline-heavy motion should stay ECS-driven.
10. **Ensure 4.5:1 contrast ratio** for all text (WCAG AA minimum).
11. **`transform`, `grid`, and `position: fixed`** are P0 Native — use them for HUD overlays and layouts.

### Output Format
- Single HTML file containing all markup and styles.
- All CSS custom properties defined in `:root` block at top of `<style>`.
- All content wrapped in `.bevy-ui-root` container.
- Comment header indicating any P1/P2 usage and why.
- Comment header indicating any Unity UXML/USS target features used, especially 9-slice, transitions, custom fonts, and state selectors.
