# Bevy UI — Open Design Integration Guide

This directory contains all files that the Bevy team prepares for integration into the [Open Design](https://github.com/nexu-io/open-design) repository. Open Design has catalog stubs (discovery entries) for Bevy UI, but the actual content files are authored here and copied into Open Design during integration.

## Directory Structure

```
opendesign-integration/
├── INTEGRATION.md                     — This guide
├── design-systems/bevy-ui/
│   ├── DESIGN.md                      — CSS property tier contract (417 lines)
│   ├── tokens.css                     — Full schema token declarations
│   └── manifest.json                  — v1 project manifest
├── skills/bevy-ui-generator/
│   └── SKILL.md                       — SOP + Pre-Export Checklist
├── design-templates/bevy-ui/
│   ├── SKILL.md                       — Template catalog entry
│   ├── example.html                   — Baked gallery preview (HUD + menu)
│   └── assets/
│       └── template.html              — Minimal skeleton with full :root tokens
```

## Integration Process

### Prerequisites

Open Design repository cloned locally at a known path. The integration target directories must exist (they are catalog stubs with empty `assets/` folders and `references/` docs).

### Step 1: Copy files into Open Design

```bash
# Set your Open Design repo path
OD_REPO=/path/to/open-design

# Design system files
cp opendesign-integration/design-systems/bevy-ui/DESIGN.md      $OD_REPO/design-systems/bevy-ui/DESIGN.md
cp opendesign-integration/design-systems/bevy-ui/tokens.css     $OD_REPO/design-systems/bevy-ui/tokens.css
cp opendesign-integration/design-systems/bevy-ui/manifest.json  $OD_REPO/design-systems/bevy-ui/manifest.json

# Skill file (replaces catalog stub)
cp opendesign-integration/skills/bevy-ui-generator/SKILL.md     $OD_REPO/skills/bevy-ui-generator/SKILL.md

# Template files (replaces catalog stub)
cp opendesign-integration/design-templates/bevy-ui/SKILL.md     $OD_REPO/design-templates/bevy-ui/SKILL.md
cp opendesign-integration/design-templates/bevy-ui/example.html $OD_REPO/design-templates/bevy-ui/example.html
cp opendesign-integration/design-templates/bevy-ui/assets/template.html $OD_REPO/design-templates/bevy-ui/assets/template.html
```

### Step 2: Validate in Open Design

```bash
cd $OD_REPO
pnpm guard      # Validates tokens.css, manifest.json, and all contract compliance
pnpm typecheck  # Validates TypeScript types
```

Both must pass with zero violations. Key checks:
- **Design system manifest check** — `manifest.json` conforms to v1 schema
- **A1 required tokens** — all 26 A1-identity + A1-structure tokens present in `tokens.css`
- **A2 required tokens** — all 26 A2 tokens present in `tokens.css`
- **B-slot required tokens** — all 4 B-slot tokens present in `tokens.css`
- **A2 defaults parity** — A2 fallback values in `tokens.css` match `_schema/defaults.css` byte-for-byte (or override with brand-specific values)
- **Craft references** — manifest.json `craft.applies/suggested/exemptions` reference only registered crafts from `craft/*.md`

### Step 3: Verify rendering

Open `design-templates/bevy-ui/example.html` in a browser. It should render a dark game UI with:
- Top HUD bar (health bar + ammo counter)
- Bottom skill button row (5 buttons)
- Centered settings menu panel (tabs + rows + close button)

All visual elements should use only P0 Native CSS properties per DESIGN.md.

## Source of Truth

All content in this integration directory is derived from the `bevy_ai_ui_parser` crate source code:

| Content | Source File |
|---------|-------------|
| CSS property whitelist | `src/core/style/css_apply/declarations.rs` — `apply_opendesign_declaration` match |
| CSS effect fallback tiers | `src/core/style/css_effects/` — all fallback modules |
| Property support matrix | `src/core/style/css_metadata.rs` — `css_effect_fallback_registry()` |
| HTML attribute extraction | `src/core/opendesign/generic/tree.rs` — `generic_element_node` |
| Font mapping | `src/core/style/css_values/text.rs` — font-family → Bevy font asset |
| Data binding contract | `src/core/interaction/types.rs` + `src/core/interaction/bindings.rs` |
| State model defaults | `src/core/model/ir.rs` — `BuiStateModel` |

When the Bevy parser changes (new CSS properties supported, new fallback tiers, new HTML attributes), update the corresponding files here and re-integrate into Open Design.

## Token Schema Compliance

The `tokens.css` `:root` block must declare every token from the Open Design shared schema. The schema is defined in `design-systems/_schema/tokens.schema.ts` in the Open Design repo.

### A1-identity tokens (brand-defining, must be authored)

```
--bg, --surface, --fg, --muted, --border, --accent, --font-display, --font-body
```

### A1-structure tokens (structural decisions, must be authored)

```
--text-xs, --text-sm, --text-base, --text-lg, --text-xl, --text-2xl, --text-3xl, --text-4xl
--leading-body, --leading-tight, --tracking-display
--section-y-desktop, --section-y-tablet, --section-y-phone
--container-max, --container-gutter-desktop, --container-gutter-tablet, --container-gutter-phone
```

### A2 tokens (required with fallback, declare explicitly)

```
--accent-on, --accent-hover, --accent-active
--success, --warn, --danger
--font-mono
--space-1 .. --space-8, --space-12
--radius-sm, --radius-md, --radius-lg, --radius-pill
--elev-flat, --elev-ring, --elev-raised
--focus-ring
--motion-fast, --motion-base, --ease-standard
```

### B-slot tokens (alias via var())

```
--surface-warm → var(--surface)
--fg-2 → var(--fg)
--meta → var(--muted)
--border-soft → var(--border)
```

## Bevy-Specific Token Overrides

Several A2 tokens override the defaults from `_schema/defaults.css`:

| Token | Default Value | Bevy Override | Reason |
|-------|---------------|---------------|--------|
| `--accent-on` | `#ffffff` | `#000000` | Orange accent is high-luminance; black text reads better |
| `--accent-hover` | `color-mix(in oklab, var(--accent), black 8%)` | `#e8790e` | Bevy parser may not support `color-mix()` in all contexts |
| `--accent-active` | `color-mix(in oklab, var(--accent), black 14%)` | `#d97006` | Same reason |
| `--success` | `#16a34a` | `#22c55e` | Tailwind green-500 matches Bevy's design language |
| `--warn` | `#eab308` | `#fbbf24` | Tailwind amber-400 |
| `--danger` | `#dc2626` | `#ef4444` | Tailwind red-500 |
| `--radius-sm` | `8px` | `6px` | Game UI uses tighter radii |
| `--radius-md` | `12px` | `8px` | Same |
| `--radius-lg` | `16px` | `12px` | Same |
| `--elev-raised` | `0 2px 8px color-mix(...)` | `0 0 0 1px var(--border)` | **No blur box-shadow** — Bevy forbids it |
| `--focus-ring` | `0 0 0 3px color-mix(...)` | `0 0 0 2px var(--accent)` | **No box-shadow blur** — border-based focus |
| `--container-max` | `1200px` | `1920px` | Game UI targets 1920×1080 |
| `--section-y-desktop` | `80px` | `48px` | Game UI uses tighter vertical rhythm |
| `--section-y-tablet` | `48px` | `32px` | Same |
| `--section-y-phone` | `32px` | `24px` | Same |
| `--text-3xl` | `48px` | `36px` | Game UI type scale is smaller |
| `--text-4xl` | `64px` | `48px` | Same |
| `--leading-body` | `1.5` | `1.4` | Game UI tighter leading |
| `--leading-tight` | `1.2` | `1.0` | Same |

## Craft Exemptions Explained

The manifest declares these craft exemptions:

| Craft | Reason for Exemption |
|-------|---------------------|
| `animation-discipline` | Bevy forbids CSS `transition`/`animation`. Motion is ECS-driven, not CSS-driven. All animation-discipline rules about transition durations/timing functions are moot. |
| `typography-hierarchy` | Game UI uses a compressed type scale (10px–48px) with fewer tiers than web (the full 8-step web hierarchy doesn't apply to HUD menus). |

## Updating This Integration

When the Bevy parser gains new capabilities:

1. Update the corresponding source file in `bevy_ai_ui_parser`
2. Update `DESIGN.md` §7 (CSS Property Tier Classification) to reflect new P0/P1/P2 properties
3. Update `tokens.css` if new design tokens are needed
4. Update `SKILL.md` Pre-Export Checklist if new forbidden/approximation rules apply
5. Re-copy files into Open Design and re-run `pnpm guard`

## Open Design Side: What to Expect

After integration, Open Design will have:

- `design-systems/bevy-ui/` with DESIGN.md + tokens.css + manifest.json (replaces empty stub)
- `skills/bevy-ui-generator/SKILL.md` with full SOP (replaces empty stub)
- `design-templates/bevy-ui/` with example.html + assets/template.html (replaces empty stub)
- `skills/bevy-ui-generator/references/` stays unchanged (Open Design's own docs)

The `references/AGENTS.md` and `references/integration-spec.md` in Open Design's `skills/bevy-ui-generator/` directory were the original TODO instructions for the Bevy Agent. After integration is complete, they can be archived or removed — their purpose was to tell the Bevy Agent what to prepare, and that preparation is now done.