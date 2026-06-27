use crate::core::opendesign::dom::has_class;

#[derive(Debug, Clone)]
pub(super) struct OpenDesignSelector {
    parts: Vec<OpenDesignSelectorPart>,
    weight: i32,
}

#[derive(Debug, Clone)]
struct OpenDesignSelectorPart {
    combinator: OpenDesignCombinator,
    compound: OpenDesignSelectorCompound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OpenDesignCombinator {
    SelfNode,
    Descendant,
    DirectChild,
}

#[derive(Debug, Default, Clone)]
struct OpenDesignSelectorCompound {
    tag: Option<String>,
    id: Option<String>,
    classes: Vec<String>,
    attributes: Vec<OpenDesignAttributeSelector>,
    states: Vec<String>,
    pseudo_element: Option<String>,
}

#[derive(Debug, Clone)]
struct OpenDesignAttributeSelector {
    name: String,
    value: Option<String>,
}

impl OpenDesignSelector {
    pub(super) fn parse(selector: &str) -> Option<Self> {
        let mut parts = Vec::new();
        let mut token = String::new();
        let mut combinator = OpenDesignCombinator::SelfNode;
        let mut chars = selector.chars().peekable();

        while let Some(character) = chars.next() {
            match character {
                '>' => {
                    push_selector_part(&mut parts, &mut token, combinator);
                    combinator = OpenDesignCombinator::DirectChild;
                    while chars.peek().is_some_and(|c| c.is_whitespace()) {
                        chars.next();
                    }
                }
                character if character.is_whitespace() => {
                    push_selector_part(&mut parts, &mut token, combinator);
                    if !parts.is_empty() {
                        combinator = OpenDesignCombinator::Descendant;
                    }
                    while chars.peek().is_some_and(|c| c.is_whitespace()) {
                        chars.next();
                    }
                }
                _ => token.push(character),
            }
        }
        push_selector_part(&mut parts, &mut token, combinator);

        if parts.is_empty() {
            return None;
        }

        let weight = parts.iter().map(|part| part.compound.weight()).sum::<i32>();
        Some(Self { parts, weight })
    }

    pub(super) fn parse_pseudo(selector: &str) -> Option<Self> {
        let pseudo_element = if selector.contains("::before") {
            "before"
        } else if selector.contains("::after") {
            "after"
        } else {
            return None;
        };

        let cleaned = selector.replace("::before", "").replace("::after", "");
        let mut parsed = Self::parse(cleaned.trim())?;
        parsed.parts.last_mut()?.compound.pseudo_element = Some(pseudo_element.to_string());
        Some(parsed)
    }

    pub(super) fn matches(&self, dom_node: roxmltree::Node<'_, '_>) -> bool {
        self.matches_from(self.parts.len() - 1, dom_node)
    }

    pub(super) fn matches_state_template(&self, dom_node: roxmltree::Node<'_, '_>) -> bool {
        self.matches_from_with_state_template(self.parts.len() - 1, dom_node)
    }

    pub(super) fn state_name(&self) -> Option<&'static str> {
        let compound = &self.parts.last()?.compound;
        compound
            .states
            .iter()
            .rev()
            .find_map(|state| match state.as_str() {
                "hover" => Some("hovered"),
                "active" | "pressed" => Some("pressed"),
                "focus" | "focus-visible" => Some("focused"),
                "disabled" => Some("disabled"),
                "checked" => Some("checked"),
                _ => None,
            })
            .or_else(|| self.initial_state_name())
    }

    pub(super) fn initial_state_name(&self) -> Option<&'static str> {
        let compound = &self.parts.last()?.compound;
        compound.attributes.iter().rev().find_map(|attribute| {
            match (attribute.name.as_str(), attribute.value.as_deref()) {
                ("aria-current", Some("page")) | ("aria-selected", Some("true")) => {
                    Some("selected")
                }
                ("disabled", _) | ("aria-disabled", Some("true")) => Some("disabled"),
                ("checked", _) | ("aria-checked", Some("true")) => Some("checked"),
                _ => None,
            }
        })
    }

    pub(super) fn pseudo_element_name(&self) -> Option<&str> {
        self.parts.last()?.compound.pseudo_element.as_deref()
    }

    pub(super) fn weight(&self) -> i32 {
        self.weight
    }

    pub(super) fn is_single_tag_selector(&self, tag: &str) -> bool {
        self.parts.len() == 1 && self.parts[0].compound.is_tag_only(tag)
    }

    fn matches_from(&self, part_index: usize, dom_node: roxmltree::Node<'_, '_>) -> bool {
        self.matches_from_with(part_index, dom_node, |compound, node| {
            compound.matches(node)
        })
    }

    fn matches_from_with_state_template(
        &self,
        part_index: usize,
        dom_node: roxmltree::Node<'_, '_>,
    ) -> bool {
        self.matches_from_with(part_index, dom_node, |compound, node| {
            compound.matches_state_template(node)
        })
    }

    fn matches_from_with(
        &self,
        part_index: usize,
        dom_node: roxmltree::Node<'_, '_>,
        compound_matches: fn(&OpenDesignSelectorCompound, roxmltree::Node<'_, '_>) -> bool,
    ) -> bool {
        let part = &self.parts[part_index];
        if !compound_matches(&part.compound, dom_node) {
            return false;
        }
        if part_index == 0 {
            return true;
        }

        match part.combinator {
            OpenDesignCombinator::DirectChild => dom_node
                .parent()
                .filter(|parent| parent.is_element())
                .is_some_and(|parent| {
                    self.matches_from_with(part_index - 1, parent, compound_matches)
                }),
            OpenDesignCombinator::Descendant => dom_node
                .ancestors()
                .skip(1)
                .filter(|ancestor| ancestor.is_element())
                .any(|ancestor| self.matches_from_with(part_index - 1, ancestor, compound_matches)),
            OpenDesignCombinator::SelfNode => false,
        }
    }
}

impl OpenDesignSelectorCompound {
    fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        if raw.is_empty() {
            return None;
        }
        if raw == "*" {
            return Some(Self::default());
        }

        let mut compound = Self::default();
        let mut chars = raw.chars().peekable();
        let mut tag = String::new();

        while let Some(character) = chars.peek().copied() {
            match character {
                '.' | '#' | ':' | '[' => break,
                _ => tag.push(read_selector_char(&mut chars)),
            }
        }
        if !tag.is_empty() {
            compound.tag = Some(unescape_css_ident(&tag).to_ascii_lowercase());
        }

        while let Some(prefix) = chars.next() {
            if prefix == '[' {
                let mut raw_attribute = String::new();
                for character in chars.by_ref() {
                    if character == ']' {
                        break;
                    }
                    raw_attribute.push(character);
                }
                if let Some(attribute) = parse_attribute_selector(&raw_attribute) {
                    compound.attributes.push(attribute);
                }
                continue;
            }

            let mut value = String::new();
            while let Some(character) = chars.peek().copied() {
                if matches!(character, '.' | '#' | ':' | '[') {
                    break;
                }
                value.push(read_selector_char(&mut chars));
            }
            if value.is_empty() {
                continue;
            }
            let value = unescape_css_ident(&value);
            match prefix {
                '.' => compound.classes.push(value),
                '#' => compound.id = Some(value),
                ':' => compound.states.push(value),
                _ => {}
            }
        }

        Some(compound)
    }

    fn weight(&self) -> i32 {
        let mut weight = 0;
        if self.tag.is_some() {
            weight += 1;
        }
        weight += self.classes.len() as i32 * 10;
        weight += self.attributes.len() as i32 * 10;
        weight += self.states.len() as i32 * 10;
        if self.id.is_some() {
            weight += 100;
        }
        weight
    }

    fn matches(&self, dom_node: roxmltree::Node<'_, '_>) -> bool {
        self.matches_with(dom_node, false)
    }

    fn matches_state_template(&self, dom_node: roxmltree::Node<'_, '_>) -> bool {
        self.matches_with(dom_node, true)
    }

    fn matches_with(
        &self,
        dom_node: roxmltree::Node<'_, '_>,
        ignore_state_attributes: bool,
    ) -> bool {
        if !dom_node.is_element() {
            return false;
        }
        if let Some(tag) = &self.tag
            && dom_node.tag_name().name().to_ascii_lowercase() != *tag
        {
            return false;
        }
        if let Some(id) = &self.id
            && dom_node.attribute("id") != Some(id.as_str())
        {
            return false;
        }
        self.classes
            .iter()
            .all(|class_name| has_class(dom_node, class_name))
            && self.attributes.iter().all(|attribute| {
                if ignore_state_attributes && attribute.is_state_attribute() {
                    return true;
                }
                let actual = dom_node.attribute(attribute.name.as_str());
                match attribute.value.as_deref() {
                    Some(expected) => actual == Some(expected),
                    None => actual.is_some(),
                }
            })
    }

    fn is_tag_only(&self, tag: &str) -> bool {
        self.tag.as_deref() == Some(tag)
            && self.id.is_none()
            && self.classes.is_empty()
            && self.attributes.is_empty()
            && self.states.is_empty()
            && self.pseudo_element.is_none()
    }
}

impl OpenDesignAttributeSelector {
    fn is_state_attribute(&self) -> bool {
        matches!(self.name.as_str(), "aria-current" | "aria-selected")
    }
}

fn parse_attribute_selector(raw: &str) -> Option<OpenDesignAttributeSelector> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let (name, value) = raw
        .split_once('=')
        .map(|(name, value)| {
            (
                name.trim(),
                Some(
                    value
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string(),
                ),
            )
        })
        .unwrap_or((raw, None));
    if name.is_empty() {
        return None;
    }

    Some(OpenDesignAttributeSelector {
        name: unescape_css_ident(name),
        value,
    })
}

fn push_selector_part(
    parts: &mut Vec<OpenDesignSelectorPart>,
    token: &mut String,
    combinator: OpenDesignCombinator,
) {
    if let Some(compound) = OpenDesignSelectorCompound::parse(token) {
        parts.push(OpenDesignSelectorPart {
            combinator: if parts.is_empty() {
                OpenDesignCombinator::SelfNode
            } else {
                combinator
            },
            compound,
        });
    }
    token.clear();
}

fn read_selector_char(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> char {
    let character = chars.next().unwrap_or_default();
    if character == '\\' {
        chars.next().unwrap_or(character)
    } else {
        character
    }
}

fn unescape_css_ident(value: &str) -> String {
    let mut output = String::new();
    let mut escaped = false;
    for character in value.chars() {
        if escaped {
            output.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else {
            output.push(character);
        }
    }
    output
}
