既然我们的目标是建立一套**不依赖特定引擎、完全数据驱动（Data-Driven）的声明式 UI**，那么这份设计稿的核心就不仅是“视觉长什么样”，更是“如何用通用数据结构去定义和描述这三个界面”。

下面我为你梳理这套真人互动影游（FMV）的 **登录/主界面**、**章节选择界面**、**设置界面** 以及 **普通游戏内置界面（Back）** 的抽象设计稿。每个界面分为 **布局蓝图（Layout Blueprint）** 和 **声明式数据描述（Data Manifesto）** 两部分，你可以直接用这套 JSON 数据在任何引擎（包括 Unity UGUI 或未来的 Bevy UI）中动态还原出来。

---

## 1. 登录与主菜单界面 (Main Menu / Title View)

### 📐 布局蓝图

该界面采用**非对称式左侧垂直对齐**布局，右侧 65% 的空间完全留给高精度的动态影视视频（Video Loop），确保极强的影视沉浸感。

```
+-------------------------------------------------------------------------+
| [SAFE AREA TOP]                                                         |
|                                                                         |
|  +------------------+                                                   |
|  |   GAME TITLE     |                                                   |
|  |  《华君传》 Logo  |                                                   |
|  +------------------+                                                   |
|                                                                         |
|  +------------------+             [ 右侧 65% 空间 ]                      |
|  | [▶] NEW GAME     |                                                   |
|  +------------------+             影视动态视频背景                       |
|  | [⟲] CONTINUE     |             (Video Loop Layer)                    |
|  +------------------+                                                   |
|  | [☰] CHAPTERS     |                                                   |
|  +------------------+                                                   |
|  | [⚙] SETTINGS     |                                                   |
|  +------------------+                                                   |
|  | [⎋] QUIT         |                                                   |
|  +------------------+                                                   |
|                                                                         |
|                                   +----------------------------------+  |
|                                   |  "江山为棋，情为局" (Slogan Text) |  |
| [SAFE AREA BOTTOM]                +----------------------------------+  |
+-------------------------------------------------------------------------+

```

### 📄 声明式 UI 数据描述

```json
{
  "view_id": "main_menu",
  "background": {
    "type": "VIDEO_STREAM",
    "uri": "videos/main_menu_bg.mp4",
    "loop": true
  },
  "layout": {
    "type": "FLEX_ROW",
    "justify": "START",
    "align": "CENTER",
    "padding": { "left": "10%", "top": "5%", "bottom": "5%" }
  },
  "components": [
    { "id": "logo", "type": "IMAGE", "asset_uri": "textures/ui/game_logo.png" },
    {
      "id": "menu_list",
      "type": "VERTICAL_STACK",
      "spacing": 20,
      "children": [
        { "id": "btn_new", "type": "BUTTON", "style": "PRIMARY_CAPSULE", "text": "NEW GAME", "action": "STORY_TRANSIT:node_001" },
        { "id": "btn_continue", "type": "BUTTON", "style": "PRIMARY_CAPSULE", "text": "CONTINUE", "action": "LOAD_LAST_SAVE" },
        { "id": "btn_chapters", "type": "BUTTON", "style": "PRIMARY_CAPSULE", "text": "CHAPTER SELECT", "action": "SWITCH_VIEW:chapter_select" },
        { "id": "btn_settings", "type": "BUTTON", "style": "PRIMARY_CAPSULE", "text": "SETTINGS", "action": "PUSH_POPUP:settings_menu" },
        { "id": "btn_quit", "type": "BUTTON", "style": "PRIMARY_CAPSULE", "text": "QUIT", "action": "ENGINE_EXIT" }
      ]
    }
  ]
}

```

---

## 2. 章节选择与进度界面 (Chapter Select View)

### 📐 布局蓝图

经典的三段式横向拆分布局：**左侧列表选关、中间/右侧预览画面与进度状态、底部功能触发按钮**。
为了防止高亮的视频背景干扰文字可读性，整个面板底层会附带一层 `Alpha 0.75` 的半透明深色遮罩盒（Dark Mask Box）。

```
+-------------------------------------------------------------------------+
| [← BACK]                                              CHAPTER SELECT    |
| ----------------------------------------------------------------------- |
|  +----------------------+    +---------------------------------------+  |
|  | CHAPTER LIST         |    | CHAPTER PREVIEW (Raw Image / Sprite)  |  |
|  |                      |    |                                       |  |
|  | [★] 第 1 章：初入宫廷  |    |                                       |  |
|  |     (Status: 100%)   |    |                                       |  |
|  |                      |    |                                       |  |
|  | [▶] 第 2 章：女帝选秀  |    | +-----------------------------------+ |  |
|  |     (Status: 30%)    |    | | Progress: [███░░░░░░░] 30%        | |  |
|  |                      |    | +-----------------------------------+ |  |
|  | [🔒] 第 3 章：深宫谍影  |    +---------------------------------------+  |
|  |     (Locked)         |                                               |
|  +----------------------+    +-------------------+ +-----------------+  |
|                              | [▶ ENTER CHAPTER] | | [➔ STORY TREE]  |  |
|                              +-------------------+ +-----------------+  |
+-------------------------------------------------------------------------+

```

### 📄 声明式 UI 数据描述

```json
{
  "view_id": "chapter_select",
  "background": {
    "type": "STATIC_SPRITE",
    "uri": "textures/bg/chapter_blur_bg.jpg"
  },
  "components": [
    { "id": "btn_back", "type": "BUTTON", "style": "NAV_BACK", "text": "BACK", "action": "SWITCH_VIEW:main_menu" },
    {
      "id": "panel_container",
      "type": "HORIZONTAL_SPLIT",
      "left_width_ratio": 0.35,
      "left_children": [
        {
          "id": "chapter_repeater",
          "type": "DYNAMIC_LIST",
          "data_source": "blackboard.chapters",
          "template": {
            "type": "CHAPTER_ITEM_WIDGET",
            "states": {
              "locked": { "interactable": false, "style": "GREY_OUT", "icon": "lock.png" },
              "unlocked": { "interactable": true, "style": "HIGHLIGHT", "icon": "play.png" }
            }
          }
        }
      ],
      "right_children": [
        { "id": "chapter_thumb", "type": "IMAGE", "binding": "selected_chapter.preview_uri" },
        { "id": "progress_bar", "type": "PROGRESS_BAR", "binding": "selected_chapter.progress_value" },
        { "id": "btn_enter", "type": "BUTTON", "style": "CTA_LARGE", "text": "ENTER CHAPTER", "action": "START_CHAPTER:selected_chapter.id" },
        { "id": "btn_tree", "type": "BUTTON", "style": "SECONDARY_CAPSULE", "text": "STORY TREE", "action": "SWITCH_VIEW:story_tree" }
      ]
    }
  ]
}

```

---

## 3. 设置界面 (Settings Popup / Modal View)

### 📐 布局蓝图

设置为全屏弹出式灯箱（Modal Overlay）效果。中心为矩形九宫格面板，四周自带暗化效果。组件采用统一的水平通栏排版，左文本、右控件。

```
+-------------------------------------------------------------------------+
|                                                                         |
|       +---------------------------------------------------------+       |
|       |  SETTINGS                                           [X] |       |
|       | ------------------------------------------------------- |       |
|       |                                                         |       |
|       |  MASTER VOLUME    [━━━━━━━●──────────]  60%             |       |
|       |                                                         |       |
|       |  MUSIC VOLUME     [━━━━━━━━━━━━━━━━━●]  100%            |       |
|       |                                                         |       |
|       |  SFX VOLUME       [━━●───────────────]  20%             |       |
|       |                                                         |       |
|       |  LANGUAGE         [ English                       [▼] ] |       |
|       |                                                         |       |
|       | ------------------------------------------------------- |       |
|       |                                       [APPLY]  [CANCEL] |       |
|       +---------------------------------------------------------+       |
|                                                                         |
+-------------------------------------------------------------------------+

```

### 📄 声明式 UI 数据描述

```json
{
  "view_id": "settings_menu",
  "type": "POPUP_MODAL",
  "overlay_color": "#000000B2", 
  "components": [
    {
      "id": "settings_dialog",
      "type": "CONTAINER",
      "style": "DIALOG_9SLICE_BOX",
      "children": [
        { "id": "txt_title", "type": "TEXT", "content": "SETTINGS" },
        { "id": "btn_close", "type": "BUTTON", "style": "ICON_CLOSE", "action": "CLOSE_POPUP" },
        {
          "id": "setting_rows",
          "type": "VERTICAL_STACK",
          "children": [
            {
              "type": "SETTING_ROW_ITEM",
              "label": "MASTER VOLUME",
              "control": { "type": "SLIDER", "min": 0, "max": 100, "binding": "settings.audio.master" }
            },
            {
              "type": "SETTING_ROW_ITEM",
              "label": "MUSIC VOLUME",
              "control": { "type": "SLIDER", "min": 0, "max": 100, "binding": "settings.audio.music" }
            },
            {
              "type": "SETTING_ROW_ITEM",
              "label": "LANGUAGE",
              "control": { "type": "DROPDOWN", "options": ["简体中文", "English", "日本語"], "binding": "settings.language" }
            }
          ]
        },
        { "id": "btn_apply", "type": "BUTTON", "style": "TEXT_SMALL", "text": "APPLY", "action": "SAVE_SETTINGS_AND_CLOSE" }
      ]
    }
  ]
}

```

---

## 4. 普通游戏内置界面 (In-Game HUD View)

### 📐 布局蓝图

当玩家正常观看电影剧本、未触发剧情分支交互时，**UI 必须呈现“零干扰”状态**。全屏无任何杂质，仅在左上角放置一个极简的半透明返回按钮（或者通过手机全面屏边缘侧滑唤出）。

```
+-------------------------------------------------------------------------+
| [← BACK]                                                                |
|                                                                         |
|                                                                         |
|                                                                         |
|                                                                         |
|                        纯全屏游戏视频画面                                 |
|                        (Full-Screen FMV Video)                          |
|                                                                         |
|                                                                         |
|                                                                         |
|                                                                         |
+-------------------------------------------------------------------------+

```

### 📄 声明式 UI 数据描述

```json
{
  "view_id": "in_game_hud",
  "background": {
    "type": "GAME_VIDEO_PLAYER"
  },
  "components": [
    {
      "id": "btn_ingame_back",
      "type": "BUTTON",
      "style": "MINIMAL_ROUND_GHOST",
      "position": { "anchor": "TOP_LEFT", "offset_x": 40, "offset_y": -40 },
      "content": { "text": "BACK", "icon_uri": "textures/icons/back_arrow.png" },
      "action": "PAUSE_VIDEO_AND_PUSH_POPUP:pause_menu"
    }
  ]
}

```

---

### 🛠️ 怎么用这套设计稿移植到任何引擎（如 Bevy/Rust）？

这套设计稿最大的优势在于**它是一套纯数据描述。**
如果你后续切换到 **Bevy Engine**：

1. 你只需要写一个 Rust 的解析系统（System），读取这几段 JSON 文件。
2. 遇到 `type: "VERTICAL_STACK"`，就在 ECS 中为实体（Entity）挂载 Bevy UI 的 `FlexDirection::Column` 属性。
3. 遇到 `action: "SWITCH_VIEW:chapter_select"`，就在 Rust 中发出一个状态切换事件（Event），清除当前界面 Entities，并根据新界面的 JSON 动态生成（Spawn）新界面组件。

这样，UI 的视觉和排版逻辑完全独立，无论未来要改版还是由 AI 批量动态生成，你都只需要改动 JSON 配置，底层引擎代码一行都不需要动！