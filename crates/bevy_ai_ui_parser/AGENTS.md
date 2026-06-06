# AGENTS.md — bevy_ai_ui_parser 智能代理工作指南

## 项目概述
bevy_ai_ui_parser 是 Bevy 引擎的 JSON 驱动 UI 解析插件，负责将 JSON/HTML 描述的 UI 结构转换为 Bevy Entity 树。BUI 2.x 格式已被完全消除，3.0-IR (BuiDocument/BuiNode) 是唯一格式。

## 构建与测试命令
```bash
cargo test -p bevy_ai_ui_parser          # 运行全部测试
cargo build -p bevy_ai_ui_parser         # 编译 crate
cargo build --examples                   # 编译所有 example
```

## 代码树结构 (迁移完成后)
```
src/core/
  model/          — 数据模型层
    ir.rs          — BuiDocument/BuiNode/BuiNodeType/BuiLayout/BuiStyle/BuiContent/BuiSemantics 等全部定义
    style.rs       — BuiStyles 字段定义
    visual.rs      — BuiVisuals/BuiTextConfig/BuiImageConfig/BuiBoxShadowConfig 等
    mod.rs          — re-exports
  parse/           — 解析层
    ir.rs           — parse_bui_document (唯一解析入口, version "3.0-ir")
    validate/       — 验证子模块
      entry.rs      — validate_bui_document / validate_bui_json_str
      node.rs       — validate_bui_node
      semantics.rs  — 验证 semantics 字段
      styles.rs     — 验证 BuiStyles 字段
      visuals.rs    — 验证 BuiVisuals 字段
  opendesign/      — OpenDesign HTML→BUI 编译管道
    cases.rs, build.rs, preset.rs, html.rs
    html/village.rs, generic/, hero/, hero/effects/, svg/
  style/            — CSS 样式层
    css_apply/, css_effects/, css_gradients/, css_parser/
  runtime/          — 运行时层
    spawn.rs, node_spawn/, plugin.rs, text.rs
  interaction/      — 交互层
    components.rs, list.rs
  support/          — 辅助工具层
    tree.rs         — find_bui_node_ref/mut
  api.rs            — 公共 API 导出
```

## 数据模型 (BuiDocument — 3.0-IR 格式)
```
BuiDocument { version, scene_name, imports, state_model, resources, root: BuiNode }
BuiNode     { id, kind, markers, classes, actions, bindings, layout, style, content, semantics, state_visuals, children }
BuiLayout   { styles: BuiStyles }
BuiStyle    { visuals: BuiVisuals }
BuiContent  { text?: BuiTextConfig, image?: BuiImageConfig }
BuiSemantics { tab_group_name, tab_binding_source, tab_value, progress_binding_source, progress_fill, list_binding_source }
BuiNodeType enum { Node, Text, TextInput, Toggle, Button, Image } — 运行时辅助, 从 kind 字段映射
```

## 公共 API
```rust
// api.rs 导出:
pub fn opendesign_html_to_bui_document(html: &str) -> Result<BuiDocument, String>
pub fn opendesign_html_to_bui_json_str(html: &str) -> Result<String, String>
pub fn opendesign_html_file_to_bui_document(path: &str) -> Result<BuiDocument, String>
pub fn opendesign_html_file_to_bui_json(path: &str) -> Result<String, String>
pub fn validate_bui_json_str(json: &str) -> Result<(), String>
pub fn validate_bui_json_file(path: &str) -> Result<(), String>
```

## 迁移完成状态
所有 8 个阶段已完成:
- Phase 1: BuiNode 基础设施升级 (node_type(), 便利构造函数, is_empty(), ensure_state_visual())
- Phase 2-6: 合并迁移所有代码 (OpenDesign + Style + Runtime + Parse + Support + Tests)
- Phase 7: 删除旧代码 (bui.rs) + compat 方法 + 清理 unused imports + fixture 转换
- Phase 8: 全局重命名 (BuiIrNode→BuiNode, BuiIrDocument→BuiDocument 等)
- 77 个测试全部通过，编译零警告

## Fixture 转换完成
- 24 个 uiParse_TestSet .json → .ir.json (转换脚本: tools/convert_2x_to_ir.py)
- 5 个 unsupported .json → .ir.json
- 3 个 OpenDesign .json 已删除 (已有 .ir.json)
- 3 个 _json.rs example 已删除 (与 IR 版重复)
- 24 个 .rs example 文件路径已更新 (.json → .ir.json)

## 关键决策记录
- 激进路径: 完全删除 2.x 格式，BuiDocument/BuiNode 现在就是 3.0-IR 格式
- BuiNodeType enum 保留为运行时辅助 (从 kind 字段映射)，不作为 serde 类型
- node_type() 方法提供 enum 匹配，kind 字段用于字符串比较
- 新 BuiNode 结构与旧 2.x 完全不同 (有 layout/style/content/semantics 子结构)