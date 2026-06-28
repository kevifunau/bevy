use std::collections::HashSet;
use std::path::PathBuf;

use bevy::prelude::*;
use bevy_ai_ui_parser::BuiDocument;
use bevy_egui::egui;
use egui_dock::DockState;

use crate::dock_layout::EditorTab;
use crate::undo::UndoStack;

/// Items that can be dragged from the Library panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibraryItem {
    /// Basic node types
    Node,
    Text,
    Button,
    Image,
    TextInput,
    Toggle,
    Slider,
    /// Template compositions
    Row,
    Column,
    ButtonWithText,
}

impl LibraryItem {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Node => "Container",
            Self::Text => "Text",
            Self::Button => "Button",
            Self::Image => "Image",
            Self::TextInput => "Text Input",
            Self::Toggle => "Toggle",
            Self::Slider => "Slider",
            Self::Row => "Row (flex-direction: row)",
            Self::Column => "Column (flex-direction: column)",
            Self::ButtonWithText => "Button + Text",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Node => "\u{1F4E6}",
            Self::Text => "\u{1F4DD}",
            Self::Button => "\u{1F518}",
            Self::Image => "\u{1F5BC}",
            Self::TextInput => "\u{2328}",
            Self::Toggle => "\u{2705}",
            Self::Slider => "\u{1F3A8}",
            Self::Row => "\u{2194}",
            Self::Column => "\u{2195}",
            Self::ButtonWithText => "\u{1F518}\u{1F4DD}",
        }
    }
}

/// What the user wants to do after the "Save changes?" dialog is resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingAction {
    New,
    Open,
    CompileHtml,
    Quit,
}

#[derive(Resource)]
pub struct EditorState {
    pub ir_path: PathBuf,
    pub dirty: bool,
    pub selected_node_id: Option<String>,
    pub viewport_rect: egui::Rect,
    pub dock_state: DockState<EditorTab>,
    pub undo_stack: UndoStack,
    pub canvas_zoom: f32,
    pub canvas_pan: egui::Vec2,
    pub collapsed_nodes: HashSet<String>,
    pub dragging_library_item: Option<LibraryItem>,
    pub drag_hover_node_id: Option<String>,
    /// Non-empty when showing "Save changes?" dialog.
    pub pending_action: Option<PendingAction>,
}

impl EditorState {
    pub fn save(&self, doc: &BuiDocument) -> Result<(), String> {
        let json =
            serde_json::to_string_pretty(doc).map_err(|e| format!("Serialize error: {e}"))?;

        std::fs::write(&self.ir_path, json).map_err(|e| format!("Write error: {e}"))?;

        info!("Saved to {}", self.ir_path.display());
        Ok(())
    }

    pub fn toggle_collapse(&mut self, node_id: &str) {
        if self.collapsed_nodes.contains(node_id) {
            self.collapsed_nodes.remove(node_id);
        } else {
            self.collapsed_nodes.insert(node_id.to_string());
        }
    }

    pub fn is_collapsed(&self, node_id: &str) -> bool {
        self.collapsed_nodes.contains(node_id)
    }
}
