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

pub(crate) fn find_bui_parent_id(node: &BuiNode, target_id: &str) -> Option<String> {
    for child in &node.children {
        if child.id == target_id {
            return Some(node.id.clone());
        }

        if let Some(found) = find_bui_parent_id(child, target_id) {
            return Some(found);
        }
    }

    None
}

pub(crate) fn remove_bui_node(node: &mut BuiNode, target_id: &str) -> Option<BuiNode> {
    if let Some(index) = node.children.iter().position(|child| child.id == target_id) {
        return Some(node.children.remove(index));
    }

    for child in &mut node.children {
        if let Some(removed) = remove_bui_node(child, target_id) {
            return Some(removed);
        }
    }

    None
}

pub(crate) fn collect_bui_subtree_ids(node: &BuiNode, ids: &mut Vec<String>) {
    ids.push(node.id.clone());
    for child in &node.children {
        collect_bui_subtree_ids(child, ids);
    }
}

#[cfg(test)]
mod tests {
    use crate::core::model::bui_node;

    use super::{collect_bui_subtree_ids, find_bui_parent_id, remove_bui_node};

    #[test]
    fn remove_bui_node_returns_removed_subtree() {
        let mut root = bui_node("root", "node");
        let mut parent = bui_node("parent", "node");
        let child = bui_node("child", "text");
        parent.children.push(child);
        root.children.push(parent);

        let removed = remove_bui_node(&mut root, "parent").expect("subtree should exist");

        assert_eq!(removed.id, "parent");
        assert_eq!(removed.children.len(), 1);
        assert!(root.children.is_empty());
    }

    #[test]
    fn collect_bui_subtree_ids_walks_all_descendants() {
        let mut root = bui_node("root", "node");
        let mut branch = bui_node("branch", "node");
        branch.children.push(bui_node("leaf", "text"));
        root.children.push(branch);

        let mut ids = Vec::new();
        collect_bui_subtree_ids(&root, &mut ids);

        assert_eq!(ids, vec!["root", "branch", "leaf"]);
    }

    #[test]
    fn find_bui_parent_id_returns_immediate_parent() {
        let mut root = bui_node("root", "node");
        let mut parent = bui_node("parent", "node");
        parent.children.push(bui_node("child", "text"));
        root.children.push(parent);

        assert_eq!(
            find_bui_parent_id(&root, "child").as_deref(),
            Some("parent")
        );
        assert_eq!(find_bui_parent_id(&root, "missing"), None);
    }
}
