//! Tests for chrome.contextMenus API

use auroraview_extensions::apis::context_menus::{
    ContextMenusApiHandler, ContextMenusManager, ContextType, CreateProperties, ItemType,
};
use auroraview_extensions::apis::ApiHandler;
use serde_json::json;
use std::sync::Arc;

#[test]
fn test_context_menus_manager_new() {
    let manager = ContextMenusManager::new();
    let items = manager.get_items("test-ext");
    assert!(items.is_empty());
}

#[test]
fn test_context_menus_manager_default() {
    let manager = ContextMenusManager::default();
    let items = manager.get_items("test-ext");
    assert!(items.is_empty());
}

#[test]
fn test_create_menu_item() {
    let manager = ContextMenusManager::new();

    let props = CreateProperties {
        item_type: ItemType::Normal,
        id: Some("my-item".to_string()),
        title: Some("My Menu Item".to_string()),
        checked: false,
        contexts: vec![ContextType::Page],
        visible: true,
        enabled: true,
        parent_id: None,
        document_url_patterns: vec![],
        target_url_patterns: vec![],
    };

    let result = manager.create("test-ext", props);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "my-item");

    let items = manager.get_items("test-ext");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, "my-item");
    assert_eq!(items[0].properties.title, Some("My Menu Item".to_string()));
}

#[test]
fn test_create_menu_item_auto_id() {
    let manager = ContextMenusManager::new();

    let props = CreateProperties {
        item_type: ItemType::Normal,
        id: None, // Auto-generate ID
        title: Some("Auto ID Item".to_string()),
        checked: false,
        contexts: vec![],
        visible: true,
        enabled: true,
        parent_id: None,
        document_url_patterns: vec![],
        target_url_patterns: vec![],
    };

    let result = manager.create("test-ext", props);
    assert!(result.is_ok());
    let id = result.unwrap();
    assert!(id.starts_with("menu_"));
}

#[test]
fn test_update_menu_item() {
    let manager = ContextMenusManager::new();

    // Create item
    let props = CreateProperties {
        item_type: ItemType::Normal,
        id: Some("update-me".to_string()),
        title: Some("Original Title".to_string()),
        checked: false,
        contexts: vec![],
        visible: true,
        enabled: true,
        parent_id: None,
        document_url_patterns: vec![],
        target_url_patterns: vec![],
    };
    manager.create("test-ext", props).unwrap();

    // Update item
    let new_props = CreateProperties {
        item_type: ItemType::Normal,
        id: Some("update-me".to_string()),
        title: Some("Updated Title".to_string()),
        checked: false,
        contexts: vec![],
        visible: true,
        enabled: true,
        parent_id: None,
        document_url_patterns: vec![],
        target_url_patterns: vec![],
    };
    let result = manager.update("test-ext", "update-me", new_props);
    assert!(result.is_ok());

    let items = manager.get_items("test-ext");
    assert_eq!(items[0].properties.title, Some("Updated Title".to_string()));
}

#[test]
fn test_update_nonexistent_item() {
    let manager = ContextMenusManager::new();

    let props = CreateProperties {
        item_type: ItemType::Normal,
        id: None,
        title: Some("Test".to_string()),
        checked: false,
        contexts: vec![],
        visible: true,
        enabled: true,
        parent_id: None,
        document_url_patterns: vec![],
        target_url_patterns: vec![],
    };

    let result = manager.update("test-ext", "nonexistent", props);
    assert!(result.is_err());
}

#[test]
fn test_remove_menu_item() {
    let manager = ContextMenusManager::new();

    // Create item
    let props = CreateProperties {
        item_type: ItemType::Normal,
        id: Some("remove-me".to_string()),
        title: Some("To Remove".to_string()),
        checked: false,
        contexts: vec![],
        visible: true,
        enabled: true,
        parent_id: None,
        document_url_patterns: vec![],
        target_url_patterns: vec![],
    };
    manager.create("test-ext", props).unwrap();

    assert_eq!(manager.get_items("test-ext").len(), 1);

    // Remove item
    let result = manager.remove("test-ext", "remove-me");
    assert!(result.is_ok());
    assert!(manager.get_items("test-ext").is_empty());
}

#[test]
fn test_remove_nonexistent_item() {
    let manager = ContextMenusManager::new();
    let result = manager.remove("test-ext", "nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_remove_all() {
    let manager = ContextMenusManager::new();

    // Create multiple items
    for i in 0..3 {
        let props = CreateProperties {
            item_type: ItemType::Normal,
            id: Some(format!("item-{}", i)),
            title: Some(format!("Item {}", i)),
            checked: false,
            contexts: vec![],
            visible: true,
            enabled: true,
            parent_id: None,
            document_url_patterns: vec![],
            target_url_patterns: vec![],
        };
        manager.create("test-ext", props).unwrap();
    }

    assert_eq!(manager.get_items("test-ext").len(), 3);

    // Remove all
    manager.remove_all("test-ext");
    assert!(manager.get_items("test-ext").is_empty());
}

#[test]
fn test_get_items_for_context() {
    let manager = ContextMenusManager::new();

    // Create page context item
    let props = CreateProperties {
        item_type: ItemType::Normal,
        id: Some("page-item".to_string()),
        title: Some("Page Item".to_string()),
        checked: false,
        contexts: vec![ContextType::Page],
        visible: true,
        enabled: true,
        parent_id: None,
        document_url_patterns: vec![],
        target_url_patterns: vec![],
    };
    manager.create("test-ext", props).unwrap();

    // Create link context item
    let props = CreateProperties {
        item_type: ItemType::Normal,
        id: Some("link-item".to_string()),
        title: Some("Link Item".to_string()),
        checked: false,
        contexts: vec![ContextType::Link],
        visible: true,
        enabled: true,
        parent_id: None,
        document_url_patterns: vec![],
        target_url_patterns: vec![],
    };
    manager.create("test-ext", props).unwrap();

    // Get page context items
    let page_items = manager.get_items_for_context(&ContextType::Page, None);
    assert_eq!(page_items.len(), 1);
    assert_eq!(page_items[0].id, "page-item");

    // Get link context items
    let link_items = manager.get_items_for_context(&ContextType::Link, None);
    assert_eq!(link_items.len(), 1);
    assert_eq!(link_items[0].id, "link-item");
}

#[test]
fn test_get_items_for_context_all() {
    let manager = ContextMenusManager::new();

    // Create item with all contexts
    let props = CreateProperties {
        item_type: ItemType::Normal,
        id: Some("all-item".to_string()),
        title: Some("All Contexts".to_string()),
        checked: false,
        contexts: vec![ContextType::All],
        visible: true,
        enabled: true,
        parent_id: None,
        document_url_patterns: vec![],
        target_url_patterns: vec![],
    };
    manager.create("test-ext", props).unwrap();

    // Should appear in any context
    assert_eq!(
        manager
            .get_items_for_context(&ContextType::Page, None)
            .len(),
        1
    );
    assert_eq!(
        manager
            .get_items_for_context(&ContextType::Link, None)
            .len(),
        1
    );
    assert_eq!(
        manager
            .get_items_for_context(&ContextType::Selection, None)
            .len(),
        1
    );
}

#[test]
fn test_get_items_for_context_empty_contexts() {
    let manager = ContextMenusManager::new();

    // Create item with empty contexts (should match all)
    let props = CreateProperties {
        item_type: ItemType::Normal,
        id: Some("empty-ctx".to_string()),
        title: Some("Empty Contexts".to_string()),
        checked: false,
        contexts: vec![], // Empty = all
        visible: true,
        enabled: true,
        parent_id: None,
        document_url_patterns: vec![],
        target_url_patterns: vec![],
    };
    manager.create("test-ext", props).unwrap();

    // Should appear in any context
    assert_eq!(
        manager
            .get_items_for_context(&ContextType::Page, None)
            .len(),
        1
    );
}

#[test]
fn test_hidden_items_not_returned() {
    let manager = ContextMenusManager::new();

    let props = CreateProperties {
        item_type: ItemType::Normal,
        id: Some("hidden".to_string()),
        title: Some("Hidden Item".to_string()),
        checked: false,
        contexts: vec![],
        visible: false, // Hidden
        enabled: true,
        parent_id: None,
        document_url_patterns: vec![],
        target_url_patterns: vec![],
    };
    manager.create("test-ext", props).unwrap();

    let items = manager.get_items_for_context(&ContextType::Page, None);
    assert!(items.is_empty());
}

#[test]
fn test_disabled_items_not_returned() {
    let manager = ContextMenusManager::new();

    let props = CreateProperties {
        item_type: ItemType::Normal,
        id: Some("disabled".to_string()),
        title: Some("Disabled Item".to_string()),
        checked: false,
        contexts: vec![],
        visible: true,
        enabled: false, // Disabled
        parent_id: None,
        document_url_patterns: vec![],
        target_url_patterns: vec![],
    };
    manager.create("test-ext", props).unwrap();

    let items = manager.get_items_for_context(&ContextType::Page, None);
    assert!(items.is_empty());
}

#[test]
fn test_context_menus_api_handler_namespace() {
    let manager = Arc::new(ContextMenusManager::new());
    let handler = ContextMenusApiHandler::new(manager);
    assert_eq!(handler.namespace(), "contextMenus");
}

#[test]
fn test_context_menus_api_handler_methods() {
    let manager = Arc::new(ContextMenusManager::new());
    let handler = ContextMenusApiHandler::new(manager);
    let methods = handler.methods();

    assert!(methods.contains(&"create"));
    assert!(methods.contains(&"update"));
    assert!(methods.contains(&"remove"));
    assert!(methods.contains(&"removeAll"));
}

#[test]
fn test_context_menus_api_create() {
    let manager = Arc::new(ContextMenusManager::new());
    let handler = ContextMenusApiHandler::new(manager.clone());

    let result = handler.handle(
        "create",
        json!({
            "id": "api-item",
            "title": "API Created Item",
            "contexts": ["page"]
        }),
        "test-ext",
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), json!("api-item"));

    let items = manager.get_items("test-ext");
    assert_eq!(items.len(), 1);
}

#[test]
fn test_context_menus_api_update() {
    let manager = Arc::new(ContextMenusManager::new());
    let handler = ContextMenusApiHandler::new(manager.clone());

    // Create
    handler
        .handle(
            "create",
            json!({
                "id": "to-update",
                "title": "Original"
            }),
            "test-ext",
        )
        .unwrap();

    // Update
    let result = handler.handle(
        "update",
        json!({
            "id": "to-update",
            "updateProperties": {
                "title": "Updated"
            }
        }),
        "test-ext",
    );
    assert!(result.is_ok());

    let items = manager.get_items("test-ext");
    assert_eq!(items[0].properties.title, Some("Updated".to_string()));
}

#[test]
fn test_context_menus_api_remove() {
    let manager = Arc::new(ContextMenusManager::new());
    let handler = ContextMenusApiHandler::new(manager.clone());

    // Create
    handler
        .handle(
            "create",
            json!({
                "id": "to-remove",
                "title": "To Remove"
            }),
            "test-ext",
        )
        .unwrap();

    // Remove
    let result = handler.handle(
        "remove",
        json!({
            "menuItemId": "to-remove"
        }),
        "test-ext",
    );
    assert!(result.is_ok());

    assert!(manager.get_items("test-ext").is_empty());
}

#[test]
fn test_context_menus_api_remove_all() {
    let manager = Arc::new(ContextMenusManager::new());
    let handler = ContextMenusApiHandler::new(manager.clone());

    // Create multiple
    for i in 0..3 {
        handler
            .handle(
                "create",
                json!({
                    "id": format!("item-{}", i),
                    "title": format!("Item {}", i)
                }),
                "test-ext",
            )
            .unwrap();
    }

    assert_eq!(manager.get_items("test-ext").len(), 3);

    // Remove all
    let result = handler.handle("removeAll", json!({}), "test-ext");
    assert!(result.is_ok());

    assert!(manager.get_items("test-ext").is_empty());
}

#[test]
fn test_context_menus_api_unsupported_method() {
    let manager = Arc::new(ContextMenusManager::new());
    let handler = ContextMenusApiHandler::new(manager);

    let result = handler.handle("unknownMethod", json!({}), "test-ext");
    assert!(result.is_err());
}
