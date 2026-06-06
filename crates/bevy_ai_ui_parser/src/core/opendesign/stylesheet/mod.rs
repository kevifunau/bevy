mod media;
mod parser;
mod selector;

use std::collections::HashMap;

pub(crate) use parser::css_declarations;
use parser::{css_rules, style_blocks};
use selector::OpenDesignSelector;

#[derive(Debug, Default)]
pub(crate) struct OpenDesignStylesheet {
    variables: HashMap<String, String>,
    rules: Vec<OpenDesignCssRule>,
}

#[derive(Debug)]
struct OpenDesignCssRule {
    selector: OpenDesignSelector,
    declarations: Vec<(String, String)>,
    order: usize,
}

impl OpenDesignStylesheet {
    pub(crate) fn parse(html: &str) -> Self {
        let mut stylesheet = Self::default();
        let mut order = 0;

        for block in style_blocks(html) {
            for (selector_group, declarations) in css_rules(block) {
                for (name, value) in &declarations {
                    if selector_group.trim() == ":root" && name.starts_with("--") {
                        stylesheet
                            .variables
                            .insert(name.to_string(), value.trim().to_string());
                    }
                }

                for selector in selector_group.split(',') {
                    let selector = selector.trim();
                    if selector.is_empty() || selector.starts_with('@') {
                        continue;
                    }
                    if selector.contains("::before") || selector.contains("::after") {
                        if let Some(selector) = OpenDesignSelector::parse_pseudo(selector) {
                            stylesheet.rules.push(OpenDesignCssRule {
                                selector,
                                declarations: declarations.clone(),
                                order,
                            });
                            order += 1;
                        }
                        continue;
                    }
                    if selector.contains("::") || selector.contains('[') {
                        continue;
                    }
                    if let Some(selector) = OpenDesignSelector::parse(selector) {
                        stylesheet.rules.push(OpenDesignCssRule {
                            selector,
                            declarations: declarations.clone(),
                            order,
                        });
                        order += 1;
                    }
                }
            }
        }

        stylesheet
    }

    pub(crate) fn matching_declarations(
        &self,
        dom_node: roxmltree::Node<'_, '_>,
    ) -> Vec<&(String, String)> {
        let mut rules = self
            .rules
            .iter()
            .filter(|rule| {
                rule.selector.state_name().is_none()
                    && rule.selector.pseudo_element_name().is_none()
                    && rule.selector.matches(dom_node)
            })
            .collect::<Vec<_>>();
        rules.sort_by_key(|rule| (rule.selector.weight(), rule.order));
        rules
            .into_iter()
            .flat_map(|rule| rule.declarations.iter())
            .collect()
    }

    pub(crate) fn matching_pseudo_declarations(
        &self,
        dom_node: roxmltree::Node<'_, '_>,
        pseudo_element: &str,
    ) -> Vec<&(String, String)> {
        let mut rules = self
            .rules
            .iter()
            .filter(|rule| {
                rule.selector.state_name().is_none()
                    && rule.selector.pseudo_element_name() == Some(pseudo_element)
                    && rule.selector.matches(dom_node)
            })
            .collect::<Vec<_>>();
        rules.sort_by_key(|rule| (rule.selector.weight(), rule.order));
        rules
            .into_iter()
            .flat_map(|rule| rule.declarations.iter())
            .collect()
    }

    pub(crate) fn matching_state_declarations(
        &self,
        dom_node: roxmltree::Node<'_, '_>,
    ) -> Vec<(&str, &(String, String))> {
        let mut rules = self
            .rules
            .iter()
            .filter_map(|rule| {
                rule.selector
                    .state_name()
                    .filter(|_| rule.selector.matches(dom_node))
                    .map(|state| (state, rule))
            })
            .collect::<Vec<_>>();
        rules.sort_by_key(|(_, rule)| (rule.selector.weight(), rule.order));
        rules
            .into_iter()
            .flat_map(|(state, rule)| {
                rule.declarations
                    .iter()
                    .map(move |declaration| (state, declaration))
            })
            .collect()
    }

    pub(crate) fn resolve_value(&self, value: &str) -> String {
        self.resolve_value_with_variables(value, &HashMap::new())
    }

    pub(crate) fn resolve_value_with_variables(
        &self,
        value: &str,
        variables: &HashMap<String, String>,
    ) -> String {
        let mut resolved = value.trim().to_string();
        for _ in 0..4 {
            let Some(start) = resolved.find("var(") else {
                break;
            };
            let Some(end) = resolved[start..].find(')') else {
                break;
            };
            let end = start + end;
            let variable_name = resolved[start + 4..end].trim();
            let replacement = variables
                .get(variable_name)
                .cloned()
                .or_else(|| self.variables.get(variable_name).cloned())
                .unwrap_or_default();
            resolved.replace_range(start..=end, &replacement);
        }
        resolved.trim().to_string()
    }

    pub(crate) fn custom_properties_for_node(
        &self,
        dom_node: roxmltree::Node<'_, '_>,
    ) -> HashMap<String, String> {
        let mut variables = HashMap::new();
        let mut ancestors = dom_node
            .ancestors()
            .filter(|node| node.is_element())
            .collect::<Vec<_>>();
        ancestors.reverse();

        for node in ancestors {
            for (name, value) in self.matching_declarations(node) {
                if !name.starts_with("--") {
                    continue;
                }
                let value = self.resolve_value_with_variables(value, &variables);
                variables.insert(name.clone(), value);
            }

            if let Some(inline_style) = node.attribute("style") {
                for (name, value) in css_declarations(inline_style) {
                    if !name.starts_with("--") {
                        continue;
                    }
                    let value = self.resolve_value_with_variables(&value, &variables);
                    variables.insert(name, value);
                }
            }
        }

        variables
    }
}
