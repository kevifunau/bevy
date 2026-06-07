# Bevy UI Design System

Derived from: Bevy BUI IR parser source code (`crates/bevy_ai_ui_parser`)
Purpose: Provide a authoritative CSS capability model so design agents produce UI that the Bevy ECS renderer can reproduce faithfully.

---

## 1 Visual Theme & Atmosphere

Bevy UI is a **game HUD and menu system**. It runs inside an ECS-driven rendering pipeline — not a browser. Every CSS property is classified into one of four tiers:

| Tier | Name | Meaning |
|------|------|---------|
| P0 | **Native** | The Bevy UI renderer reproduces the property exactly. Use freely. |
| P1 | **HelperLayer** | Achieved by overlaying extra helper nodes (solid bands, box_shadow children, absolute-positioned border strips). Prefer PNGs when possible. |
| P2 | **Approximation** | Rough visual mimicry through composite node strategies. Accept visible imperfections. Use only as last resort. |
| — | **Forbidden** | The renderer has no equivalent. Must never appear in output. |

**Dark mode only.** No light palette. No CSS animation timeline — all motion is ECS-driven.

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
- Default approach: `display: flex; flex-direction: column;` for vertical stacks, `display: flex; flex-direction: row;` for horizontal rows.
- Use `gap` instead of margins for spacing between flex/grid children.
- Use `padding` for internal spacing within a node.

### Layout Anti-Patterns
- Do not nest more than 4 levels deep. Flatten the DOM.
- Do not use `float: left/right` for layout (Forbidden).
- Do not use `display: inline/inline-block/block` (Forbidden — Bevy UI only renders `flex`, `grid`, and `none`).
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
- `display`: `flex`, `grid`, `none`
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

### Forbidden (Must Never Appear)

These properties have **no renderer equivalent** and must never appear in output HTML:

- `float`, `clear`
- `display: inline`, `display: inline-block`, `display: block`
- `transition`
- `animation`, `@keyframes`
- `cursor`
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

## 10 Agent Prompt Guide

### Rules
1. **Use P0 properties freely.** They render exactly.
2. **Use P1 only when needed.** Prefer PNGs (`background-image: url()`) over helper-layer strategies for gradients and shadows.
3. **Use P2 only as last resort.** Accept visible imperfections.
4. **Never use Forbidden properties.** Validation will reject them.
5. **Add `data-action`** on every interactive node (buttons, toggles, links).
6. **Add `data-tab`** on tab navigation items.
7. **Add `data-skill` / `data-equip`** on icon nodes with `aria-label`.
8. **Flatten the DOM** — no more than 4 nesting levels.
9. **No `transition`, `animation`, or `@keyframes`.** Motion is ECS-driven.
10. **Ensure 4.5:1 contrast ratio** for all text (WCAG AA minimum).
11. **`transform`, `grid`, and `position: fixed`** are P0 Native — use them for HUD overlays and layouts.

### Output Format
- Single HTML file containing all markup and styles.
- All CSS custom properties defined in `:root` block at top of `<style>`.
- All content wrapped in `.bevy-ui-root` container.
- Comment header indicating any P1/P2 usage and why.