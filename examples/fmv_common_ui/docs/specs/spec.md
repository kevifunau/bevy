这是一个**不依赖任何特定游戏引擎（Engine-Agnostic）**的通用系统设计规范文档（System Specification Document）。由于底层未来可能会采用 **Rust 与 Bevy Engine** 这一类基于 **ECS（Entity Component System，实体组件系统）** 架构的引擎，本规范将完全从**数据驱动、声明式UI（Declarative UI）以及状态机**的抽象维度进行设计，确保其核心逻辑可以无缝移植到任何图形框架中。

---

# 系统设计规范：通用 FMV 智能互动影像系统 (IFUNS-Spec)

**文档版本：** v1.0.0

**架构类型：** 数据驱动 / 声明式状态机 (Data-Driven & Declarative State Machine)

**适用范围：** 跨平台、跨引擎（支持传统 OOP 引擎及现代 ECS 引擎如 Bevy）

---

## 1. 核心设计哲学

为了保证系统能够完全独立于游戏引擎，且能完美契合未来的 AI 自动化生成（剧本直转游戏），系统基于以下三个原则设计：

1. **数据即状态（Data as Truth）：** 所有的 UI 表现、视频播放、分支控制，完全由一份结构化的数据（如 JSON/BSON）定义。
2. **声明式渲染（Declarative Rendering）：** 引擎的 UI 层只负责“根据当前状态数据（State）渲染界面”，UI 本身不存储业务逻辑（无状态 UI）。
3. **黑板模式解耦（Blackboard Pattern）：** 玩家的进度、数值（好感度、关键线索）存储在独立的“全局黑板”中，逻辑判定通过条件表达式（Condition Expressions）进行求值。

---

## 2. 核心数据字典规范 (Data Schema Specification)

系统的最底层是纯粹的数据结构。AI 剧本编译器或生成工具必须输出符合以下规范的数据。

### 2.1 剧情图节点 (Narrative Graph Node)

整个游戏由一个有向有环图（Directed Graph）构成。每个节点（Node）代表一段视频或一个交互时刻。

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "StoryNode",
  "type": "object",
  "properties": {
    "node_id": { "type": "string" },
    "node_title": { "type": "string" },
    "asset_manifest": {
      "type": "object",
      "properties": {
        "video_uri": { "type": "string" },
        "audio_bgm_uri": { "type": "string", "nullable": true },
        "thumbnail_uri": { "type": "string" }
      },
      "required": ["video_uri", "thumbnail_uri"]
    },
    "playback_mode": { 
      "type": "string", 
      "enum": ["ONCE_AND_STOP", "ONCE_AND_LOOP", "INFINITE_LOOP"] 
    },
    "interaction": {
      "type": "object",
      "properties": {
        "type": { "type": "string", "enum": ["NONE", "SELECTION", "QTE", "INPUT_SLIDER"] },
        "trigger_time": { "type": "number", "description": "触发交互的视频时间戳(秒)，-1表示视频播放完毕后触发" },
        "timeout": { "type": "number", "description": "交互超时时间(秒)，-1表示无限等待" },
        "timeout_target_node": { "type": "string", "nullable": true },
        "components": {
          "type": "array",
          "items": { "$ref": "#/definitions/InteractionComponent" }
        }
      },
      "required": ["type", "trigger_time"]
    }
  },
  "required": ["node_id", "node_title", "asset_manifest", "playback_mode", "interaction"]
}

```

### 2.2 交互组件元素 (Interaction Component)

定义 UI 交互肢的抽象属性（例如按钮、滑动条）：

```json
{
  "definitions": {
    "InteractionComponent": {
      "type": "object",
      "properties": {
        "component_id": { "type": "string" },
        "visual_style": { "type": "string", "description": "UI样式标记，如 'PRIMARY_CAPSULE', 'GHOST_LINE'" },
        "content": {
          "type": "object",
          "properties": {
            "text": { "type": "string" },
            "icon_uri": { "type": "string", "nullable": true }
          },
          "required": ["text"]
        },
        "behavior": {
          "type": "object",
          "properties": {
            "condition": { "type": "string", "description": "布尔表达式，如 'player.gold >= 100 && player.reputation > 5'" },
            "on_click_target": { "type": "string", "description": "点击后跳转的目标 NodeId" },
            "mutations": { 
              "type": "array", 
              "items": { "type": "string" },
              "description": "状态变更表达式，如 ['player.gold -= 100', 'npc.favor += 10']" 
            }
          },
          "required": ["on_click_target"]
        }
      },
      "required": ["component_id", "content", "behavior"]
    }
  }
}

```

---

## 3. 抽象系统模块设计 (Abstract Module Design)

无论使用何种引擎，系统内部必须抽象出以下四个相互隔离的运行时管理器：

```
+-----------------------------------------------------------------------+
|                       1. Asset Pipeline (资源管道)                     |
|            (负责异步加载 URI 指定的视频/图片，生成底层纹理句柄)                  |
+-----------------------------------------------------------------------+
                                   |
                                   v
+-----------------------------------------------------------------------+
|                       2. Global Blackboard (全局黑板)                  |
|            (存储所有运行时变量: { "player": { "gold": 100 } })          |
+-----------------------------------------------------------------------+
                                   |
                         (条件判定 / 状态变更)
                                   v
+-----------------------------------------------------------------------+
|                       3. State Machine Director (状态机导演)           |
|            (驱动当前 Node 状态切换、控制视频播放进度与交互触发时机)               |
+-----------------------------------------------------------------------+
                                   |
                           (输出只读 UI 状态流)
                                   v
+-----------------------------------------------------------------------+
|                       4. Declarative UI Renderer (声明式UI渲染器)      |
|            (接收 UI State 描述数据，动态映射为具体引擎的 UI 节点/实体)            |
+-----------------------------------------------------------------------+

```

### 3.1 状态机导演 (State Machine Director)

系统核心。管理一个逻辑上的虚拟时钟（Virtual Clock），该时钟与当前底层视频播放器的播放进度绑定。

* **Tick 驱动逻辑：** 1. 当视频播放时间达到 `interaction.trigger_time` 时，导演向外发出一个 `UI_RENDER_REQUEST` 信号，附带 `components` 数据。
2. 如果 `timeout` 大于 0，启动倒计时。倒计时结束未操作，自动触发 `timeout_target_node` 路由。
3. 接收到 UI 层传回的 `ON_COMPONENT_CLICKED` 事件后，执行表达式解析器。

### 3.2 表达式解析器 (Expression Evaluator)

这是一个轻量级的 DSL（领域特定语言）解析器，独立于引擎。

* **Condition 求值：** 将数据中的字符串（如 `"player.favorability >= 50"`）解析为抽象语法树（AST），并在全局黑板数据上运行，返回 `true` 或 `false`。
* **Mutation 执行：** 解析诸如 `"player.gold -= 10"` 这样的指令，直接修改黑板中的数值，并通知相关系统。

---

## 4. 针对 ECS 架构（如 Bevy）的适配指引

由于你提到后续可能会采用 **Bevy Engine (Rust)** 研发，以下是该 Spec 在 ECS 架构下的对应概念映射：

### 4.1 核心组件设计 (Components)

在 Bevy 中，上述 JSON 数据不会存成复杂的嵌套树，而是打散为**扁平的实体（Entities）和组件（Components）**：

* **`StoryNodeComponent` (组件)：** 挂载在代表当前剧情节点的 Entity 上，包含视频路径和播放状态。
* **`InteractionComponent` (组件)：** 作为子实体（Child Entity）挂载。包含 `TargetNode`、`ConditionString`。
* **`BlackboardResource` (资源)：** 在 Bevy 中以 `Resource` 形式全局唯一存在，使用 `HashMap<String, Val>` 存储。

### 4.2 核心系统切分 (Systems)

* **`video_playback_system`：** 负责检测当前视频播放进度。当时间戳匹配时，给符合条件的 Interaction 实体附加一个 `ActiveUI` 标记组件。
* **`declarative_ui_system` (对应 Bevy UI 或 `bevy_egui`/`kayak_ui`)：** * 该 System 监听带有 `ActiveUI` 标记的实体。
* 读取实体数据，使用类似声明式（类似 React/Yew）的代码动态构建 UI 树：
```rust
// 伪代码示例：Bevy UI 声明式流
fn render_ui_system(
    mut commands: Commands,
    query: Query<(&InteractionComponent, &ActiveUI)>,
    blackboard: Res<BlackboardResource>
) {
    for (interaction, _) in query.iter() {
        if evaluate_condition(&interaction.condition, &blackboard) {
            // 使用 Bevy UI 动态创建按钮实体
            commands.spawn(NodeBundle { .. })
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from(interaction.text.clone()));
                    });
        }
    }
}

```





---

## 5. UI 视觉与排版通用规范 (Agnostic Layout Spec)

为了确保 UI 在任何引擎（包括 Rust 的原生网格渲染、Bevy UI 或 HTML/Canvas）下表现一致，排版必须遵循以下数学原则：

1. **比例盒模型（Percentage-Based Box Model）：** * 所有容器和按钮的相对尺寸必须基于视口（Viewport）百分比或系数（如：主菜单面板最大宽度固定为 `0.35 * Viewport_Width`）。
2. **安全区（Safe Area Insets）：**
* 系统提供全局 `SafeArea` 变量（由底层图形环境提供，如刘海屏数据）。所有交互核心 UI 组件的绝对锚点（Anchors）必须在 `SafeArea` 边界内进行偏移计算。


3. **九宫格网格定义（Generic 9-Slice Definition）：**
* 资产清单中需明确指定边框切片像素：`[Top, Right, Bottom, Left]`。渲染引擎在处理长条形或大块半透明面板时，必须根据此数据保证其四个角（Corners）的像素长宽比为 `1:1` 保持不形变。