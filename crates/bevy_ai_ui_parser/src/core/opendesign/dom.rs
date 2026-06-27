pub(crate) fn has_class(node: roxmltree::Node<'_, '_>, class_name: &str) -> bool {
    node.is_element()
        && node
            .attribute("class")
            .is_some_and(|classes| classes.split_whitespace().any(|class| class == class_name))
}

pub(crate) fn first_text_by_class(
    node: roxmltree::Node<'_, '_>,
    class_name: &str,
) -> Option<String> {
    node.descendants()
        .find(|candidate| has_class(*candidate, class_name))
        .map(node_text)
        .filter(|text| !text.is_empty())
}

pub(crate) fn node_text(node: roxmltree::Node<'_, '_>) -> String {
    node.children()
        .filter_map(|candidate| candidate.text())
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}
