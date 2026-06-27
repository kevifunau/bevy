---
name: bevy-ui-screen
description: |
  Multi-role Bevy UI production for HUDs, menus, dialogs, inventory panels,
  and loading overlays. Let UX plan first, then visual, optionally motion,
  and finally program roles hand off, and only assemble Bevy-safe HTML/CSS/JS
  static contracts plus Asset/ in the final program pass. Use when the brief
  asks for a "bevy ui", "game hud", "game menu", "inventory ui",
  "settings panel", or browser-to-Bevy parser validation.
triggers:
  - "bevy ui"
  - "game hud"
  - "game menu"
  - "inventory ui"
  - "settings panel"
  - "dialog ui"
  - "browser to bevy"
  - "游戏 UI"
  - "Bevy 界面"
od:
  mode: prototype
  platform: game-pc
  scenario: gaming
  featured: 20
  preview:
    type: html
    entry: example.html
  design_system:
    requires: true
    sections: [color, typography, layout, components]
  craft:
    requires: [anti-ai-slop, state-coverage]
    suggested: [animation-discipline]
  example_prompt: "Build a fantasy Bevy UI inventory screen through a role-based workflow: UX策划先给文本线框，UI美术再定静态视觉；如果需要，再让 UI TA补交互状态；最后 UI程序输出 index.html 和 Asset/，必要时附带可选 bevy-ui.assets.json。"
---

# Bevy UI Screen Skill

Treat this as a **game UI assembly task**, not a normal web styling task.

When side files are available, prefer these bundled references instead of
inventing structure from scratch:

- `assets/template.html`
- `references/layouts.md`
- `references/prompt-recipes.md`
- `references/slicing-profiles.md`
- `bevy-ui.assets.example.json`

Work through five internal passes even if they happen in one run:

Role-specific prompt contracts now also live under:

- `skills/bevy-ux-planner/`
- `skills/bevy-ui-artist/`
- `skills/bevy-ui-ta/`
- `skills/bevy-ui-programmer/`

Treat this root skill as the shared Bevy UI baseline. When a concrete role is
active, prefer that role skill's narrower instructions over the generic root
copy.

1. **PM pass**
   Treat this as the **UX planner pass**, not the asset pass and not the final
   implementation pass.
   Only do these jobs here:
   - break the gameplay request down into concrete UI fields and modules
   - clarify target platform, target resolution, and input mode
   - define information hierarchy, layout zones, and primary actions
   - reason about edge cases and state changes before visual work starts
   - output a PM brief, an ASCII/text wireframe, and a semantic layout handoff
   If the brief includes screenshots, sketches, or reference images, only use
   them to extract:
   - layout zones
   - information hierarchy
   - interaction hotspots
   - what must stay image-driven later vs what can remain simple layout
   Ask only the minimum questions needed to avoid generating the wrong layout.
   End this stage at PM brief review.
   Do not slice images, start preview services, run browser validation, perform
   brand-spec extraction, generate final HTML/CSS bundles, or write
   `bevy-ui.assets.json` here.
2. **DOM pass**
   Design a conservative, parser-friendly HTML structure. Prefer stable tags
   and shallow nesting. Put semantic hooks on nodes: `id`, `data-action`,
   `data-tab`, `data-tab-group`, `data-tab-panel`, `data-skill`, `data-equip`.
3. **Asset pass**
   Only after UX and structure intent are confirmed should the visual role
   inspect the current project's own resources before generating anything.
   Reuse game art from the current project's `assets/`, `build/`, or imported
   folders that are already part of this run. If visuals require painted
   chrome, buttons, frames, logos, or icons, treat them as image assets
   instead of CSS effects.
   If the reference image matches a known Bevy UI slicing profile, run the
   slicer first so the cuts stay stable across repeated runs.
   If the user gives both text and reference images, use the images for
   appearance and the text for semantics / missing states.
4. **Motion pass (optional)**
   Only enter this pass if the producer explicitly wants UI motion or state polish beyond the static art preview.
5. **CSS assembly pass**
   Use HTML/CSS only for layout, spacing, typography, and simple image-backed
   presentation. Avoid complex browser-only tricks as the primary look.
6. **Compiler pass**
   Ship a coherent Bevy UI bundle:
   - `index.html`
   - `Asset/` when images are referenced
   - optional `bevy-ui.assets.json` when tooling needs explicit asset metadata

## Hard rules

- The default workflow is role-based: UX clarifies and wires the layout first; final HTML output only belongs to the UI程序 pass.
- PM brief is a text-only gate and must stop before asset work starts.
- The UX / PM pass may only do requirement clarification, layout planning,
  ASCII wireframe output, and semantic structure handoff.
- The UX / PM pass must not inspect, reuse, import, or reference assets from
  historical projects or sibling projects unless the user explicitly provides
  those files in the current run.
- When checking for reusable assets, only consider the current project's own
  folders and files that are already part of this run's working context.
- Prefer real image assets over decorative CSS when the visual style depends on painted art.
- Use relative image paths like `Asset/buttons/confirm_idle.png`.
- Keep DOM depth low and selector usage simple.
- Keep the engine input valid with `index.html + Asset/`. Use optional `bevy-ui.assets.json` only for state textures, 9-slice data, atlases, and semantic icon mappings that tools need explicitly.
- Prefer the profile-based slicer for repeatable screenshot cuts. Only fall back
  to custom image scripts when no slicer profile matches.

## Suggested asset buckets

- `Asset/backgrounds/`
- `Asset/panels/`
- `Asset/buttons/`
- `Asset/icons/`
- `Asset/sliders/`
- `Asset/scrollbars/`
- `Asset/atlas/`

If the project has no assets yet, create the buckets that are actually used by
the screen and keep paths stable.

## Naming rules

- `*_idle`
- `*_hover`
- `*_pressed`
- `*_disabled`

## Validation checklist

Before finishing, verify:

1. `index.html` exists
2. `Asset/` exists when images are referenced
3. HTML image URLs resolve to real files
4. Stable ids, `data-action-*`, `data-binding`, and static `BUI_ACTIONS` cover runtime behavior
5. `bevy-ui.assets.json`, if present, is marked as optional tooling metadata and its paths resolve to real files
6. The main look comes from assets, not fake CSS painting
7. The structure should preview well in a browser and remain reasonable for Bevy parser ingestion

## Output shape

Aim for this project bundle:

```text
index.html
Asset/
  backgrounds/
  panels/
  buttons/
  icons/
  sliders/
  scrollbars/
  atlas/
bevy-ui.assets.json   # optional
```

Start from `assets/template.html`, lift sections from `references/layouts.md`,
and adapt the asset manifest from `bevy-ui.assets.example.json`.
