# Official UI Backlog Status

This file tracks which official `examples/ui` cases have already been processed into the BUI parser workstream and what the current outcome is.

## Supported via `uiParse_*`

- `examples/ui/images/image_node.rs` -> `uiParse_image_node`
- `examples/ui/images/image_node_resizing.rs` -> `uiParse_image_node_resizing`
- `examples/ui/images/ui_texture_atlas.rs` -> `uiParse_ui_texture_atlas`
- `examples/ui/images/ui_texture_atlas_slice.rs` -> `uiParse_ui_texture_atlas_slice`
- `examples/ui/images/ui_texture_slice.rs` -> `uiParse_ui_texture_slice`
- `examples/ui/images/ui_texture_slice_flip_and_tile.rs` -> `uiParse_ui_texture_slice_flip_and_tile`
- `examples/ui/layout/anchor_layout.rs` -> `uiParse_anchor_layout`
- `examples/ui/layout/display_and_visibility.rs` -> `uiParse_display_and_visibility`
- `examples/ui/layout/fixed_node.rs` -> `uiParse_fixed_node`
- `examples/ui/layout/flex_layout.rs` -> `uiParse_flex_layout`
- `examples/ui/layout/grid.rs` -> `uiParse_grid`
- `examples/ui/layout/size_constraints.rs` -> `uiParse_size_constraints`
- `examples/ui/layout/z_index.rs` -> `uiParse_z_index`
- `examples/ui/styling/transparency_ui.rs` -> `uiParse_transparency_ui`
- `examples/ui/window_fallthrough.rs` -> `uiParse_window_fallthrough`
- `examples/ui/relative_cursor_position.rs` -> `uiParse_relative_cursor_position`
- `examples/ui/ui_scaling.rs` -> `uiParse_ui_scaling`
- `examples/ui/ui_transform.rs` -> `uiParse_ui_transform`
- `examples/ui/widgets/button.rs` -> `uiParse_button`
- `examples/ui/ui_target_camera.rs` -> `uiParse_ui_target_camera`
- `examples/ui/scroll_and_overflow/overflow.rs` -> `uiParse_overflow`
- `examples/ui/scroll_and_overflow/overflow_clip_margin.rs` -> `uiParse_overflow_clip_margin`
- `examples/ui/widgets/tab_navigation.rs` -> `uiParse_tab_navigation`
- `examples/ui/text/text_input.rs` -> `uiParse_text_input`

## Explicit Unsupported Boundaries

- `examples/ui/layout/ghost_nodes.rs`
  Outcome: unsupported in the strict BUI contract.
  Reason: the source example depends on Bevy `ghost_nodes`, which is explicitly experimental and requires the `ghost_nodes` feature flag. The current BUI contract only exposes stable, non-experimental UI capabilities.
  Validation artifact: `unsupported/ghost_nodes.experimental.json`


- `examples/ui/styling/borders.rs`
  Outcome: partially unsupported in the strict BUI contract.
  Reason: the source example depends on several visual capabilities that the current contract does not expose, including per-edge `BorderColor`, per-corner `BorderRadius`, `Outline`, and `justify_self`.
  Validation artifact: `unsupported/borders.per_edge.json`

- `examples/ui/ui_material.rs`
  Outcome: partially unsupported in the strict BUI contract.
  Reason: the official example requires real `UiMaterial` binding and animated material uniforms. The current BUI contract only accepts `visuals.material_shader` as a marker/logging field and does not bind a live `UiMaterial` asset or expose material uniforms/textures.
  Validation artifact: `unsupported/ui_material.binding.json`

- `examples/ui/text/editable_text_filter.rs`
  Outcome: partially unsupported in the strict BUI contract.
  Reason: the official example depends on `EditableTextFilter` and `AutoFocus`. The current BUI contract does not expose text-input character filters or autofocus behavior.
  Validation artifact: `unsupported/editable_text_filter.json`
