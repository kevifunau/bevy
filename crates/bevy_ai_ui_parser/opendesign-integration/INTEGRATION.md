# Bevy UI — Open Design Plugin: Source of Truth & Sync Guide

This directory is the **single source of truth** for the Bevy UI Open Design plugin. It merges the former `opendesign-integration` (contract SDK) and `bevy-ui-screen` (OD plugin) into one coherent package owned by the Bevy engine repo.

Open Design consumes these files via `sync_to_od.sh` (see below). **Do not edit OD copies directly** — any divergence should be synced back here first.

## Architecture Principle

```
HTML (source) → compile to IR JSON → runtime loads IR
```

The runtime never compiles HTML. `AiUiPlugin::from_path("*.ir.json")` is the primary API; `from_html_path` exists for dev prototyping only.

## Directory Structure

```
opendesign-integration/
├── open-design.json                          — OD plugin manifest (pipeline, 4 SubAgents, review gates)
├── SKILL.md                                  — Top-level plugin skill definition
├── AUTHORING.md                              — OD asset-first authoring guide
├── API.md                                    — Bevy runtime API reference (from_path primary)
├── INTEGRATION.md                            — This guide
├── references/                               — Shared reference docs for all sub-skills
│   ├── layouts.md
│   ├── prompt-recipes.md
│   └── slicing-profiles.md
├── assets/
│   └── template.html                         — HTML skeleton for generated projects
├── design-systems/bevy-ui/
│   ├── DESIGN.md                             — CSS/Unity USS capability model (715 lines, OD version)
│   ├── tokens.css                            — OD token declarations
│   ├── components.html                       — Component preview/catalog
│   ├── manifest.json                         — OD design-system manifest
│   ├── bevy_capability_contract.json         — Machine-readable capability contract (Grid/gap/z-index = P0)
│   ├── bevy_html_tag_map.json                — HTML tag → Bevy UI node mapping
│   └── bevy_ui_assets_schema.json            — Asset manifest JSON schema
├── design-templates/bevy-ui/
│   ├── AUTHORING.md                          — Asset-first authoring guide (template-specific)
│   ├── SKILL.md                              — Template catalog entry
│   ├── bevy-ui.assets.example.json           — Asset manifest example
│   ├── example.html                          — Template preview
│   └── assets/
│       └── template.html                     — HTML skeleton (duplicate of top-level, OD structure)
├── skills/
│   ├── bevy-ui-artist/                       — Art asset generation sub-skill
│   │   ├── SKILL.md
│   │   ├── agents/openai.yaml
│   │   ├── references/
│   │   │   ├── art-boundaries.md
│   │   │   └── art-preview-contract.md
│   │   ├── scripts/
│   │   │   ├── build_art_prompt.py
│   │   │   └── svg_to_png.py
│   ├── bevy-ui-programmer/                   — HTML/CSS assembly sub-skill
│   │   ├── SKILL.md
│   │   ├── agents/openai.yaml
│   │   ├── references/
│   │   │   ├── assembly-checklist.md
│   │   │   ├── structure-contract.md
│   │   │   ├── bevy_strict_lint.py           — Validation script (505 lines, checks against contract JSON)
│   │   │   ├── build_prompt_template.py
│   │   │   ├── build_structure_contract_stub.py
│   │   │   └── run_self_check.sh
│   ├── bevy-ui-ta/                           — Technical art / animation sub-skill
│   │   ├── SKILL.md
│   │   ├── agents/openai.yaml
│   │   ├── references/
│   │   │   ├── motion-boundaries.md
│   │   │   ├── state-checklist.md
│   ├── bevy-ux-planner/                      — UX planning sub-skill
│   │   ├── SKILL.md
│   │   ├── agents/openai.yaml
│   │   ├── references/
│   │   │   ├── scope-and-boundaries.md
│   │   │   ├── wireframe-deliverables.md
│   │   │   ├── wireframe_sections.py
│   └── bevy-ui-generator/                    — Generator SOP (bridge skill)
│       ├── SKILL.md
│       ├── references/
│       │   ├── AGENTS.md
│       │   ├── integration-spec.md
```

## What is NOT in This Plugin

| Excluded | Reason |
|----------|--------|
| `comfyui/` | UI artist domain, not engine contract scope — stays in OD repo |
| `v2-validation/` | Obsolete dev artifact |
| `API_CONTRACT_UPDATE_NOTES.md` | Temporary dev notes |
| `V2_ENHANCEMENT_SUMMARY.md` | Temporary dev notes |

## Sync to Open Design

Use `sync_to_od.sh` to copy from this directory into the OD repo:

```bash
# From the bevy repo root
./crates/bevy_ai_ui_parser/opendesign-integration/sync_to_od.sh /path/to/open-design

# Dry run (show what would change, no actual copy)
./crates/bevy_ai_ui_parser/opendesign-integration/sync_to_od.sh /path/to/open-design --dry-run
```

### Sync Mapping

| Source in bevy repo | Target in OD repo |
|---------------------|-------------------|
| `open-design.json` | `plugins/_official/examples/bevy-ui-screen/open-design.json` |
| `SKILL.md` | `plugins/_official/examples/bevy-ui-screen/SKILL.md` |
| `AUTHORING.md` | `plugins/_official/examples/bevy-ui-screen/AUTHORING.md` |
| `references/` | `plugins/_official/examples/bevy-ui-screen/references/` |
| `assets/` | `plugins/_official/examples/bevy-ui-screen/assets/` |
| `design-systems/bevy-ui/*` | `design-systems/bevy-ui/` (excluding DESIGN.md-only overlaps) |
| `design-templates/bevy-ui/*` | `design-templates/bevy-ui/` |
| `skills/bevy-ui-artist/` | `plugins/_official/examples/bevy-ui-screen/skills/bevy-ui-artist/` |
| `skills/bevy-ui-programmer/` | `plugins/_official/examples/bevy-ui-screen/skills/bevy-ui-programmer/` |
| `skills/bevy-ui-ta/` | `plugins/_official/examples/bevy-ui-screen/skills/bevy-ui-ta/` |
| `skills/bevy-ux-planner/` | `plugins/_official/examples/bevy-ui-screen/skills/bevy-ux-planner/` |
| `skills/bevy-ui-generator/SKILL.md` | `skills/bevy-ui-generator/SKILL.md` |
| `skills/bevy-ui-generator/references/` | `skills/bevy-ui-generator/references/` |

### Validation After Sync

```bash
cd /path/to/open-design
pnpm guard      # Validates tokens.css, manifest.json, contract compliance
pnpm typecheck  # Validates TypeScript types
```

Both must pass with zero violations.

## Source of Truth for Contracts

All contract content derives from the `bevy_ai_ui_parser` crate source:

| Content | Source File |
|---------|-------------|
| CSS property whitelist | `src/core/style/css_apply/declarations.rs` — `apply_opendesign_declaration` match |
| CSS effect fallback tiers | `src/core/style/css_effects/` — all fallback modules |
| Property support matrix | `src/core/style/css_metadata.rs` — `css_effect_fallback_registry()` |
| HTML attribute extraction | `src/core/opendesign/generic/tree.rs` — `generic_element_node` |
| Font mapping | `src/core/style/css_values/text.rs` — font-family → Bevy font asset |
| Data binding contract | `src/core/interaction/types.rs` + `src/core/interaction/bindings.rs` |
| Runtime action API | `src/core/interaction/action_registry.rs`, `src/core/interaction/keyboard.rs`, `src/core/runtime/declarative_actions.rs` |
| State model defaults | `src/core/model/ir.rs` — `BuiStateModel` |
| BUI IR contract | `schema/bui.schema.json` plus `src/core/model/ir.rs` |
| Capability contract JSON | `bevy_capability_contract.json` — checked by `bevy_strict_lint.py` |
| opus48 test cases | `examples/UiParserTest/opus48/` — 72 HTML + IR pairs |

When the parser changes, update the corresponding files here and re-sync to OD.

## Updating This Integration

1. Update the parser source file in `bevy_ai_ui_parser`.
2. Update `DESIGN.md` / `bevy_capability_contract.json` when the supported HTML/CSS surface changes.
3. Update `AUTHORING.md` / `bevy_ui_assets_schema.json` when asset/manifest semantics change.
4. Update sub-skill `SKILL.md` files when OD authoring instructions change.
5. Run `sync_to_od.sh` and validate in OD repo.
