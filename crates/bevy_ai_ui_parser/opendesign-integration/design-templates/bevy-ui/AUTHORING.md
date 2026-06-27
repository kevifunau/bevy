# Bevy UI Authoring Guide

Related design doc:
- `docs/bevy-ui-agent-design.zh-CN.md`

This template is for shipping **game UI**, not generic web UI.

The target output is always a small project:

```text
project-root/
  index.html
  assets/
  bevy-ui.assets.json   # optional OD asset manifest / parser hints
```

## Unity UXML/USS Target

Use Unity UI Toolkit UXML/USS as the compatibility north star because it is a production game-UI model with HTML/CSS-like structure.

OD should support the Unity UI Toolkit UXML/USS surface first. The Bevy engine/parser may trail some properties today, but the output contract should remain stable so engine work can catch up instead of forcing OD to generate a temporary reduced dialect.

- **Native Bevy HTML/CSS:** emit directly when `DESIGN.md` marks it P0.
- **Unity UXML/USS supported surface:** preserve Unity-supported properties and concepts such as 9-slice, button states, cursor intent, transition timing, custom fonts, and UXML-style control roles.
- **Asset/manifest/ECS contract:** HTML/CSS/JS remains the source of truth. Mirror game-specific data into `bevy-ui.assets.json`, `data-*`, or ECS metadata when the engine benefits from parser hints or validation data that CSS alone does not express cleanly.
- **Forbidden browser CSS:** avoid web-only behaviors that neither Unity UI Toolkit nor the Bevy parser should depend on, such as floats, inline layout, generated `content`, pointer-event hacks, and keyframe animations.

## What Open Design Should Do

Open Design should help you with two stages:

1. Asset stage
2. Assembly stage

### 1. Asset Stage

First decide which parts of the screen are real art assets:
- background
- panel frame
- button skins
- logo
- icons
- slider art
- scrollbar art
- atlas animation

Then handle them in this order:

1. Reuse existing project art from `assets/`, `build/`, or imported folders
2. Import local files into the Bevy UI project asset tree
3. Generate only the missing assets

Do not ask HTML/CSS to fake painted game UI if the real answer is “this needs an image”.

### 2. Assembly Stage

After the assets exist, assemble the screen with:
- `index.html` for structure, layout, text, and semantic hooks
- optional `bevy-ui.assets.json` for parser hints and validation metadata

Use HTML for:
- layout
- text
- image references required for browser preview (`<img src>` and `background-image`)
- `data-action`
- `data-tab`
- `data-tab-group`
- `data-tab-panel`
- `data-skill`
- `data-equip`

Use the manifest for:
- asset inventory and adoption status
- node-to-asset hints that duplicate or clarify HTML/CSS image usage
- 9-slice setup
- button state textures
- atlas metadata
- semantic icon mapping
- expected bounds / layout checkpoints for browser-vs-engine validation
- transition/state timing intent when the final runtime needs ECS-driven motion
- Unity UXML/USS target notes that Bevy Engine may need to implement next

Do not treat `bevy-ui.assets.json` as Bevy's generated BUI/IR. Generated engine
IR should use a separate name such as `*.bui.json` or `*.ir.json`.

## Bevy-Safe HTML Tag Habit

Prefer a conservative tag set for Bevy authoring:

- `div`
- `section`
- `button`

Use `main`, `aside`, `article`, and other semantic tags only if you have already verified that your current parser path accepts them cleanly.

When in doubt, choose stable structure over richer semantics.

## Recommended Working Loop

### Loop A: Start from existing game art

1. Create a Bevy UI project
2. Import your UI resource pack into `assets/`
3. Ask Open Design to assemble a screen using those assets
4. Export `index.html` + `assets/` and add `bevy-ui.assets.json` when game art metadata is useful
5. Run the Bevy parser example to compare browser vs Bevy

### Loop B: Missing art, let OD fill gaps

1. Create a Bevy UI project
2. Describe the target screen
3. Ask Open Design to list required asset categories first
4. Reuse/import any assets you already have
5. Ask Open Design to generate only the missing items
6. Re-assemble without regenerating all assets

## Naming Rules

Use stable file names:

- `*_idle`
- `*_hover`
- `*_pressed`
- `*_disabled`

Examples:

- `assets/buttons/confirm_idle.png`
- `assets/buttons/confirm_hover.png`
- `assets/buttons/confirm_pressed.png`
- `assets/panels/settings_panel_bg.png`

## Folder Rules

Keep assets in fixed buckets:

- `assets/backgrounds/`
- `assets/panels/`
- `assets/buttons/`
- `assets/icons/`
- `assets/sliders/`
- `assets/scrollbars/`
- `assets/atlas/`

All HTML references must use relative paths like:

```html
background-image: url("./assets/buttons/confirm_idle.png");
```

## Good Prompt Pattern

Use prompts shaped like this:

```text
Build a Bevy UI settings screen with asset-first workflow.
First identify required assets.
Reuse any existing assets in the project.
Generate only missing assets.
Then output index.html, assets/, and a bevy-ui.assets.json sidecar with parser hints if the screen uses game art assets.
Do not rely on complex CSS to fake game art.
```

## What “Good” Looks Like

A good Bevy UI export has these properties:

- browser preview looks coherent
- Bevy runtime can resolve every image path
- main visual identity comes from real images
- HTML still stays clean and semantic
- manifest only carries the extra game-UI contract

## Fast Validation Checklist

Before handing the result to Bevy, check:

1. `index.html` exists
2. HTML image URLs point to real files
3. if `bevy-ui.assets.json` exists, every manifest path points to a real file
4. buttons have stable ids or simple class selectors
5. no critical visual dependency is hiding only in manifest data
6. no critical visual dependency is hiding in fancy CSS

## Practical Advice

If the screen still feels “too web-like”, the usual root cause is not parser fidelity.
It is usually that the screen still lacks real painted assets.

In that situation, do not keep expanding CSS complexity.
Add or improve art assets first, then re-assemble.
