# Bevy UI Slicing Profiles

Use a fixed slicer profile before writing ad hoc screenshot-cutting code.

## Implemented now

- `slg-login`
  - realistic medieval / western SLG login shell
  - outputs reusable background, panel frame, logo, and button skins
  - also emits `brand-spec.md` and `bevy-ui.assets.slice-draft.json`

## Reserved profile ids

- `slg-lobby`
- `fantasy-dialog`
- `inventory-panel`

## Rules

- Prefer the slicer whenever a reference screenshot clearly matches one of the above profiles.
- Keep all outputs inside the standard `Asset/` bucket structure.
- Use stable state names: `*_idle`, `*_hover`, `*_pressed`, `*_disabled`.
- Only fall back to custom scripting when no profile matches or the reference needs materially different cuts.
