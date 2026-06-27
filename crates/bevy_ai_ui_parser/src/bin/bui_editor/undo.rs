use std::collections::VecDeque;

use bevy_ai_ui_parser::BuiDocument;

pub trait BuiEditCommand: Send + Sync {
    fn apply(&self, doc: &mut BuiDocument);
    fn undo(&self, doc: &mut BuiDocument);
    fn description(&self) -> &str;
}

pub struct UndoStack {
    undo: VecDeque<Box<dyn BuiEditCommand>>,
    redo: VecDeque<Box<dyn BuiEditCommand>>,
    max_size: usize,
}

impl Default for UndoStack {
    fn default() -> Self {
        Self {
            undo: VecDeque::new(),
            redo: VecDeque::new(),
            max_size: 100,
        }
    }
}

impl UndoStack {
    pub fn push(&mut self, command: Box<dyn BuiEditCommand>) {
        if self.undo.len() >= self.max_size {
            self.undo.pop_front();
        }
        self.undo.push_back(command);
        self.redo.clear();
    }

    pub fn undo(&mut self, doc: &mut BuiDocument) -> bool {
        if let Some(command) = self.undo.pop_back() {
            command.undo(doc);
            self.redo.push_back(command);
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self, doc: &mut BuiDocument) -> bool {
        if let Some(command) = self.redo.pop_back() {
            command.apply(doc);
            self.undo.push_back(command);
            true
        } else {
            false
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }
}

pub mod commands {
    use super::BuiEditCommand;
    use bevy_ai_ui_parser::{BuiDocument, BuiNode};

    pub struct SetStyleField {
        pub node_id: String,
        pub field_name: String,
        pub old_value: Option<String>,
        pub new_value: Option<String>,
    }

    impl BuiEditCommand for SetStyleField {
        fn apply(&self, doc: &mut BuiDocument) {
            if let Some(node) = find_node_mut(&mut doc.root, &self.node_id) {
                set_field(node, &self.field_name, self.new_value.clone());
            }
        }

        fn undo(&self, doc: &mut BuiDocument) {
            if let Some(node) = find_node_mut(&mut doc.root, &self.node_id) {
                set_field(node, &self.field_name, self.old_value.clone());
            }
        }

        fn description(&self) -> &str {
            "Set style field"
        }
    }

    pub struct SetTextField {
        pub node_id: String,
        pub old_content: String,
        pub new_content: String,
    }

    impl BuiEditCommand for SetTextField {
        fn apply(&self, doc: &mut BuiDocument) {
            if let Some(node) = find_node_mut(&mut doc.root, &self.node_id) {
                if let Some(text) = &mut node.content.text {
                    text.content = self.new_content.clone();
                }
            }
        }

        fn undo(&self, doc: &mut BuiDocument) {
            if let Some(node) = find_node_mut(&mut doc.root, &self.node_id) {
                if let Some(text) = &mut node.content.text {
                    text.content = self.old_content.clone();
                }
            }
        }

        fn description(&self) -> &str {
            "Set text content"
        }
    }

    pub struct SetTextFontSize {
        pub node_id: String,
        pub old_value: f32,
        pub new_value: f32,
    }

    impl BuiEditCommand for SetTextFontSize {
        fn apply(&self, doc: &mut BuiDocument) {
            if let Some(node) = find_node_mut(&mut doc.root, &self.node_id) {
                if let Some(text) = &mut node.content.text {
                    text.font_size = self.new_value;
                }
            }
        }

        fn undo(&self, doc: &mut BuiDocument) {
            if let Some(node) = find_node_mut(&mut doc.root, &self.node_id) {
                if let Some(text) = &mut node.content.text {
                    text.font_size = self.old_value;
                }
            }
        }

        fn description(&self) -> &str {
            "Set font size"
        }
    }

    pub struct SetTextColor {
        pub node_id: String,
        pub old_value: String,
        pub new_value: String,
    }

    impl BuiEditCommand for SetTextColor {
        fn apply(&self, doc: &mut BuiDocument) {
            if let Some(node) = find_node_mut(&mut doc.root, &self.node_id) {
                if let Some(text) = &mut node.content.text {
                    text.font_color = self.new_value.clone();
                }
            }
        }

        fn undo(&self, doc: &mut BuiDocument) {
            if let Some(node) = find_node_mut(&mut doc.root, &self.node_id) {
                if let Some(text) = &mut node.content.text {
                    text.font_color = self.old_value.clone();
                }
            }
        }

        fn description(&self) -> &str {
            "Set text color"
        }
    }

    pub struct DeleteNode {
        pub node_id: String,
        pub parent_id: String,
        pub deleted_node: BuiNode,
        pub index: usize,
    }

    impl BuiEditCommand for DeleteNode {
        fn apply(&self, doc: &mut BuiDocument) {
            if let Some(parent) = find_node_mut(&mut doc.root, &self.parent_id) {
                parent.children.retain(|c| c.id != self.node_id);
            }
        }

        fn undo(&self, doc: &mut BuiDocument) {
            if let Some(parent) = find_node_mut(&mut doc.root, &self.parent_id) {
                let index = self.index.min(parent.children.len());
                parent.children.insert(index, self.deleted_node.clone());
            }
        }

        fn description(&self) -> &str {
            "Delete node"
        }
    }

    pub struct AddNode {
        pub parent_id: String,
        pub node: BuiNode,
    }

    impl BuiEditCommand for AddNode {
        fn apply(&self, doc: &mut BuiDocument) {
            if let Some(parent) = find_node_mut(&mut doc.root, &self.parent_id) {
                parent.children.push(self.node.clone());
            }
        }

        fn undo(&self, doc: &mut BuiDocument) {
            if let Some(parent) = find_node_mut(&mut doc.root, &self.parent_id) {
                parent.children.retain(|c| c.id != self.node.id);
            }
        }

        fn description(&self) -> &str {
            "Add node"
        }
    }

    pub struct MoveNode {
        pub node_id: String,
        pub old_left: Option<String>,
        pub old_top: Option<String>,
        pub new_left: Option<String>,
        pub new_top: Option<String>,
    }

    impl BuiEditCommand for MoveNode {
        fn apply(&self, doc: &mut BuiDocument) {
            if let Some(node) = find_node_mut(&mut doc.root, &self.node_id) {
                node.layout.styles.left = self.new_left.clone();
                node.layout.styles.top = self.new_top.clone();
            }
        }

        fn undo(&self, doc: &mut BuiDocument) {
            if let Some(node) = find_node_mut(&mut doc.root, &self.node_id) {
                node.layout.styles.left = self.old_left.clone();
                node.layout.styles.top = self.old_top.clone();
            }
        }

        fn description(&self) -> &str {
            "Move node"
        }
    }

    fn find_node_mut<'a>(root: &'a mut BuiNode, id: &str) -> Option<&'a mut BuiNode> {
        if root.id == id {
            return Some(root);
        }
        for child in &mut root.children {
            if let Some(found) = find_node_mut(child, id) {
                return Some(found);
            }
        }
        None
    }

    fn set_field(node: &mut BuiNode, field_name: &str, value: Option<String>) {
        let s = &mut node.layout.styles;
        match field_name {
            "display" => s.display = value,
            "position_type" | "position" => s.position_type = value,
            "width" => s.width = value,
            "height" => s.height = value,
            "min_width" => s.min_width = value,
            "min_height" => s.min_height = value,
            "max_width" => s.max_width = value,
            "max_height" => s.max_height = value,
            "box_sizing" => s.box_sizing = value,
            "aspect_ratio" => s.aspect_ratio = value,
            "left" => s.left = value,
            "top" => s.top = value,
            "right" => s.right = value,
            "bottom" => s.bottom = value,
            "z_index" => s.z_index = value,
            "global_z_index" => s.global_z_index = value,
            "flex_direction" => s.flex_direction = value,
            "flex_wrap" => s.flex_wrap = value,
            "flex_grow" => s.flex_grow = value,
            "flex_shrink" => s.flex_shrink = value,
            "flex_basis" => s.flex_basis = value,
            "justify_content" => s.justify_content = value,
            "align_items" => s.align_items = value,
            "align_self" => s.align_self = value,
            "align_content" => s.align_content = value,
            "row_gap" => s.row_gap = value,
            "column_gap" => s.column_gap = value,
            "margin" => s.margin = value,
            "margin_top" => s.margin_top = value,
            "margin_bottom" => s.margin_bottom = value,
            "margin_left" => s.margin_left = value,
            "margin_right" => s.margin_right = value,
            "padding" => s.padding = value,
            "padding_top" => s.padding_top = value,
            "padding_bottom" => s.padding_bottom = value,
            "padding_left" => s.padding_left = value,
            "padding_right" => s.padding_right = value,
            "overflow" => s.overflow = value,
            "visibility" => s.visibility = value,
            "grid_template_columns" => s.grid_template_columns = value,
            "grid_template_rows" => s.grid_template_rows = value,
            "grid_auto_columns" => s.grid_auto_columns = value,
            "grid_auto_rows" => s.grid_auto_rows = value,
            "grid_column" => s.grid_column = value,
            "grid_row" => s.grid_row = value,
            "background_color" => node.style.visuals.background_color = value,
            "border_color" => node.style.visuals.border_color = value,
            "border_width" => node.style.visuals.border_width = value,
            "border_radius" => node.style.visuals.border_radius = value,
            _ => {}
        }
    }
}
