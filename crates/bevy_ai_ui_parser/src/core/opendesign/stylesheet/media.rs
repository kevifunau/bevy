use crate::core::{
    style::css_sizing::{css_first_size, css_size_to_px},
    support::viewport::current_opendesign_viewport,
};

pub(super) fn media_query_matches(selector: &str) -> bool {
    let query = selector.trim_start_matches("@media").trim();
    let mut matched_any_width_condition = false;

    for condition in query.split("and") {
        let condition = condition
            .trim()
            .trim_start_matches('(')
            .trim_end_matches(')')
            .trim();

        if let Some(value) = condition.strip_prefix("min-width:") {
            matched_any_width_condition = true;
            let Some(width) = css_first_size(value).and_then(|size| css_size_to_px(&size)) else {
                return false;
            };
            if current_opendesign_viewport().width < width {
                return false;
            }
        } else if let Some(value) = condition.strip_prefix("max-width:") {
            matched_any_width_condition = true;
            let Some(width) = css_first_size(value).and_then(|size| css_size_to_px(&size)) else {
                return false;
            };
            if current_opendesign_viewport().width > width {
                return false;
            }
        }
    }

    matched_any_width_condition
}
