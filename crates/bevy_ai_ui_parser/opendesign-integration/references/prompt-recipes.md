# Bevy UI Prompt Recipes

Use these as starting prompt shapes when generating from text, screenshots, or both.

## 1. Text-only screen brief

```text
Build a Bevy UI inventory screen with asset-first workflow.
Screen type: inventory.
Art direction: dark fantasy bronze and parchment.
Target resolution: 1920x1080.
First identify required asset categories.
Reuse any existing assets in the project.
Generate only missing assets.
Then output index.html and Asset/. Add bevy-ui.assets.json only as optional tooling metadata if needed.
Do not rely on complex CSS to fake game art.
```

## 2. Text + reference image

```text
Use the attached reference image as visual direction, but adapt it to a Bevy-safe game UI workflow.
Keep the overall composition, panel hierarchy, and icon mood from the image.
First list which parts should become real assets.
Reuse existing assets if available, and generate only missing ones.
Then assemble index.html and Asset/ with stable ids, relative asset paths, and static BUI_ACTIONS where runtime behavior is needed. Add bevy-ui.assets.json only as optional tooling metadata if needed.
```

## 3. Existing game assets first

```text
Build a Bevy UI settings screen using the assets already in this project.
Do not redesign the art style from scratch.
First inventory available buttons, panels, icons, and backgrounds.
Map them to the screen structure.
Only generate assets for missing states.
Then output the final Bevy UI bundle.
```

## 4. Browser vs Bevy validation prep

```text
Generate a Bevy UI HUD for parser validation.
Keep layout simple and parser-friendly.
Use real assets for the main visual identity.
Add stable ids for every component that needs manifest-backed states or slicers.
Make the browser preview and the Bevy parse result structurally comparable.
```
