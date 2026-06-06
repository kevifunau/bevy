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
    states: Vec<String>,
    pseudo_element: Option<String>,
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

    pub(super) fn state_name(&self) -> Option<&'static str> {
        self.parts
            .last()?
            .compound
            .states
            .iter()
            .rev()
            .find_map(|state| match state.as_str() {
                "hover" => Some("hovered"),
                "active" => Some("pressed"),
                "focus" | "focus-visible" => Some("focused"),
                _ => None,
            })
    }

    pub(super) fn pseudo_element_name(&self) -> Option<&str> {
        self.parts.last()?.compound.pseudo_element.as_deref()
    }

    pub(super) fn weight(&self) -> i32 {
        self.weight
    }

    fn matches_from(&self, part_index: usize, dom_node: roxmltree::Node<'_, '_>) -> bool {
        let part = &self.parts[part_index];
        if !part.compound.matches(dom_node) {
            return false;
        }
        if part_index == 0 {
            return true;
        }

        match part.combinator {
            OpenDesignCombinator::DirectChild => dom_node
                .parent()
                .filter(|parent| parent.is_element())
                .is_some_and(|parent| self.matches_from(part_index - 1, parent)),
            OpenDesignCombinator::Descendant => dom_node
                .ancestors()
                .skip(1)
                .filter(|ancestor| ancestor.is_element())
                .any(|ancestor| self.matches_from(part_index - 1, ancestor)),
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
                '.' | '#' | ':' => break,
                _ => tag.push(read_selector_char(&mut chars)),
            }
        }
        if !tag.is_empty() {
            compound.tag = Some(unescape_css_ident(&tag).to_ascii_lowercase());
        }

        while let Some(prefix) = chars.next() {
            let mut value = String::new();
            while let Some(character) = chars.peek().copied() {
                if matches!(character, '.' | '#' | ':') {
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
        weight += self.states.len() as i32 * 10;
        if self.id.is_some() {
            weight += 100;
        }
        weight
    }

    fn matches(&self, dom_node: roxmltree::Node<'_, '_>) -> bool {
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
    }
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
