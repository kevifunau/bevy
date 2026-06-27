# Bevy AI UI Parser API Guide

This crate is a Bevy plugin for loading constrained Open Design / Bevy UI HTML or BUI JSON into native Bevy UI. It is not a browser runtime: it parses a stable declarative contract and routes interaction into ECS.

## Output Shape Expected From Open Design

```text
my_screen/
  index.html
  Asset/
    panel.png
    button_idle.png
    button_active.png
    icon_settings.png
```

`bevy-ui.assets.json` may be useful for AI handoff or later tooling, but the runtime path can load `index.html` plus referenced images directly.

## Minimal Bevy Setup

The recommended workflow: compile HTML to IR JSON at build time, then load IR at runtime.

```rust
use bevy::prelude::*;
use bevy_ai_ui_parser::AiUiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AiUiPlugin::from_path("ui/main_menu.ir.json"))
        .run();
}
```

For editor-enabled loading:

```rust
app.add_plugins(AiUiPlugin::from_path_with_editor("ui/main_menu.ir.json"));
```

For development prototyping (runtime HTML compilation — not recommended for production):

```rust
app.add_plugins(AiUiPlugin::from_html_path("ui/main_menu/index.html"));
```

## Asset Loading

When using `AiUiPlugin::from_path(...)` (recommended), image URLs from the original HTML are preserved in the IR JSON and resolved through Bevy's normal `AssetServer`. In examples, the asset root is usually set to the folder containing the original HTML, so paths such as `Asset/play-button.png` work.

When using `AiUiPlugin::from_html_path(...)` (dev prototyping only), the HTML is compiled to IR at runtime before loading — this adds startup latency and is not intended for production use.

In a game project, either:

- Put UI folders under the Bevy asset root and reference them with asset-relative paths.
- Or configure an `AssetPlugin` file path / asset source that matches your project layout.

## Supported Control Contracts

### Button

```html
<button id="play_button" data-action="game.start_matchmaking">PLAY</button>
```

Runtime behavior:

- Pointer press emits `BuiActionTriggered { trigger: Press }`.
- Focused button + `Enter` / `Space` emits the same press action.
- Focused button + gamepad South emits the same press action.
- Pointer press moves Bevy input focus to the pressed control.
- `disabled="true"` or `aria-disabled="true"` blocks dispatch.

### Toggle / Checkbox

```html
<input
  id="audio_toggle"
  type="checkbox"
  checked="true"
  data-binding="settings.audio_enabled"
  data-action-change="settings.audio.changed"
/>
```

Runtime behavior:

- Press toggles Bevy `Checked`.
- Writes `<id>.checked` and the `data-binding` key as `BuiBindingValue::Bool`.
- Emits `ValueChanged`.

### Text Input / InputField

```html
<input
  id="account_input"
  type="text"
  value="Racer"
  placeholder="Account"
  data-binding="login.account"
  data-action-change="login.account.changed"
  data-action-submit="login.submit"
  data-action-focus="login.focus"
  data-action-blur="login.blur"
/>
```

Runtime behavior:

- Value changes write `<id>` and the `data-binding` key as text.
- Focus / blur emit lifecycle actions.
- Focused input + `Enter` emits `Submit`.
- Focused input does not treat `Enter` as a button press.

### Slider

```html
<input
  id="volume_slider"
  type="range"
  min="0"
  max="100"
  value="65"
  step="5"
  data-binding="settings.volume"
  data-action-change="settings.volume.changed"
/>
```

Runtime behavior:

- `SliderValue` changes write `<id>` and the `data-binding` key as number.
- Emits `ValueChanged`.
- Focused slider + arrow keys or DPad left/right increments/decrements by `step` and clamps to `min` / `max`.
- Pointer drag / track behavior is delegated to Bevy UI Widgets slider behavior.

### ScrollView / Scroller

```html
<section
  id="inventory_scroll"
  data-scroll-view="true"
  data-scroll-binding="inventory.list"
  data-action-scroll="inventory.scrolled"
  style="overflow-y: auto"
>
  ...
</section>
```

Runtime behavior:

- Mouse wheel updates Bevy `ScrollPosition` on the hovered scroll view.
- Ctrl + wheel swaps axes.
- Focused scroll view + arrow keys or DPad scrolls in the requested axis.
- Writes `<id>.scroll_x`, `<id>.scroll_y`, and bound scroll keys.
- Emits `Scroll`.

### DropDown

```html
<select id="difficulty_select" data-binding="settings.difficulty">
  <option value="easy">Easy</option>
  <option value="hard" selected="true" data-action-select="difficulty.changed">Hard</option>
</select>
```

Runtime behavior:

- Pressing an option writes the selected value to the bound key.
- Focused option + `Enter` / `Space` / gamepad South selects that option.
- Selected option receives the `selected` visual state.
- Emits `SelectionChanged`.
- Popup open/close is a future layer on this contract.

### Focus Navigation

Runtime behavior:

- Controls with a stable `id` and Bevy UI semantics enter the parser-generated focus order.
- The focus order follows the spawned OD/BUI document tree, not ECS entity ids.
- `Tab` moves to the next control; `Shift+Tab` moves to the previous control.
- DPad up/down and arrow up/down move between controls unless the focused control consumes those keys, such as a slider or scroll view.
- Pointer press moves focus to the pressed control.
- Focused pressable controls use `Enter`, `Space`, and gamepad South for activation.

## Registering Business Logic

Prefer the action registry for game-side behavior:

```rust
use bevy::prelude::*;
use bevy_ai_ui_parser::{AiUiPlugin, BuiActionAppExt};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AiUiPlugin::from_html_path("ui/car/index.html"))
        .add_bui_action_handler("game.start_matchmaking", |world, event| {
            info!("BUI action {} from node {}", event.action, event.id);
            let _ = world;
        })
        .run();
}
```

You can also read `BuiActionTriggered` directly:

```rust
use bevy::prelude::*;
use bevy_ai_ui_parser::BuiActionTriggered;

fn handle_actions(mut actions: MessageReader<BuiActionTriggered>) {
    for action in actions.read() {
        match action.action.as_str() {
            "game.start_matchmaking" => {}
            "garage.open" => {}
            _ => {}
        }
    }
}
```

## Updating UI State From Game Code

Use `BuiStateSet` for semantic state writes:

```rust
use bevy::prelude::*;
use bevy_ai_ui_parser::{BuiBindingValue, BuiStateSet};

fn set_status(mut writer: MessageWriter<BuiStateSet>) {
    writer.write(BuiStateSet {
        key: "race.status".to_string(),
        value: BuiBindingValue::Text("MATCHMAKING...".to_string()),
    });
}
```

HTML nodes bind to that state with `data-binding` or parser-specific semantics. For example:

```html
<strong id="race_status_text" data-binding="race.status">READY</strong>
```

## Static Prototype Actions

The runtime can execute a small declarative action DSL. Use it for OD prototype parity and light UI behavior, not for business logic.

```html
<script type="application/json" data-bui-actions>
{
  "actions": {
    "open-events": [
      { "op": "set-selected", "group": ["nav_garage", "nav_events"], "target": "nav_events" },
      { "op": "set-text", "binding": "race.status", "node": "race_status_text_text_1", "value": "EVENTS OPENED" }
    ],
    "start-race": [
      { "op": "set-text", "binding": "race.status", "node": "race_status_text_text_1", "value": "MATCHMAKING..." },
      { "op": "delay", "ms": 900 },
      { "op": "set-text", "binding": "race.status", "node": "race_status_text_text_1", "value": "READY TO RACE" }
    ]
  }
}
</script>
```

Supported operations in v1:

- `set-text`
- `set-binding` / `set-state`
- `set-image`
- `set-selected-image`
- `set-selected`
- `set-visible`
- `set-visual-state`
- `clear-visual-state`
- `run-action`
- `delay` / `wait`

Static JavaScript exports are accepted:

```html
<script>
window.BUI_ACTIONS = { "actions": { "open": [ { "op": "set-visible", "target": "panel", "value": "visible" } ] } };
</script>
```

```html
<script>
Bui.registerActions({ "actions": { "open": [ { "op": "set-visible", "target": "panel", "value": "visible" } ] } });
</script>
```

The Bevy runtime does not execute arbitrary browser JavaScript.

## Compile / Validate Without Running Bevy UI

```rust
use bevy_ai_ui_parser::{
    opendesign_html_file_to_bui_json,
    opendesign_html_to_bui_json_str,
    validate_bui_json_file,
    validate_bui_json_str,
};

let json = opendesign_html_file_to_bui_json("ui/car/index.html")?;
validate_bui_json_str(&json)?;
# Ok::<(), String>(())
```

## Current v1 Limits

- Arbitrary `<script>` logic is not executed.
- Dropdown popup open/close is not complete yet.
- Slider pointer dragging / track interaction relies on Bevy UI Widgets behavior.
- Scrollbar thumb dragging is not implemented yet.
- JS-style backend calls should become ECS action handlers, not browser `fetch()`.
