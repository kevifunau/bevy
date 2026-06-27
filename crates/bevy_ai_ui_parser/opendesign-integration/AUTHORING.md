# Bevy UI Authoring Guide

This plugin is for **game UI**, not generic web UI.

The target output is a compact project bundle:

```text
project-root/
  index.html
  bevy-ui.assets.json   # optional OD asset manifest / parser hints
  Asset/
```

## Workflow

1. Identify required asset categories first.
2. Reuse or import existing art before generating new images.
3. If a supplied screenshot matches a known profile, run the Bevy UI slicer before ad hoc cutout work.
4. Generate only the missing assets.
5. Assemble the screen with Bevy-safe HTML/CSS/JS as the source of truth.
6. Emit `bevy-ui.assets.json` only as an optional OD-authored sidecar when states, slicers, atlases, semantic image bindings, or validation hints are useful to tools.

## Stage discipline

- `pm-brief`: discovery, brief writing, and handoff contract only.
- `pm-brief-review`: explicit reviewer approval or revision.
- `wireframe` / `structure-preview`: layout and semantic structure only.
- `asset-direction` / `asset-final`: slicing, candidate asset work, and manifest draft.
- `assembly`: local preview and final validation.

Do not run screenshot slicing, preview servers, Playwright, or browser validation during `pm-brief`.
Do not inspect or reuse assets from past projects during `pm-brief`; the current
project may only consume assets that already exist inside its own workspace or
were explicitly provided in the current request.

## Recommended asset categories

- background
- panel frame
- button state images
- logo
- icons
- slider art
- scrollbar art
- atlas animation sheet

## Folder rules

- `Asset/backgrounds/`
- `Asset/panels/`
- `Asset/buttons/`
- `Asset/icons/`
- `Asset/sliders/`
- `Asset/scrollbars/`
- `Asset/atlas/`

## Manifest responsibility

Use HTML/CSS/JS for structure, layout, text, prototype behavior, image URLs, and semantic hooks.

Use optional `bevy-ui.assets.json` for:

- asset inventory and adoption status
- node-to-asset parser hints that clarify HTML/CSS image usage
- 9-slice setup
- button state textures
- atlas metadata
- semantic icon mapping
- expected bounds / layout checkpoints for browser-vs-engine validation

Do not treat `bevy-ui.assets.json` as Bevy's generated BUI/IR. Generated engine
IR should be written separately by the engine/parser.

## Prompt pattern

```text
Build a Bevy UI settings screen with asset-first workflow.
First identify required assets.
Reuse any existing assets in the project.
If the reference image matches a known Bevy UI slicer profile, run the slicer first.
Generate only missing assets.
Then output index.html and Asset/. Add a bevy-ui.assets.json sidecar only when parser/tooling hints are useful.
Do not rely on complex CSS to fake game art.
```
