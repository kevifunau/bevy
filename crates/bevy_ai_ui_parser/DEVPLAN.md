# bevy_ai_ui_parser Development Plan

> This document tracks the current state, completed work, and remaining tasks for the
> bevy_ai_ui_parser crate. A new development session should read this file first to
> understand context and pick up where the previous session left off.

---

## Architecture Overview

The crate implements a two-phase UI pipeline:

**Phase 1 (Compile)**: HTML → BUI JSON IR (`.ir.json`) + SVG→PNG rasterization
**Phase 2 (Runtime)**: BUI JSON IR → Bevy UI ECS tree (spawn, interaction, state)
**Phase 3 (Editor)**: Runtime editor for live UI editing (hover, drag, delete, save)

The IR-first strategy means `.ir.json` is the source of truth. HTML is a one-way import
that compiles to IR; the runtime loads IR JSON and spawns Bevy UI. The editor modifies
the IR in-memory and can save it back to `.ir.json`.

---

## Current State (as of 2026-06-23)

### Test Case Rebuild — COMPLETE

Replaced all old hand-crafted test examples with opus48-based cases:

- **Deleted**: `opendesignTest/` (9 old cases), `uiParse_TestSet/` (24 reverse-engineered Bevy examples),
  `unsupported/`, `GOAL.md`, `OFFICIAL_UI_BACKLOG_STATUS.md`, `auto_screenshot.rs`, `compile_opendesign_ir.rs`
- **Test fixtures moved**: 7 HTML/IR/manifest files referenced by `include_str!` in the crate's
  test suite moved from `examples/UiParserTest/opendesignTest/` into `crates/bevy_ai_ui_parser/src/tests/fixtures/`.
  Updated `shared.rs` paths. All 152 lib tests still pass.
- **New opus48 examples**: 72 examples mirroring the source structure at
  `examples/UiParserTest/opus48/{Dev,interactive}/` — 16 cases (Dev 8 + interactive 8), each with
  multiple HTML pages. Every page has a compiled `.ir.json` and a runner `.rs`.
- **API cleanup** (`examples/UiParserTest/common.rs`): Replaced 8 confusing `run_with_*` variants
  (including `run_with_html_without_button_feedback`) with 2 clean functions:
  - `run(ir_rel_path)` — load pre-compiled IR JSON, render with full interaction (button feedback always on)
  - `run_with_editor(ir_rel_path)` — load IR JSON with editor mode (F8 toggle)
  Removed `button_feedback_enabled` parameter (always on). Removed `bui_path()` nested-directory
  lookup (simplified to `ui_test_path()` direct resolution). Removed `HERO_GAME_UI`/`BEVY_UI_LOGIN_CARTOON`
  screenshot profiles (deleted examples).
- **Architecture**: HTML is source code; `compile_opendesign_html` CLI compiles it to IR JSON (build step).
  Runtime examples load IR JSON via `AiUiPlugin::from_path()` — no HTML compilation at runtime.
- **Cargo.toml**: Removed 40+ old/broken example registrations. Added 72 new opus48 example registrations.
  Fixed 6 previously broken references (missing files/directories).

### Phase 1: SVG→PNG Pipeline — COMPLETE

The SVG rasterization pipeline is fully implemented and verified:

- **Two-pass design**: Pass 1 (tree traversal) accumulates `SvgAssetEntry` vec and creates
  image nodes with `texture_path = "assets/png/{key}.png"`; Pass 2 (asset pipeline)
  rasterizes all SVGs via resvg/tiny-skia and writes PNGs to disk.
- **Dependencies**: `resvg = "0.44"`, `usvg = "0.44"`, `tiny-skia = "0.11"` (verified
  compatible triplet, all pure Rust).
- **Key files**:
  - `src/core/opendesign/svg/extract.rs` — `extract_svg_markup()`, `svg_asset_key()`,
    `svg_viewbox_size()`, `svg_render_scale()`, `SvgAssetEntry` struct
  - `src/core/opendesign/svg/rasterize.rs` — `rasterize_svg_to_png()`,
    `write_svg_png_asset()`, `rasterize_svg_assets()`
  - `src/core/opendesign/svg/render.rs` — `is_svg_tag()`, `svg_image_node()` (creates
    `BuiNodeType::Image` with PNG path)
  - `src/core/opendesign/svg/semantic.rs` — DELETED (was semantic SVG analysis)
  - `src/core/opendesign/svg/shape.rs` — DELETED (was custom SVG shape handling)
- **Unicode fallback removed**: No `svg:fallback` text markers; all SVGs become image nodes.
- **Tests**: `src/tests/css_effects/svg_rasterize.rs` — 8 tests covering key generation,
  viewBox parsing, render scale, PNG rasterization, SVG markup extraction.
- **End-to-end verified**: `compile_opendesign_html hero-game-ui.html` → 15 PNG files
  (valid RGBA, 128×128) + IR JSON with 16 image nodes, zero `svg:fallback` markers.

### Phase 1: HTML Root Container Recognition — COMPLETE

All 72 opus48 HTML files now compile (was 69/72, then fixed 3 edge cases):

- **Recognized root containers** (string-level + DOM-level):
  - `<div class="overlay">`, `<main class="game-stage">`, `<div/main class="bevy-ui-root">`
  - `<main/section/div class="stage">`, `<div class="page">`
- **`find_element_with_class()`**: Exact/prefix class matching — prevents `<div class="stage-tag">`
  from incorrectly matching `<div class="stage">`.
- **`find_matching_div_close()`**: UTF-8-aware div depth tracker — properly handles
  self-closing `<div/>`, nested `<div>` tags, and multi-byte UTF-8 characters (▲, etc).
  Replaces the previous `rfind("</div>")` which over-matched.
- **`normalize_html_entities_for_xml()`**: Rewritten to escape bare `&` as `&amp;`
  (e.g. `"& vertical"` → `"&amp; vertical"`), while preserving known HTML entities
  (`&nbsp;`, `&ensp;`, `&emsp;`, `&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`).

### Phase 1: Auto-save `.ir.json` — COMPLETE

When `AiUiPlugin::from_html_path()` is used, the plugin's `load_bui_document()` now
automatically writes the compiled IR JSON to `{html_path}.ir.json` alongside the HTML file.

- `plugin.rs:441-446` — `serde_json::to_string_pretty` + `fs::write`
- `BuiSourcePaths` stores both `ir_json_path` and `html_path` for editor save-back.

### Phase 1: IR Fixture Regeneration — COMPLETE

- `hero-game-ui.ir.json` regenerated with image nodes (replaces old text-fallback fixture).
- `hero_game_ui_html_and_ir_entry_paths_produce_identical_bui_documents` test restored to
  full `serde_json::to_value` equality check (152 tests pass).

### Test Results

All 152 lib tests pass. 3 compiler warnings (expected dead code — unused model fields,
unused `opendesign_html_to_bui_document` convenience function).

---

## Remaining Work

### P1-A: Runtime Bevy Visual Verification Example (HIGH)

**Goal**: Create a runnable Bevy example that loads hero-game-ui HTML + PNG images and
renders them in a window, so we can visually verify the SVG→PNG pipeline works at runtime.

**Why**: The compile pipeline generates PNGs and IR JSON, but we haven't verified that the
runtime Bevy UI actually loads and renders the PNG images correctly. The `ImageNode`
spawned from `BuiNodeType::Image` needs `asset_server.load("assets/png/{key}.png")`
which requires the PNGs to be in Bevy's asset directory.

**Steps**:
1. Create `examples/UiParserTest/hero_game_ui_render.rs` — a minimal Bevy app that:
   - Uses `AiUiPlugin::from_html_path("hero-game-ui.html")`
   - Has a `Camera2d` for UI rendering
   - Runs and shows the hero game UI with SVG-rendered PNG images
2. Verify asset paths are relative to the HTML file's parent directory
   (`asset_server.load` resolves relative to `assets/` folder in Bevy)
3. May need to adjust PNG path generation so `texture_path` uses Bevy-compatible
   asset paths (currently `"assets/png/{key}.png"` which should work if the assets
   folder is next to the HTML file)

**Key concern**: Bevy's `AssetServer` loads assets relative to the `assets/` directory
at the project root, not relative to the HTML file. We may need to:
- Either copy/symlink PNGs to the Bevy project's `assets/` directory
- Or adjust the `texture_path` to use absolute or project-relative paths
- Or register a custom asset provider that resolves paths relative to the HTML file

**Files to modify**:
- `examples/UiParserTest/hero_game_ui_render.rs` (NEW)
- Possibly `src/core/runtime/image.rs` or `src/core/opendesign/svg/rasterize.rs`
  (path resolution)
- `Cargo.toml` (register new example)

### P1-B: BuiSource::HtmlInline PNG Limitation (MEDIUM)

**Current limitation**: `BuiSource::HtmlInline` passes `base_dir: None`, so SVGs are
collected but PNGs are NOT written to disk. At runtime, `asset_server.load()` will fail
to find the PNG files.

**Options**:
1. Accept the limitation — `HtmlInline` only works for HTML without SVGs
2. Write PNGs to a temp directory and pass that as `base_dir`
3. Embed PNG bytes directly into IR JSON (inefficient but self-contained)

**Recommendation**: Option 1 for now, with a clear error message when SVGs are encountered
in `HtmlInline` mode. Document the limitation.

### P1-C: Bevy Asset Path Resolution (MEDIUM)

**Problem**: PNG paths in IR JSON are `"assets/png/{key}.png"` — relative to the HTML
file directory. But Bevy's `AssetServer` resolves paths relative to the project's
`assets/` root directory. If the HTML file is at `examples/UiParserTest/opendesignTest/
hero_game_ui/hero-game-ui.html`, the PNGs are at `hero_game_ui/assets/png/`, but Bevy
expects them at the project-level `assets/` directory.

**Options**:
1. Make `texture_path` relative to Bevy's asset root (requires knowing the project root)
2. Create a custom `AssetProvider` that resolves paths relative to the HTML directory
3. Copy generated PNGs into Bevy's `assets/` directory during build
4. Change `rasterize_svg_assets()` to write PNGs relative to a configurable output dir

**Recommendation**: Start with option 3 (manual copy/symlink for the example) and
later implement option 1 or 2 for production use.

### P2: UI Editor Enhancement (HIGH — Phase 3 architecture)

**Goal**: Build a Unity UI Toolkit-style editor with full editing capabilities.

**Current editor state** (Phase 1 editor — basic):
- Hover highlight (yellow border)
- Drag-to-reposition (position changes only)
- Delete nodes (with dialog)
- Save/discard edits (writes back `.ir.json`)
- Toggle editor mode (keyboard shortcut)
- The editor only works with `BuiSource::Path` (IR JSON file)

**What needs to be added for Phase 3**:
1. **Inspector panel** (bevy_egui): Show selected node's properties (layout, styles,
   content, markers, node_type). Edit properties inline.
2. **Hierarchy panel** (bevy_egui): Tree view of all BUI nodes. Click to select.
   Drag to reorder.
3. **Canvas rendering** (Bevy UI): The edited document is rendered as Bevy UI in the
   main viewport. Selection highlight, hover feedback.
4. **Undo/redo** (new module): Stack of `BuiEdit` actions. Undo reverses the last edit;
   redo re-applies it. Persist across save/discard.
5. **Create nodes** (new module): Add new child nodes (Container, Text, Image, Button).
   Template-based insertion with default styles.
6. **Resize nodes** (new module): Drag handles for width/height resize.
7. **Multi-select** (new module): Select multiple nodes for batch operations.
8. **State preview** (bevy_egui panel): Toggle visual states (hover, active, focus,
   checked) on selected nodes to preview state-dependent styles.

**Dependency**: `bevy_egui` (git submodule at `crates/bevy_egui`, v0.40.0) and
`bevy_inspector_egui` (git submodule at `crates/bevy_inspector_egui`, with
`CreateTypeData` fix). Both already set up and `cargo check` passes.

**Key files**:
- `src/core/editor/` — 14 existing modules (state, hover, drag, delete, save, etc.)
- `crates/bevy_egui/` — git submodule
- `crates/bevy_inspector_egui/` — git submodule

### P2-A: Editor Inspector Panel (HIGH)

**Steps**:
1. Add `bevy_egui` and `bevy_inspector_egui` as optional dependencies in `Cargo.toml`
   (feature-gated: `editor = ["bevy_egui", "bevy_inspector_egui"]`)
2. Create `src/core/editor/inspector.rs` — egui panel that reads selected node's
   `BuiNode` data and displays editable fields
3. Wire into `AiUiPlugin::from_path_with_editor()` — add egui systems when editor
   feature is enabled
4. Selection state: `BuiEditorState.selected_node_id` → lookup in `BuiDocumentResource`
   → display properties

### P2-B: Editor Hierarchy Panel (HIGH)

**Steps**:
1. Create `src/core/editor/hierarchy.rs` — egui tree view of `BuiDocument.root`
2. Click node → set `BuiEditorState.selected_node_id`
3. Drag node → reorder children in parent (emit `BuiEdit::Reorder`)
4. Expand/collapse subtrees

### P2-C: Editor Undo/Redo (MEDIUM)

**Steps**:
1. Create `src/core/editor/undo.rs` — `UndoStack` resource
2. Every `BuiEdit` is pushed to the undo stack on creation
3. Ctrl+Z pops from undo stack and applies inverse edit
4. Ctrl+Y pops from redo stack and re-applies edit
5. Save/discard clears both stacks

---

## Known Issues & Technical Debt

### 1. Bevy Asset Path vs File-System Path Mismatch

PNG texture paths in IR JSON are file-system relative (`assets/png/{key}.png`),
but Bevy's `AssetServer` resolves relative to the project's `assets/` root.
This mismatch means runtime rendering of SVG-derived images may fail unless
PNGs are placed in the correct Bevy asset directory.

**Impact**: P1-A (visual verification) is blocked until this is resolved.
**Location**: `src/core/opendesign/svg/rasterize.rs:write_svg_png_asset()`,
`src/core/runtime/image.rs:build_image_node()`.

### 2. HtmlInline SVG Limitation

`BuiSource::HtmlInline` passes `base_dir: None`, so SVGs are collected into
`SvgAssetEntry` vec but PNGs are never written to disk. Runtime will fail to
load images. No warning or error is emitted for this case.

**Location**: `src/core/opendesign/html.rs:opendesign_html_to_bui_document_with_manifest()`
line ~450 (`BuiSource::HtmlInline` path).

### 3. Editor Only Works with IR JSON Files

`source_supports_editor()` returns `true` only for `BuiSource::Path`. This means
`from_html_path_with_editor()` silently disables the editor. The auto-saved
`.ir.json` file should make this work, but the editor initialization doesn't
currently reload from the saved `.ir.json`.

**Location**: `src/core/runtime/plugin.rs:280-282`.

### 4. opendesign_html_to_bui_document() Dead Code Warning

The convenience function `opendesign_html_to_bui_document()` (without manifest)
is never used. It's a public API entry point but no code path calls it.

**Location**: `src/core/opendesign/html.rs:231`.

### 5. Manifest Dead Code Warnings

`OpenDesignAssetEntry.kind` and `OpenDesignAssetEntry.usage` fields are never read.
`OpenDesignComponentEntry.id` and `OpenDesignComponentEntry.kind` are never read.
These are parsed from manifest JSON but not used in compilation.

**Location**: `src/core/opendesign/manifest.rs:32-43`.

---

## Critical Implementation Notes

### resvg 0.44 API

```rust
// resvg::render requires &mut PixmapMut, not &mut Pixmap
// Use pixmap.as_mut() for the conversion
resvg::render(&tree, tiny_skia::Transform::identity(), &mut pixmap.as_mut());

// usvg::Tree::from_str requires xmlns="http://www.w3.org/2000/svg" on root <svg>
// roxmltree parses xmlns as namespace (not attribute), so manual injection is needed
// See extract_svg_markup() → serialize_node_recursive() for the xmlns injection logic
```

### find_matching_div_close()

```rust
// UTF-8-aware div depth tracker. Uses byte-level scanning for "<div" and "</div>"
// patterns, but advances by char.len_utf8() for non-matching characters to avoid
// slicing at non-character boundaries (e.g. multi-byte '▲' at byte 4273..4276)
// Self-closing <div/> (rare in HTML but possible) is handled by checking tag.endswith("/>")
```

### find_element_with_class()

```rust
// Searches for exact class match or class prefix match
// "<div class=\"page\">" matches find_element_with_class(html, "div", "page")
// "<div class=\"page other\">" also matches (prefix with trailing space)
// "<div class=\"stage-tag\">" does NOT match find_element_with_class(html, "div", "stage")
// This prevents false matches that broke pixel_jump/hud.html compilation
```

### normalize_html_entities_for_xml()

```rust
// Character-by-char processing that:
// - Replaces &nbsp; → &#160;, &ensp; → &#8194;, &emsp; → &#8195;
// - Preserves &amp;, &lt;, &gt;, &quot;, &apos; as-is
// - Preserves &#[number]; as-is (numeric character references)
// - Escapes bare & → &amp; (e.g. "Controls & Sensitivity" → "Controls &amp; Sensitivity")
// This ensures roxmltree can parse the fragment as valid XML
```

### SVG→PNG Pipeline Flow

```
HTML input
  → extract_opendesign_fragment() — string-level root container extraction
  → normalize_html_entities_for_xml() — entity normalization for XML parsing
  → wrap in <bui_root>, parse with roxmltree
  → find_opendesign_root_nodes() — DOM-level root container search
  → opendesign_html_to_generic_bui_document() — tree traversal
    → generic_append_children() — recursive DOM walk
      → For SVG elements: extract_svg_markup() → svg_asset_key() → svg_image_node()
        → Accumulates SvgAssetEntry { key, svg_markup, width, height }
      → For non-SVG elements: normal BuiNode creation
  → rasterize_svg_assets() — writes PNGs to {base_dir}/assets/png/{key}.png
    → For each SvgAssetEntry:
      → usvg::Tree::from_str(svg_markup) — parse SVG
      → tiny_skia::Pixmap::new(width * scale, height * scale) — create canvas
      → resvg::render(&tree, transform, &mut pixmap.as_mut()) — rasterize
      → pixmap.encode_png() → write to disk
```

---

## File Reference Map

### Core Source Files (src/core/)

| Path | Role |
|------|------|
| `opendesign/html.rs` | Top-level HTML compilation: fragment extraction, root container search, pipeline orchestration |
| `opendesign/svg/extract.rs` | SVG markup extraction, asset key generation, viewBox/size calculation |
| `opendesign/svg/rasterize.rs` | resvg/tiny-skia PNG rasterization and disk writing |
| `opendesign/svg/render.rs` | SVG → BuiNodeType::Image conversion |
| `opendesign/generic/tree.rs` | DOM tree walking, element processing, SVG→image node creation |
| `opendesign/generic/document.rs` | HTML → BuiDocument compilation entry point |
| `runtime/plugin.rs` | Bevy plugin: source loading, system registration, editor toggle |
| `runtime/spawn.rs` | BuiNode → ECS entity spawning (all node types including Image) |
| `runtime/image.rs` | Image node building (AssetServer.load, TextureAtlas, background layout) |
| `runtime/components.rs` | ECS components/resources: BuiId, BuiRootEntity, BuiDocumentResource, BuiSourcePaths |
| `editor/state.rs` | Editor state: mode, selection, drag, edits |
| `editor/mod.rs` | Editor module: 14 sub-modules (hover, drag, delete, save, etc.) |
| `model/ir.rs` | BuiDocument, BuiNode, BuiNodeType, all IR types |
| `model/style.rs` | BuiStyles (layout + visual style structs) |
| `model/visual.rs` | BuiVisuals, BuiImageConfig, BuiTextConfig, BuiBoxShadowConfig |
| `parse/ir.rs` | BUI JSON parsing from string/file |
| `parse/validate/entry.rs` | Document-level validation |
| `api.rs` | Public API: compile HTML→JSON, validate JSON |

### Test Files (src/tests/)

| Path | Coverage |
|------|----------|
| `css_effects/svg_rasterize.rs` | SVG→PNG pipeline (8 tests) |
| `cases_hero.rs` | Hero game UI (3 tests, including HTML↔IR equality) |
| `opendesign_html.rs` | Full HTML compilation tests |
| `shared.rs` | Test fixtures: HERO_GAME_UI_HTML, HERO_GAME_UI_IR |

### External Test Data

| Path | Content |
|------|---------|
| `examples/UiParserTest/opendesignTest/hero_game_ui/hero-game-ui.html` | Hero game UI HTML source |
| `examples/UiParserTest/opendesignTest/hero_game_ui/hero-game-ui.ir.json` | Regenerated IR JSON (image nodes) |
| `examples/UiParserTest/opendesignTest/hero_game_ui/assets/png/` | 15 PNG files from SVG rasterization |
| `/Volumes/DockCase/CodeRep/AITOUIToolkit/Assets/Examples/opus48/` | 72 HTML test cases (Dev + interactive) |

### Submodule Dependencies

| Path | Version | Notes |
|------|---------|-------|
| `crates/bevy_egui/` | v0.40.0 | All bevy deps patched to path with `0.19.0-dev` |
| `crates/bevy_inspector_egui/` | custom | `CreateTypeData` fix applied, cargo check passes |

### Cargo.toml Key Dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| `bevy_app` | 0.19.0-dev (path) | Bevy app framework |
| `bevy_ecs` | 0.19.0-dev (path) | ECS system |
| `bevy_asset` | 0.19.0-dev (path) | Asset loading |
| `bevy_ui` | 0.19.0-dev (path) | Bevy UI system |
| `roxmltree` | 0.21.1 | XML/HTML DOM parsing |
| `serde_json` | 1.0.140 | JSON serialization |
| `resvg` | 0.44 | SVG rasterization |
| `usvg` | 0.44 | SVG parsing |
| `tiny-skia` | 0.11 | Software rendering backend |

---

## Verification Commands

```bash
# Compile check
cargo check -p bevy_ai_ui_parser

# Run all 152 lib tests
cargo test -p bevy_ai_ui_parser --lib

# Compile a single HTML file to IR JSON
cargo run --example compile_opendesign_html -- <html_path> <output_ir_json_path>

# Batch test all opus48 HTML files (72 files, all should pass)
find /Volumes/DockCase/CodeRep/AITOUIToolkit/Assets/Examples/opus48 \
  -name "*.html" ! -name "*.meta" | sort | while read f; do \
  cargo run --example compile_opendesign_html -- "$f" /tmp/test.json 2>&1 | grep -E "Compiled|Failed"; \
done

# Validate a BUI IR JSON file
cargo run --example validate_bui_json -- <ir_json_path>
```

---

## Git Status — Uncommitted Changes

The SVG pipeline refactor is complete but uncommitted. Summary of changes:

**Modified (11 files)**:
- `Cargo.toml` — added resvg, usvg, tiny-skia deps
- `src/core/opendesign/html.rs` — expanded HTML compilation (root containers, entity normalization, find_matching_div_close, find_element_with_class)
- `src/core/opendesign/generic/tree.rs` — added svg_assets param, SVG→image node creation
- `src/core/opendesign/generic/document.rs` — added svg_assets param
- `src/core/opendesign/generic/mod.rs` — removed unused re-export
- `src/core/opendesign/hero/mod.rs` — simplified, removed ensure_text_icon_child
- `src/core/opendesign/svg/mod.rs` — rewritten module exports
- `src/core/opendesign/svg/render.rs` — rewritten (is_svg_tag + svg_image_node only)
- `src/core/runtime/plugin.rs` — auto-save .ir.json on HtmlPath compile
- `src/tests/css_effects/mod.rs` — svg_fallback → svg_rasterize module rename
- `src/tests/cases_hero.rs` — structural → full equality comparison

**Deleted (3 files)**:
- `src/core/opendesign/svg/semantic.rs` — 248 lines
- `src/core/opendesign/svg/shape.rs` — 218 lines
- `src/tests/css_effects/svg_fallback.rs` — 85 lines

**New (3 files)**:
- `src/core/opendesign/svg/extract.rs` — 115 lines
- `src/core/opendesign/svg/rasterize.rs` — 52 lines
- `src/tests/css_effects/svg_rasterize.rs` — 95 lines

**Net**: -612 lines (simplified from custom SVG interpretation to resvg rasterization)

---

## Session Handoff Checklist

When starting a new session, verify:

1. `cargo check -p bevy_ai_ui_parser` — no errors
2. `cargo test -p bevy_ai_ui_parser --lib` — 152 tests pass
3. Read this DEVPLAN.md for current state
4. Check git status for uncommitted changes
5. Pick up from the "Remaining Work" section, starting with the highest-priority item

---

## Change Log

| Date | What | Details |
|------|------|---------|
| 2026-06-23 | SVG→PNG pipeline complete | resvg 0.44 + tiny-skia 0.11 + usvg 0.44; 2x render scale; 8 tests |
| 2026-06-23 | Root container expansion | Added `<div class="page">`, `<div class="stage">`; 72/72 opus48 pass |
| 2026-06-23 | Entity normalization rewrite | Bare `&` → `&amp;`; known entities preserved |
| 2026-06-23 | UTF-8 div close tracker | `find_matching_div_close()` replaces `rfind("</div>")` |
| 2026-06-23 | Class exact matching | `find_element_with_class()` prevents false prefix matches |
| 2026-06-23 | Auto-save .ir.json | `load_bui_document(HtmlPath)` writes IR alongside HTML |
| 2026-06-23 | IR fixture regeneration | hero-game-ui.ir.json with image nodes; full equality test |
