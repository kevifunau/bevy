---
name: bevy-ui
od.mode: prototype
od.platform: desktop
od.upstream: https://github.com/kevifunau/bevy
od.preview:
  entry: example.html
od.design_system:
  requires: true
  name: bevy-ui
---

# Bevy UI Design Template

This template provides a starter skeleton for building Bevy UI HTML files.

## Template Files

| File | Purpose |
|------|---------|
| `assets/template.html` | Minimal HTML skeleton with `:root` token block, box-sizing reset, dark viewport, `.bevy-ui-root` flex container. Use this as the base for every new Bevy UI screen. |
| `example.html` | Self-contained example showing a game HUD (health bar, ammo counter, skill buttons) and a settings menu panel with tab navigation. Uses ONLY P0 Native CSS properties. Demonstrates `data-action`, `data-tab`, `data-skill` binding attributes. |

## How to Use

1. Copy `assets/template.html` as your starting point.
2. Add your UI structure inside `.bevy-ui-root`.
3. Reference `:root` custom properties for all colors, spacing, radii, fonts, and elevation.
4. Apply CSS properties strictly by tier (P0 Native → P1 HelperLayer → P2 Approximation → Forbidden never).
5. Add `data-action` on interactive nodes, `data-tab` on tab navigation, `data-skill`/`data-equip` on icon markers.
6. Run the self-critique checklist (8 items) before finalizing.
7. Export as a single HTML file.