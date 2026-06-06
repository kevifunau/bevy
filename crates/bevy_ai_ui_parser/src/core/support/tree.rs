use crate::core::model::BuiNode;

pub(crate) fn find_bui_node_ref<'a>(node: &'a BuiNode, id: &str) -> Option<&'a BuiNode> {
    if node.id == id {
        return Some(node);
    }

    for child in &node.children {
        if let Some(found) = find_bui_node_ref(child, id) {
            return Some(found);
        }
    }

    None
}

pub(crate) fn find_bui_node_mut<'a>(node: &'a mut BuiNode, id: &str) -> Option<&'a mut BuiNode> {
    if node.id == id {
        return Some(node);
    }

    for child in &mut node.children {
        if let Some(found) = find_bui_node_mut(child, id) {
            return Some(found);
        }
    }

    None
}
