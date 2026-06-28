# opus48 HTML→IR 渲染不一致问题修复文档

## 概述

对 opus48 用例仓中 **16 个项目、76 个页面** 的 HTML 源码与编译后 IR JSON 进行全量对比，发现 **7 个根因**，覆盖全部表面不一致问题。

| 根因编号 | 简述 | 影响页面数 | 严重程度 |
|---------|------|-----------|---------|
| R1 | Preset 无条件覆盖 CSS 值 | 76/76 | CRITICAL |
| R2 | SVG 元素跳过 CSS 样式应用 | ~6 | MEDIUM |
| R3 | `rgba()`/`rgb()` 颜色在部分路径无法解析 | ~5 | HIGH |
| R4 | `border-radius` 四值语法只取首值 | 2 | HIGH |
| R5 | `text-shadow` 数据模型缺少 `blur_radius` | 8 | HIGH |
| R6 | HTML 根节点查找逻辑导致兄弟元素丢失 | 1 | CRITICAL |
| R7 | CSS 继承/级联优先级问题 | 2 | HIGH |

---

## R1：Preset 无条件覆盖 CSS 值

**严重程度**：CRITICAL
**影响范围**：全部 76 页

### 现象

1. 全部 76 页根节点背景色变为硬编码 `#3B281862`（半透明棕色），而非各页面 CSS 定义的 `--bg0`
2. ~50 页 `justify_content` 被强制为 `center`（HTML 未设置时应为 `flex-start`，或显式声明为 `space-between`）
3. ~50 页 `align_items` 被强制为 `center`（HTML 未设置时应为 `stretch`）

### 根因定位

**文件 1**：`src/core/opendesign/preset.rs:30-36`

```rust
OpenDesignPreset::OverlayRoot => {
    node.layout.styles.width = Some("100%".to_string());
    node.layout.styles.height = Some("100%".to_string());
    node.layout.styles.justify_content = Some("center".to_string());   // 硬编码
    node.layout.styles.align_items = Some("center".to_string());       // 硬编码
    node.style.visuals.background_color = Some("#3B281862".to_string()); // 硬编码
}
```

**文件 2**：`src/core/opendesign/generic/document.rs:28-36`

```rust
let mut source_root = generic_element_node("overlay_root", "node", stylesheet, overlay);
// ↑ 先解析 CSS，正确设置了 background_color / justify_content / align_items
apply_opendesign_preset(&mut source_root, OpenDesignPreset::OverlayRoot);
// ↑ 后执行，无条件 = Some(...) 覆盖掉 CSS 的值
```

**对比正确路径**：`src/core/opendesign/html/village.rs:74-76` 中 preset 在 CSS 之前执行，CSS 可以正确覆盖 preset 默认值。

### 修复方案

**方案 A（推荐）**：将 preset 中的赋值改为条件填充，仅在 CSS 未设置时才使用默认值：

```rust
OpenDesignPreset::OverlayRoot => {
    node.layout.styles.width.get_or_insert(|| "100%".to_string());
    node.layout.styles.height.get_or_insert(|| "100%".to_string());
    // 不强制设置 justify_content / align_items / background_color
    // 让 CSS 值优先
}
```

**方案 B**：调换执行顺序，先 `apply_opendesign_preset` 再 `apply_opendesign_styles`，与 `village.rs` 保持一致。

### 受影响的页面示例

| 项目 | 页面 | HTML 期望 | IR 实际 |
|-----|------|----------|--------|
| action_arena | index | bg `#0C111B`, justify 默认 | bg `#3B281862`, justify `center` |
| card_clash | battle | bg `#16102E`, justify 默认 | bg `#3B281862`, justify `center` |
| pixel_jump | hud | bg `#0B1026`, justify `space-between` | bg `#3B281862`, justify `center` |
| candy_match | index | bg `#3A1858` | bg `#3B281862` |
| mini_mix | index | bg `#FFF6EE` | bg `#3B281862` |

---

## R2：SVG 元素跳过 CSS 样式应用

**严重程度**：MEDIUM
**影响范围**：~6 页（pixel_jump、idle_tycoon 中 16 个 SVG 节点）

### 现象

HTML 中 `<svg>` 元素上的 CSS `margin-right`、`margin-bottom` 等间距属性在 IR JSON 中丢失，导致 SVG 图标与相邻元素之间缺少间距。

### 根因定位

**文件**：`src/core/opendesign/generic/tree.rs:56-73`

```rust
if is_svg_tag(child.tag_name().name()) {
    if child.tag_name().name() == "svg" {
        let image_node = svg_image_node(parent, child, index, &png_path);
        parent.children.push(image_node);  // 直接推入，未应用 CSS 样式
    }
    continue;  // 跳过了下面的 generic_element_node() 路径
}
// 非 SVG 元素走这条路径，会调用 apply_opendesign_styles()：
let mut child_node = generic_element_node(&id, kind, stylesheet, child);
```

`svg_image_node()`（`src/core/opendesign/svg/render.rs:12-33`）只设置了 width/height/image，**从未调用 `apply_opendesign_styles()`**。

### 修复方案

在 `svg_image_node()` 返回后、push 到 parent 之前，调用 CSS 样式应用：

```rust
let mut image_node = svg_image_node(parent, child, index, &png_path);
apply_opendesign_styles(stylesheet, &mut image_node, child);  // 新增
parent.children.push(image_node);
```

### 受影响的页面示例

| 项目 | 页面 | 丢失属性的元素 | HTML 期望 |
|-----|------|--------------|----------|
| pixel_jump | index | `.coin-ico`, `.hi-ico` SVG | `margin-right: 16px` |
| pixel_jump | levels | `.prog-ico`, 6 个 `.star`, 2 个 `.gem-ico` SVG | `margin-right: 6-14px` |
| pixel_jump | hud | `.heart`, `.coin-hud-ico`, `.timer-ico`, `.gem-hud-ico` SVG | `margin-right: 14-18px` |
| idle_tycoon | index | `.res-ico`, `.main-act-ico` SVG | `margin-right: 14-20px` |
| idle_tycoon | upgrade | `.up-btn-coin` SVG | `margin-right: 10px` |
| idle_tycoon | gacha | `.pull-gem` SVG, `.drop-art` SVG | `margin-right: 8px`, `margin-bottom: 12px` |

---

## R3：`rgba()`/`rgb()` 颜色在部分路径无法解析

**严重程度**：HIGH
**影响范围**：~5 页

### 现象

使用 `rgba()` 或 `rgb()` 函数定义的背景色在 IR 中丢失，节点无 `background_color`。

### 根因定位

**文件**：`src/core/style/css_values/background.rs:47-78`（`css_simple_color` 函数）

`css_simple_color()` 只处理：
- hex 颜色（`#RGB`, `#RRGGBB`, `#RRGGBBAA`）
- oklch 颜色
- `color-mix()` 函数
- `transparent` 关键字
- 命名颜色（`black`, `white`, `red` 等）

**不支持** `rgba()` / `rgb()` CSS 函数。

而 `color.rs` 中的 `css_embedded_rgb_function_to_hex`（line 57）是 `fn`（私有），`background.rs` 无法访问：

```rust
// color.rs line 57 — 私有函数
fn css_embedded_rgb_function_to_hex(value: &str) -> Option<String> {
```

**失败链路**：

```
background-color: rgba(90,34,48,0.18)
  → css_simple_color("rgba(90,34,48,0.18)")
  → 所有分支都不匹配
  → 返回 None
  → background_color 未设置
```

### 修复方案

1. 将 `color.rs` 中的 `css_embedded_rgb_function_to_hex` 改为 `pub(crate)`
2. 在 `background.rs` 的 `css_simple_color()` 中增加对 `rgba()` / `rgb()` 的解析分支

```rust
pub(crate) fn css_simple_color(value: &str) -> Option<String> {
    let value = value.trim();
    if let Some(color) = css_color_mix_with_transparency(value) { return Some(color); }
    if let Some(color) = oklch_to_hex(value) { return Some(color); }
    if let Some(color) = css_embedded_oklch_color(value) { return Some(color); }
    if let Some(color) = css_embedded_rgb_function_to_hex(value) { return Some(color); } // 新增
    if value == "transparent" { return Some("transparent".to_string()); }
    if is_hex_color(value) { return Some(value.to_string()); }
    // ...
}
```

### 受影响的页面示例

| 项目 | 页面 | 元素 | HTML 期望 | IR 实际 |
|-----|------|------|----------|--------|
| card_clash | battle | `.field-enemy` | `rgba(90,34,48,0.18)` 半透明红底 | 无 background_color |
| card_clash | battle | `.field-self` | `rgba(31,116,140,0.16)` 半透明蓝底 | 无 background_color |
| action_arena | settings | `row_border_bottom` (×3) | `#2E3D58` 分割线 | 无 background_color |
| card_clash | settings | `row_border_bottom` (×2) | `#2A1F58` 分割线 | 无 background_color |

---

## R4：`border-radius` 四值语法只取首值

**严重程度**：HIGH
**影响范围**：2 页（visual_novel/dialogue）

### 现象

CSS `border-radius` 的四值语法（如 `220px 220px 40px 40px`）被截断为单一值 `220px`，导致圆角形状错误。

### 根因定位

**文件 1**：`src/core/style/css_apply/declarations.rs:258`

```rust
"border-radius" => {
    if let Some(radius) = css_first_size(&value) {  // 只取第一个 size
        bui_node.style.visuals.border_radius = Some(radius);
    }
}
```

**文件 2**：`src/core/style/css_sizing.rs:48-55`

```rust
pub(crate) fn css_first_size(value: &str) -> Option<String> {
    // ...
    css_size_tokens(value)
        .into_iter()
        .find_map(|part| css_length_to_bui_val(&part))  // find_map 只返回第一个匹配
}
```

**文件 3**：`src/core/style/css_parser/layout.rs:218-219`

```rust
pub(crate) fn parse_border_radius(value: &str) -> Result<BorderRadius, String> {
    Ok(BorderRadius::all(parse_val(value)?))  // 同一值应用到四角
}
```

### 修复方案

解析 4 值语法，按 CSS 规范映射到四角：

- 1 值：四角相同
- 2 值：top-left + bottom-right, top-right + bottom-left
- 3 值：top-left, top-right + bottom-left, bottom-right
- 4 值：top-left, top-right, bottom-right, bottom-left

```rust
"border-radius" => {
    let radii = css_all_sizes(&value);  // 解析所有值
    match radii.len() {
        1 => set_all_corners(radii[0]),
        2 => { set_corner_tl_br(radii[0]); set_corner_tr_bl(radii[1]); }
        3 => { set_corner_tl(radii[0]); set_corner_tr_bl(radii[1]); set_corner_br(radii[2]); }
        4 => { set_all_corners_individually(&radii); }
        _ => {}
    }
}
```

### 受影响的页面示例

| 项目 | 页面 | 元素 | HTML 期望 | IR 实际 |
|-----|------|------|----------|--------|
| visual_novel | dialogue | `.sprite` | `border-radius: 220px 220px 40px 40px` | `border_radius: "220px"` |
| visual_novel | dialogue | `.name-tag` | `border-radius: 24px 24px 0 0` | `border_radius: "24px"` |

---

## R5：`text-shadow` 数据模型缺少 `blur_radius` 字段

**严重程度**：HIGH
**影响范围**：8 页（visual_novel 全 6 页 + horror_survival 2 页）

### 现象

CSS `text-shadow: 0 0 32px rgba(232,199,122,0.55)` 中的 blur radius（`32px`）丢失，部分情况下 color 也丢失。

### 根因定位

**文件 1**：`src/core/model/visual.rs:79-86`

```rust
pub struct BuiTextShadowConfig {
    pub offset_x: Option<f32>,
    pub offset_y: Option<f32>,
    pub color: Option<String>,
    // 缺少 blur_radius 字段
}
```

**文件 2**：`src/core/style/css_effects/shadow.rs:9-38`

```rust
pub(crate) fn css_text_shadow(value: &str) -> Option<BuiTextShadowConfig> {
    // ...
    for token in css_size_tokens(&layer) {
        if let Some(number) = css_text_shadow_length(&token) {
            if offset_x.is_none() {
                offset_x = Some(number);
            } else if offset_y.is_none() {
                offset_y = Some(number);
                break;  // 找到 offset_y 后立即退出，不读取 blur_radius
            }
        }
    }
    // ...
}
```

对于 `text-shadow: 0 0 32px rgba(...)`：
- `offset_x = 0` ✓
- `offset_y = 0` ✓
- `blur_radius = 32px` ✗（循环已 break）
- `color` — 取决于 color token 的位置，有时能捕获有时不能

### 修复方案

1. 在 `BuiTextShadowConfig` 中增加 `blur_radius: Option<f32>` 字段
2. 修改解析循环，在找到 `offset_y` 后继续读取第三个长度值作为 `blur_radius`

```rust
pub struct BuiTextShadowConfig {
    pub offset_x: Option<f32>,
    pub offset_y: Option<f32>,
    pub blur_radius: Option<f32>,  // 新增
    pub color: Option<String>,
}
```

### 受影响的页面示例

| 项目 | 页面 | 元素 | HTML 期望 | IR 实际 |
|-----|------|------|----------|--------|
| visual_novel | index | `.title-main` | `text-shadow: 0 0 32px rgba(232,199,122,0.55)` | `{offset_x:0, offset_y:0}` 无 blur 无 color |
| visual_novel | choice | `.header-title` | `text-shadow: 0 0 24px rgba(232,199,122,0.4)` | `{offset_x:0, offset_y:0}` |
| visual_novel | gallery | `.header-title` | `text-shadow: 0 0 20px rgba(232,199,122,0.35)` | `{offset_x:0, offset_y:0}` |
| visual_novel | saves | `.header-title` | 同上 | 同上 |
| visual_novel | settings | `.header-title` | 同上 | 同上 |
| horror_survival | index | `.title-main` | `text-shadow: 0 0 24px #5FD3C055` | color 保留但 blur 丢失 |
| horror_survival | result | `.verdict-title` | `text-shadow: 0 0 30px #C9A23A44` | color 保留但 blur 丢失 |

---

## R6：HTML 根节点查找逻辑导致兄弟元素丢失

**严重程度**：CRITICAL
**影响范围**：1 页（visual_novel/index）

### 现象

visual_novel/index 页面的 `.kv` 背景视觉层（5 个节点）和 `.star` 装饰元素（4 个节点）完全从 IR 中消失。

### 根因定位

**文件**：`src/core/opendesign/html.rs:274-299`

```rust
let overlay = parsed
    .descendants()
    .find(|node| has_class(*node, "overlay"));  // 优先找 .overlay

let root = overlay  // .overlay 存在就用它做根
    .or_else(|| has_class(*node, "game-stage"))
    // ...
    .or_else(|| has_class(*node, "page"));  // .page 是最后兜底
```

visual_novel/index.html 的 DOM 结构：

```html
<div class="page">
  <div class="kv">...</div>        <!-- .overlay 的兄弟 -->
  <div class="kv-glow">...</div>   <!-- .overlay 的兄弟 -->
  <div class="star-a">...</div>    <!-- .overlay 的兄弟 -->
  <div class="overlay">...</div>   <!-- 被选为根节点 -->
</div>
```

代码找到 `.overlay` 作为根节点，其兄弟元素 `.kv`、`.star-*` 被完全忽略。

### 修复方案

当 `.page` 是 `.overlay` 的父级时，应以 `.page` 为根节点：

```rust
let root = parsed
    .descendants()
    .find(|node| has_class(*node, "page"))  // 优先找 .page
    .or_else(|| parsed.descendants().find(|node| has_class(*node, "overlay")))
    // ...
```

或在找到 `.overlay` 后检查其父元素是否为 `.page`，如果是则使用 `.page`。

### 受影响的节点

| 缺失节点 ID | HTML class | 作用 |
|------------|-----------|------|
| `kv` | `.kv` | 主视觉背景容器（1080×1180, `#241844`） |
| `kv_glow` | `.kv-glow` | 光晕效果 |
| `kv_moon` | `.kv-moon` | 月亮图形 |
| `kv_silhouette` | `.kv-silhouette` | 剪影 |
| `kv_veil` | `.kv-veil` | 遮罩层 |
| `star_a` ~ `star_d` | `.star-a` ~ `.star-d` | 4 个浮动星星装饰 SVG |

---

## R7：CSS 继承/级联优先级问题

**严重程度**：HIGH
**影响范围**：2 页

### 现象

1. visual_novel/dialogue 中 `.line-emph` 的 `line-height` 应为继承自 `.line` 的 `64px`，实际为 `1.2`
2. candy_match/game 中 `.goal-tx-done` 的 `font-size` 应为继承自 `.goal-tx` 的 `42px`，实际为默认值 `16.0`

### 根因定位

**文件**：`src/core/opendesign/generic/text.rs:34-87`

`apply_inherited_text_styles` 在祖先遍历中应用可继承属性。但 yoga-reset 的通配选择器：

```css
*, *::before, *::after {
  line-height: 1.2;
}
```

匹配所有元素，其值覆盖了通过 class 继承的值。

**问题本质**：CSS 级联中，直接继承的 class 值（如 `.line { line-height: 64px }`）应优先于通配选择器的默认值（`* { line-height: 1.2 }`），但当前实现未区分优先级。

对于 `font-size` 问题，当元素有多个 class 且其中一个 class 定义了 `font-size`，但继承链断裂时，回退到 text 节点的默认值 `16.0`。

### 修复方案

在继承链中引入优先级区分：
- 直接继承值（来自父元素的 class 规则）优先级高于通配选择器默认值
- 或在应用继承时，检查值是否来自通配选择器，如果是则不覆盖已有的继承值

### 受影响的页面示例

| 项目 | 页面 | 元素 | HTML 期望 | IR 实际 |
|-----|------|------|----------|--------|
| visual_novel | dialogue | `.line-emph` text | `line-height: 64px`（继承自 `.line`） | `line_height: "1.2"`（通配选择器） |
| candy_match | game | `.goal-tx-done` text | `font-size: 42px`（继承自 `.goal-tx`） | `font_size: 16.0`（默认值） |

---

## 修复优先级建议

| 优先级 | 根因 | 理由 |
|-------|------|------|
| P0 | R1 | 影响全部 76 页，修复后消除最大面积的不一致 |
| P0 | R6 | 整个视觉层丢失，页面完全不可用 |
| P1 | R3 | `rgba()` 是常见 CSS 写法，影响多个项目的背景色和分割线 |
| P1 | R4 | `border-radius` 四值语法是标准 CSS，影响视觉形状 |
| P1 | R5 | `text-shadow` blur 是常见视觉效果 |
| P1 | R7 | CSS 继承问题影响文字排版 |
| P2 | R2 | SVG margin 丢失影响间距，但不破坏布局结构 |

---

## 验证方法

修复后，对每个项目运行 `cargo run --example <page_name>` 并与浏览器中打开对应 HTML 文件进行视觉对比。

也可使用自动截图功能批量验证：

```bash
BUI_SCREENSHOT_PATH=/tmp/screenshots/<project>_<page>.png \
cargo run --example <page_name>
```
