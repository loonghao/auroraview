//! Accessibility tree node processing

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::snapshot::RefInfo;

/// Accessibility tree node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A11yNode {
    /// Node ID
    pub node_id: String,
    /// ARIA role
    pub role: String,
    /// Accessible name
    pub name: String,
    /// Description
    pub description: String,
    /// Value (for inputs)
    pub value: Option<String>,
    /// Children nodes
    pub children: Vec<A11yNode>,
    /// Whether this node is interactive
    pub interactive: bool,
    /// Backend node ID for CDP operations
    pub backend_node_id: Option<i64>,
    /// Depth in tree
    pub depth: usize,
}

impl A11yNode {
    /// Create a new node
    pub fn new(node_id: impl Into<String>, role: impl Into<String>) -> Self {
        Self {
            node_id: node_id.into(),
            role: role.into(),
            name: String::new(),
            description: String::new(),
            value: None,
            children: Vec::new(),
            interactive: false,
            backend_node_id: None,
            depth: 0,
        }
    }

    /// Check if role is interactive
    pub fn is_interactive_role(role: &str) -> bool {
        matches!(
            role.to_lowercase().as_str(),
            "button"
                | "link"
                | "textbox"
                | "searchbox"
                | "checkbox"
                | "radio"
                | "combobox"
                | "listbox"
                | "option"
                | "menuitem"
                | "menuitemcheckbox"
                | "menuitemradio"
                | "tab"
                | "switch"
                | "slider"
                | "spinbutton"
                | "scrollbar"
                | "treeitem"
                | "gridcell"
                | "rowheader"
                | "columnheader"
        )
    }
}

/// Process CDP accessibility tree response into structured nodes and refs
pub fn process_a11y_tree(ax_tree: Value) -> (Vec<A11yNode>, HashMap<String, RefInfo>) {
    let mut nodes = Vec::new();
    let mut refs = HashMap::new();
    let mut ref_counter = 1u32;

    // Get nodes array from response
    let ax_nodes = match ax_tree.get("nodes").and_then(|n| n.as_array()) {
        Some(n) => n,
        None => return (nodes, refs),
    };

    // Build node map
    let mut node_map: HashMap<String, A11yNode> = HashMap::new();
    let mut children_map: HashMap<String, Vec<String>> = HashMap::new();

    for ax_node in ax_nodes {
        let node_id = ax_node["nodeId"].as_str().unwrap_or("").to_string();

        if node_id.is_empty() {
            continue;
        }

        // Extract role
        let role = ax_node["role"]["value"]
            .as_str()
            .unwrap_or("none")
            .to_string();

        // Skip ignored/none nodes
        if role == "none" || role == "Ignored" || ax_node["ignored"].as_bool().unwrap_or(false) {
            continue;
        }

        let mut node = A11yNode::new(&node_id, &role);

        // Extract name
        if let Some(name_obj) = ax_node.get("name") {
            node.name = name_obj["value"].as_str().unwrap_or("").to_string();
        }

        // Extract description
        if let Some(desc_obj) = ax_node.get("description") {
            node.description = desc_obj["value"].as_str().unwrap_or("").to_string();
        }

        // Extract value
        if let Some(value_obj) = ax_node.get("value") {
            node.value = value_obj["value"].as_str().map(|s| s.to_string());
        }

        // Extract backend node ID
        node.backend_node_id = ax_node["backendDOMNodeId"].as_i64();

        // Check if interactive
        node.interactive = A11yNode::is_interactive_role(&role);

        // Track children
        if let Some(children) = ax_node["childIds"].as_array() {
            let child_ids: Vec<String> = children
                .iter()
                .filter_map(|c| c.as_str().map(|s| s.to_string()))
                .collect();
            children_map.insert(node_id.clone(), child_ids);
        }

        // Create ref for interactive nodes
        if node.interactive && !node.name.is_empty() {
            let ref_id = format!("@{}", ref_counter);
            ref_counter += 1;

            let mut ref_info = RefInfo::new(&ref_id, &role, &node.name);

            // Add description
            if !node.description.is_empty() {
                ref_info = ref_info.with_description(&node.description);
            }

            // Add backend node ID
            if let Some(backend_id) = node.backend_node_id {
                ref_info = ref_info.with_backend_node_id(backend_id);
            }

            refs.insert(ref_id, ref_info);
        }

        node_map.insert(node_id, node);
    }

    // Build tree structure (find root nodes)
    let all_children: std::collections::HashSet<_> =
        children_map.values().flatten().cloned().collect();

    for (node_id, mut node) in node_map {
        // Root nodes are those not referenced as children
        if !all_children.contains(&node_id) {
            build_tree(&mut node, &children_map, &mut HashMap::new(), 0);
            nodes.push(node);
        }
    }

    (nodes, refs)
}

/// Recursively build tree structure
fn build_tree(
    _node: &mut A11yNode,
    _children_map: &HashMap<String, Vec<String>>,
    _node_map: &mut HashMap<String, A11yNode>,
    _depth: usize,
) {
    // Note: This is a simplified version. Full implementation would
    // properly reconstruct the tree hierarchy from the flat node list.
}

/// Alternative: Process from DOM tree with accessibility info
#[allow(dead_code)]
pub fn process_dom_tree_with_a11y(
    dom_root: Value,
    a11y_map: &HashMap<i64, A11yNode>,
) -> Vec<A11yNode> {
    let mut nodes = Vec::new();

    fn process_node(
        node: &Value,
        a11y_map: &HashMap<i64, A11yNode>,
        nodes: &mut Vec<A11yNode>,
        depth: usize,
    ) {
        let backend_node_id = node["backendNodeId"].as_i64().unwrap_or(0);

        // Check if this node has accessibility info
        if let Some(a11y_node) = a11y_map.get(&backend_node_id) {
            let mut cloned = a11y_node.clone();
            cloned.depth = depth;
            nodes.push(cloned);
        }

        // Process children
        if let Some(children) = node["children"].as_array() {
            for child in children {
                process_node(child, a11y_map, nodes, depth + 1);
            }
        }
    }

    if let Some(root) = dom_root.get("root") {
        process_node(root, a11y_map, &mut nodes, 0);
    }

    nodes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interactive_roles() {
        assert!(A11yNode::is_interactive_role("button"));
        assert!(A11yNode::is_interactive_role("textbox"));
        assert!(A11yNode::is_interactive_role("link"));
        assert!(!A11yNode::is_interactive_role("generic"));
        assert!(!A11yNode::is_interactive_role("text"));
    }

    #[test]
    fn test_process_empty_tree() {
        let empty = serde_json::json!({});
        let (nodes, refs) = process_a11y_tree(empty);
        assert!(nodes.is_empty());
        assert!(refs.is_empty());
    }

    #[test]
    fn test_process_simple_tree() {
        let tree = serde_json::json!({
            "nodes": [
                {
                    "nodeId": "1",
                    "role": {"value": "button"},
                    "name": {"value": "Click Me"},
                    "backendDOMNodeId": 100
                },
                {
                    "nodeId": "2",
                    "role": {"value": "textbox"},
                    "name": {"value": "Search"},
                    "backendDOMNodeId": 101
                }
            ]
        });

        let (_, refs) = process_a11y_tree(tree);
        assert_eq!(refs.len(), 2);
        assert!(refs.contains_key("@1"));
        assert!(refs.contains_key("@2"));
    }
}
