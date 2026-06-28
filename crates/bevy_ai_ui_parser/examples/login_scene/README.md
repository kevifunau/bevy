# Login Scene — 从 Unity UGUI 到 Bevy 的完整复刻

本目录是一个完整的登录场景游戏，复刻自 Unity UGUI 教学项目（唐老师 UGUI_Demo）。
它演示了如何用 `bevy_ai_ui_parser` 插件开发一个多面板的游戏 UI 流程。

## 目录结构

```
login_scene/
├── webgameui/                    ← UI 设计源码（HTML/CSS/JS + 资源）
│   ├── index.html                ← 登录页（= LoginPanel.prefab）
│   ├── register.html             ← 注册页（= RegisterPanel.prefab）
│   ├── server_select.html        ← 选服页（= ChooseServerPanel.prefab）
│   ├── server_list.html          ← 服务器列表页（= ServerPanel.prefab）
│   ├── Asset/                    ← 图片资源 + ServerInfo.json
│   └── intent.json               ← 设计意图
├── prefabs/                      ← 编译产物（IR JSON，= Unity 的 Prefabs/）
│   ├── index.ir.json
│   ├── register.ir.json
│   ├── server_select.ir.json
│   └── server_list.ir.json
├── main.rs                       ← 游戏代码（业务逻辑，= Unity 的 Scripts/）
└── README.md                     ← 本文件
```

## 开发流程

### 第一步：分析 Unity prefab 结构

用 Python 脚本解析 Unity prefab 的 YAML，提取 GO 层级和组件：

```
LoginPanel (root)
├── imgUN (Image[input_frame])         → 输入框背景
├── imgPW (Image[input_frame])         → 输入框背景
├── btnSure (Button, Image[panel_bg])  → 登录按钮
├── btnRe (Button, Image[panel_bg])    → 注册按钮
├── togPW (Toggle)
│   ├── Background (Image[checkbox_bg])
│   │   └── Checkmark (Image[checkmark])
│   └── Label (Text="记住密码")
├── togAuto (Toggle, Label="自动登录")
├── inputUN (InputField, placeholder="请输入账号")
├── inputPW (InputField, placeholder="请输入密码")
├── txtUN (Text="账号:")
└── txtPW (Text="密码:")
```

### 第二步：用 HTML/CSS/JS 复刻

严格对照 prefab 的 GO 和组件，写成标准 HTML：

```html
<div class="bevy-ui-root game-stage">
  <div class="login-panel">
    <div class="form-row">
      <span class="form-label">账号:</span>
      <div class="input-wrapper">
        <input type="text" placeholder="请输入账号"
               data-binding="login.username" />
      </div>
    </div>
    <button data-action="login.sure">登录</button>
    <button data-action="login.register">注册</button>
  </div>
</div>
```

**关键对照关系：**

| Unity prefab | HTML | 说明 |
|--------------|------|------|
| Prefab `.prefab` (YAML) | `.html` 文件 | 结构 + 样式源码 |
| `Image` 组件 + sprite | CSS `background-image` | 图片背景 |
| `Text` 组件 + text | HTML 标签文字 | 文字内容 |
| `Button` 组件 + onClick | `data-action="xxx"` | 交互事件 |
| `InputField` + placeholder | `<input placeholder="...">` | 输入框 |
| `Toggle` + isOn | `<input type="checkbox">` | 复选框 |
| `CanvasGroup` 淡入淡出 | CSS `transition`（浏览器）/ DSL `delay`（Bevy） | 动画 |

**浏览器预览 JS：** 每个 HTML 里内联了 `<script>` 标签，包含浏览器交互逻辑
（按钮点击、页面跳转、localStorage 模拟数据持久化）。这些 JS 仅用于浏览器预览，
编译时会被插件丢弃——Bevy 运行时的交互由 `main.rs` 的 ECS handler 替代。

### 第三步：编译 HTML → IR JSON

```bash
# 编译单个文件
cargo run -p bevy_ai_ui_parser --bin compile_opendesign_html -- \
  webgameui/index.html prefabs/index.ir.json

# 编译全部 4 个文件
for page in index register server_select server_list; do
  cargo run -p bevy_ai_ui_parser --bin compile_opendesign_html -- \
    webgameui/${page}.html prefabs/${page}.ir.json
done
```

编译器做的事：
1. 解析 HTML（roxmltree XML 解析器，编译前自动剥离 `<script>` 标签）
2. 解析 CSS（选择器、级联、变量、媒体查询）
3. 生成 BUI IR JSON（版本 3.0-ir 的中间表示）
4. 栅格化内联 SVG（resvg → PNG）

**HTML 不被修改**——编译器只读 HTML，输出 IR JSON。如果 HTML 有不支持的
CSS 属性，应该由 `bevy_strict_lint.py` 在编译前校验并报错，而不是在编译时发现。

### 第四步：写游戏代码（main.rs）

main.rs 只包含业务逻辑，不包含任何 UI 加载代码。

**插件负责的（静态 UI）：**
- 加载 IR JSON（= Unity `Instantiate(prefab)`）
- 卸载旧面板（= Unity `Destroy(panel)`）
- 渲染 Bevy UI 实体树
- 交互系统（按钮点击、键盘导航、数据绑定）

**main.rs 负责的（业务逻辑）：**
- 注册面板路径
- Action handler（按钮点击后的业务逻辑）
- 数据绑定（推送数据到 UI）

```rust
fn main() {
    App::new()
        // 插件加载第一个面板 + 注册所有交互系统
        .add_plugins(AiUiPlugin::from_path("prefabs/index.ir.json"))
        // 注册所有面板（插件负责加载/卸载）
        .register_bui_panel("login", "prefabs/index.ir.json")
        .register_bui_panel("register", "prefabs/register.ir.json")
        .register_bui_panel("server_select", "prefabs/server_select.ir.json")
        .register_bui_panel("server_list", "prefabs/server_list.ir.json")
        // Action handlers — 只有业务逻辑
        .add_bui_action_handler("login.sure", handle_login_sure)
        .add_bui_action_handler("login.register", handle_login_register)
        .add_bui_action_handler("register.cancel", handle_register_cancel)
        // ...
        .run();
}

// 按钮"注册" → 切换到注册面板
fn handle_login_register(world: &mut World, _event: &BuiActionTriggered) {
    world.resource_mut::<BuiPanelSwitch>().show("register");
}

// 按钮"登录" → 校验 + 切换到选服面板
fn handle_login_sure(world: &mut World, _event: &BuiActionTriggered) {
    let username = read_binding(world, "login.username");
    if username.len() <= 6 {
        show_tip(world, "账号必须大于6位");
        return;
    }
    world.resource_mut::<BuiPanelSwitch>().show("server_select");
}
```

**对照 Unity 的 UIManager：**

| Unity C# | Rust main.rs |
|-----------|-------------|
| `Resources.Load("UI/LoginPanel")` | `.register_bui_panel("login", "...ir.json")` |
| `UIManager.ShowPanel<RegisterPanel>()` | `BuiPanelSwitch::show("register")` |
| `UIManager.HidePanel<LoginPanel>()` | 插件自动 despawn 旧面板 |
| `btnSure.onClick.AddListener(...)` | `.add_bui_action_handler("login.sure", ...)` |
| `inputUN.text` | `BuiStateStore` 读取 |
| `togPW.isOn = false` | `BuiStateSet(Bool(false))` 写入 |

## 运行

```bash
# 从 bevy workspace 根目录运行
cargo run -p bevy_ai_ui_parser --example login_scene
```

## 交互流程

```
启动 → 登录页（AiUiPlugin 在 Startup 加载 index.ir.json）
  │
  ├── 点"注册" → BuiPanelSwitch::show("register") → 注册页
  │     ├── 点"取消" → show("login") → 回登录页
  │     └── 注册成功 → show("login") → 回登录页
  │
  ├── 点"登录"（校验失败）→ 显示提示框
  │     └── 点"确定" → 隐藏提示框（DSL set-visible）
  │
  └── 点"登录"（校验成功）→ show("server_select") → 选服页
        │
        ├── 点"返回" → show("login") → 回登录页
        │
        └── 选服页 → show("server_list") → 服务器列表页
              ├── 点"换服" → show("server_select") → 回选服页
              ├── 点"返回" → show("login") → 回登录页
              └── 点"开始游戏" → AppExit（演示结束）
```

## 数据流

```
用户输入 → BuiStateStore（插件自动同步）→ main.rs 读取
main.rs 写入 → BuiStateSet → 插件自动同步 → UI 更新
main.rs 切换 → BuiPanelSwitch::show() → 插件加载新 IR JSON
```

## 已知限制

- **选服页服务器列表为空**：浏览器 JS 动态创建服务器项的逻辑被编译时丢弃。
  需要插件支持 ListView/Repeater 机制（类似 Unity 的 `ListView` + `makeItem`/`bindItem`）
  才能在 Bevy 运行时动态填充。
- **Toggle 视觉**：checkbox 的 `background-image` 在 Bevy 中由 toggle 节点的
  `image_config` 承载，`::checked` 伪类的切换需要通过 `BuiStateSet(Bool)` 驱动。

## 设计原则

1. **HTML 是源码，不被修改**——编译器只读 HTML 输出 IR JSON
2. **IR JSON = prefab**——插件负责加载/卸载，游戏代码不碰
3. **main.rs = Scripts/**——只有业务逻辑，没有 UI 加载代码
4. **浏览器 JS = 原型**——验证交互设计，编译时丢弃，Bevy 用 ECS handler 替代

## HTML 交互声明优先级

HTML 作者（或 AI）写交互时，必须按以下优先级：

```
1. 能用 DSL 声明的 → 用 data-binding / data-action / data-bui-actions
   → 编译进 IR JSON，Bevy 自动执行，不需要写 Rust 代码

2. DSL 处理不了的 → 用 JS（仅浏览器预览）
   → 编译时丢弃，Bevy 里需要手动写 Rust ECS handler 替代
```

### DSL 能处理的交互（编译后自动工作）

| 交互类型 | DSL 写法 | 举例 |
|---------|---------|------|
| 显示/隐藏 | `set-visible` in DSL | `{ "op": "set-visible", "target": "tip_panel", "value": "hidden" }` |
| 改文字 | `data-binding` + `BuiStateSet` | `<span data-binding="tip.info">默认文字</span>` |
| 切换选中 | `data-binding` (Bool) | `<input type="checkbox" data-binding="login.rememberPw">` |
| 延时执行 | `delay` in DSL | `{ "op": "delay", "ms": 900 }` |
| 设置图片 | `set-image` in DSL | `{ "op": "set-image", "target": "icon", "value": "Asset/new.png" }` |

### DSL 处理不了的交互（需要 Rust handler）

| 交互类型 | 为什么 DSL 不行 | Rust 怎么做 |
|---------|---------------|------------|
| 页面跳转 | 需要知道目标面板名 | `BuiPanelSwitch::show("register")` |
| 输入校验 | 需要条件判断 | `if username.len() <= 6 { ... }` |
| 数据读写 | 需要文件/网络 IO | `std::fs::read_to_string("ServerInfo.json")` |
| 联动逻辑 | 需要跨节点条件 | 取消记住密码 → `BuiStateSet(Bool(false))` 推送 autoLogin |
| 计算逻辑 | 需要数学运算 | 服务器分组、区间计算 |

### 关键规则

- **文字内容必须用 `data-binding`**，不能只靠 JS `element.textContent = ...`
  - 错误：`<div id="tip"></div>` + JS `tip.textContent = msg`
  - 正确：`<span data-binding="tip.info">默认文字</span>` + `BuiStateSet(Text(msg))`
- **可点击区域必须挂 `data-action`**，不能靠浏览器原生行为（如 `<label>` 包裹）
  - 错误：`<label><input type="checkbox" data-action-change="..."></label>`（点 label 不触发）
  - 正确：把 `data-action-change` 放在用户实际点击的元素上
- **JS 只用于浏览器预览**，编译时会被丢弃，不能依赖 JS 实现核心交互
