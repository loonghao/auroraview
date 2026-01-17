//! Accessibility tree formatting

use std::collections::HashMap;

use super::tree::A11yNode;
use crate::snapshot::RefInfo;

/// Format accessibility tree as text with refs
pub fn format_tree(nodes: &[A11yNode], refs: &HashMap<String, RefInfo>) -> String {
    let mut output = String::new();

    // Create reverse mapping from backend_node_id to ref_id
    let backend_to_ref: HashMap<i64, String> = refs
        .iter()
        .filter_map(|(ref_id, info)| info.backend_node_id.map(|id| (id, ref_id.clone())))
        .collect();

    for node in nodes {
        format_node(node, &backend_to_ref, &mut output, 0);
    }

    output
}

/// Format a single node recursively
fn format_node(
    node: &A11yNode,
    backend_to_ref: &HashMap<i64, String>,
    output: &mut String,
    depth: usize,
) {
    let indent = "  ".repeat(depth);

    // Get ref if this node has one
    let ref_str = node
        .backend_node_id
        .and_then(|id| backend_to_ref.get(&id))
        .map(|r| format!(" [{}]", r))
        .unwrap_or_default();

    // Format based on role
    match node.role.as_str() {
        // Skip generic containers unless they have meaningful content
        "generic" | "none" | "Ignored" if node.name.is_empty() => {
            // Just process children
            for child in &node.children {
                format_node(child, backend_to_ref, output, depth);
            }
        }

        // Landmark roles
        "banner" | "header" => {
            output.push_str(&format!("{}header{}\n", indent, ref_str));
            for child in &node.children {
                format_node(child, backend_to_ref, output, depth + 1);
            }
        }

        "navigation" | "nav" => {
            let name = if node.name.is_empty() {
                "nav".to_string()
            } else {
                format!("nav: {}", node.name)
            };
            output.push_str(&format!("{}{}{}\n", indent, name, ref_str));
            for child in &node.children {
                format_node(child, backend_to_ref, output, depth + 1);
            }
        }

        "main" => {
            output.push_str(&format!("{}main{}\n", indent, ref_str));
            for child in &node.children {
                format_node(child, backend_to_ref, output, depth + 1);
            }
        }

        "contentinfo" | "footer" => {
            output.push_str(&format!("{}footer{}\n", indent, ref_str));
            for child in &node.children {
                format_node(child, backend_to_ref, output, depth + 1);
            }
        }

        "complementary" | "aside" => {
            output.push_str(&format!("{}aside{}\n", indent, ref_str));
            for child in &node.children {
                format_node(child, backend_to_ref, output, depth + 1);
            }
        }

        // Interactive elements
        "button" => {
            output.push_str(&format!("{}[button{} \"{}\"]", indent, ref_str, node.name));
            if !node.description.is_empty() {
                output.push_str(&format!(" - {}", node.description));
            }
            output.push('\n');
        }

        "link" => {
            output.push_str(&format!("{}[link{} \"{}\"]", indent, ref_str, node.name));
            if !node.description.is_empty() {
                output.push_str(&format!(" - {}", node.description));
            }
            output.push('\n');
        }

        "textbox" | "searchbox" => {
            let value = node.value.as_deref().unwrap_or("");
            let display_value = if value.is_empty() {
                node.name.clone()
            } else {
                format!("{}={}", node.name, value)
            };
            output.push_str(&format!(
                "{}[textbox{} \"{}\"]",
                indent, ref_str, display_value
            ));
            output.push('\n');
        }

        "checkbox" => {
            let checked = node.value.as_ref().map(|v| v == "true").unwrap_or(false);
            let state = if checked { "☑" } else { "☐" };
            output.push_str(&format!(
                "{}[checkbox{} {} \"{}\"]",
                indent, ref_str, state, node.name
            ));
            output.push('\n');
        }

        "radio" => {
            let selected = node.value.as_ref().map(|v| v == "true").unwrap_or(false);
            let state = if selected { "●" } else { "○" };
            output.push_str(&format!(
                "{}[radio{} {} \"{}\"]",
                indent, ref_str, state, node.name
            ));
            output.push('\n');
        }

        "combobox" | "listbox" => {
            output.push_str(&format!(
                "{}[dropdown{} \"{}\"]",
                indent, ref_str, node.name
            ));
            if let Some(value) = &node.value {
                output.push_str(&format!(" = {}", value));
            }
            output.push('\n');
            for child in &node.children {
                format_node(child, backend_to_ref, output, depth + 1);
            }
        }

        "option" => {
            output.push_str(&format!("{}  - {}{}\n", indent, node.name, ref_str));
        }

        "tab" => {
            let selected = node.description.contains("selected");
            let marker = if selected { "▶" } else { " " };
            output.push_str(&format!(
                "{}[tab{} {} \"{}\"]",
                indent, ref_str, marker, node.name
            ));
            output.push('\n');
        }

        "tablist" => {
            output.push_str(&format!("{}tabs:\n", indent));
            for child in &node.children {
                format_node(child, backend_to_ref, output, depth + 1);
            }
        }

        // Structural elements
        "heading" => {
            let level = node
                .description
                .chars()
                .find(|c| c.is_ascii_digit())
                .unwrap_or('1');
            output.push_str(&format!(
                "{}[h{}] {}{}\n",
                indent, level, node.name, ref_str
            ));
        }

        "list" => {
            output.push_str(&format!("{}list:\n", indent));
            for child in &node.children {
                format_node(child, backend_to_ref, output, depth + 1);
            }
        }

        "listitem" => {
            output.push_str(&format!("{}- {}{}\n", indent, node.name, ref_str));
            for child in &node.children {
                format_node(child, backend_to_ref, output, depth + 1);
            }
        }

        "table" => {
            output.push_str(&format!("{}table:\n", indent));
            for child in &node.children {
                format_node(child, backend_to_ref, output, depth + 1);
            }
        }

        "row" => {
            output.push_str(&format!("{}| ", indent));
            for child in &node.children {
                if !child.name.is_empty() {
                    output.push_str(&format!("{} | ", child.name));
                }
            }
            output.push('\n');
        }

        "grid" => {
            let count = node.children.len();
            output.push_str(&format!("{}grid ({} items):\n", indent, count));
            for child in &node.children {
                format_node(child, backend_to_ref, output, depth + 1);
            }
        }

        "article" | "section" | "region" => {
            if !node.name.is_empty() {
                output.push_str(&format!("{}{}:{}\n", indent, node.name, ref_str));
            }
            for child in &node.children {
                format_node(child, backend_to_ref, output, depth + 1);
            }
        }

        // Text content
        "StaticText" | "text" => {
            if !node.name.is_empty() && !node.name.trim().is_empty() {
                output.push_str(&format!("{}\"{}\"\n", indent, node.name.trim()));
            }
        }

        // Image
        "image" | "img" => {
            if !node.name.is_empty() {
                output.push_str(&format!("{}[image: {}]{}\n", indent, node.name, ref_str));
            }
        }

        // Default: show role and name
        _ => {
            if !node.name.is_empty() {
                output.push_str(&format!(
                    "{}{}: {}{}\n",
                    indent, node.role, node.name, ref_str
                ));
            }
            for child in &node.children {
                format_node(child, backend_to_ref, output, depth + 1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_button() {
        let mut nodes = vec![A11yNode::new("1", "button")];
        nodes[0].name = "Submit".to_string();
        nodes[0].backend_node_id = Some(100);

        let mut refs = HashMap::new();
        refs.insert(
            "@1".to_string(),
            RefInfo::new("@1", "button", "Submit").with_backend_node_id(100),
        );

        let output = format_tree(&nodes, &refs);
        assert!(output.contains("[button [@1]"));
        assert!(output.contains("Submit"));
    }

    #[test]
    fn test_format_textbox() {
        let mut nodes = vec![A11yNode::new("1", "textbox")];
        nodes[0].name = "Search".to_string();
        nodes[0].value = Some("hello".to_string());

        let refs = HashMap::new();
        let output = format_tree(&nodes, &refs);
        assert!(output.contains("[textbox"));
        assert!(output.contains("Search=hello"));
    }
}
