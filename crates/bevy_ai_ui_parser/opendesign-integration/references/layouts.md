# Bevy UI Layout Starters

These are starter shapes for common Bevy UI screens. Keep them shallow and
stable, then swap in real assets.

## 1. HUD Overlay

Use for combat HUD, live status bars, and skill trays.

```html
<main class="bevy-screen">
  <section class="bevy-panel bevy-row" id="top_status">
    <div class="bevy-fill bevy-stack">
      <h1 class="bevy-title">Encounter</h1>
      <p class="bevy-copy">Wave 03 · Northern Gate</p>
    </div>
    <div id="party_hp" data-action="open_party_stats"></div>
  </section>

  <section class="bevy-row" id="bottom_hud">
    <div class="bevy-panel bevy-fill" id="skill_bar"></div>
    <button class="bevy-button" id="inventory_btn" data-action="open_inventory">Inventory</button>
  </section>
</main>
```

## 2. Settings / System Menu

Use for tabbed panels with 9-slice backgrounds and button state images.

```html
<main class="bevy-screen">
  <section class="bevy-panel bevy-grid" id="settings_shell">
    <header class="bevy-row" id="settings_header">
      <h1 class="bevy-title">Settings</h1>
      <button class="bevy-button" id="close_btn" data-action="close_settings">Close</button>
    </header>

    <nav class="bevy-row" id="settings_tabs">
      <button id="tab_audio" data-tab="audio" data-tab-group="settings">Audio</button>
      <button id="tab_video" data-tab="video" data-tab-group="settings">Video</button>
      <button id="tab_controls" data-tab="controls" data-tab-group="settings">Controls</button>
    </nav>

    <section id="panel_audio" data-tab-panel="audio" data-tab-group="settings"></section>
    <section id="panel_video" data-tab-panel="video" data-tab-group="settings"></section>
    <section id="panel_controls" data-tab-panel="controls" data-tab-group="settings"></section>
  </section>
</main>
```

## 3. Inventory / Equipment

Use for icon grids, item cards, and inspect panels.

```html
<main class="bevy-screen">
  <section class="bevy-grid" id="inventory_layout" style="grid-template-columns: 1.1fr 0.9fr;">
    <div class="bevy-panel bevy-stack" id="inventory_grid"></div>
    <aside class="bevy-panel bevy-stack" id="item_inspector">
      <div id="item_icon"></div>
      <div id="item_stats"></div>
      <button class="bevy-button" id="equip_btn" data-action="equip_item">Equip</button>
    </aside>
  </section>
</main>
```

## 4. Modal / Dialog

Use for quest accept dialogs, reward popups, or confirm flows.

```html
<main class="bevy-screen">
  <section class="bevy-panel bevy-stack" id="quest_dialog">
    <h1 class="bevy-title">Accept Quest</h1>
    <p class="bevy-copy">The northern caravan needs escort through wolf territory.</p>
    <div class="bevy-row">
      <button class="bevy-button" id="confirm_btn" data-action="accept_quest">Accept</button>
      <button class="bevy-button" id="cancel_btn" data-action="cancel_dialog">Cancel</button>
    </div>
  </section>
</main>
```

## Assembly reminders

- Real image-backed art should live in `Asset/...`
- Use manifest selectors like `#close_btn`, `#confirm_btn`, `#quest_dialog`
- Keep DOM depth around 4 or less
- Prefer ids for components that need button states, slicers, or atlases
