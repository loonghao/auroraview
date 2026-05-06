//! DOM.* CDP methods.
//!
//! This module provides DOM-related CDP methods:
//! - `get_document`
//! - `get_styles_for_node`
//! - `query_selector`
//! - `query_selector_all`
//! - `get_outer_html`
//! - `get_attributes`
//! - `set_node_value`
//! - `set_attribute_value`
//! - `remove_attribute`

use std::collections::HashMap;
use std::time::Duration;

use serde_json::{json, Value};
use tracing::{debug, warn};

use super::CdpClient;
use crate::cdp::CdpError;

impl CdpClient {
    /// `DOM.getDocument` — get the DOM document node.
    ///
    /// Returns the root `Document` node as JSON.
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if the CDP call fails or the response is malformed.
    pub async fn get_document(&self, timeout: Duration) -> Result<Value, CdpError> {
        // Use retry logic for this idempotent method
        let result = self
            .call_with_retry(
                "DOM.getDocument",
                json!({}),
                timeout,
                3,
                Duration::from_millis(100),
                Duration::from_secs(5),
            )
            .await?;
        debug!(?result, "DOM.getDocument succeeded");
        Ok(result)
    }

    /// `CSS.getStylesForNode` — get computed styles for a DOM node.
    ///
    /// `node_id` is the DOM node ID (from `DOM.getDocument` or `DOM.querySelector`).
    /// Returns the computed styles as JSON.
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if the CDP call fails or the response is malformed.
    pub async fn get_styles_for_node(
        &self,
        node_id: i64,
        timeout: Duration,
    ) -> Result<Value, CdpError> {
        let params = json!({"nodeId": node_id});
        // Use retry logic for this idempotent method
        let result = self
            .call_with_retry(
                "CSS.getStylesForNode",
                params,
                timeout,
                3,
                Duration::from_millis(100),
                Duration::from_secs(5),
            )
            .await?;
        debug!(?node_id, ?result, "CSS.getStylesForNode succeeded");
        Ok(result)
    }

    /// `DOM.querySelector` — find the first element matching a CSS selector.
    ///
    /// `node_id` is the parent node ID (usually from `DOM.getDocument`).
    /// `selector` is a CSS selector string (e.g., `"#my-id"`, `".my-class"`).
    /// Returns the found node ID, or `None` if not found.
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if the CDP call fails or the response is malformed.
    pub async fn query_selector(
        &self,
        node_id: i64,
        selector: &str,
        timeout: Duration,
    ) -> Result<Option<i64>, CdpError> {
        let params = json!({
            "nodeId": node_id,
            "selector": selector,
        });
        // Use retry logic for this idempotent method
        let result = self
            .call_with_retry(
                "DOM.querySelector",
                params,
                timeout,
                3,
                Duration::from_millis(100),
                Duration::from_secs(5),
            )
            .await?;
        let found_node_id = result
            .get("nodeId")
            .and_then(Value::as_i64)
            .filter(|&id| id != 0); // CDP returns 0 when not found
        debug!(?node_id, %selector, ?found_node_id, "DOM.querySelector succeeded");
        Ok(found_node_id)
    }

    /// `DOM.querySelectorAll` — find all elements matching a CSS selector.
    ///
    /// `node_id` is the parent node ID (usually from `DOM.getDocument`).
    /// `selector` is a CSS selector string.
    /// Returns a vector of node IDs.
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if the CDP call fails or the response is malformed.
    pub async fn query_selector_all(
        &self,
        node_id: i64,
        selector: &str,
        timeout: Duration,
    ) -> Result<Vec<i64>, CdpError> {
        let params = json!({
            "nodeId": node_id,
            "selector": selector,
        });
        // Use retry logic for this idempotent method
        let result = self
            .call_with_retry(
                "DOM.querySelectorAll",
                params,
                timeout,
                3,
                Duration::from_millis(100),
                Duration::from_secs(5),
            )
            .await?;
        let node_ids: Vec<i64> = result
            .get("nodeIds")
            .and_then(Value::as_array)
            .map(|arr| arr.iter().filter_map(Value::as_i64).collect())
            .unwrap_or_default();
        debug!(?node_id, %selector, count = node_ids.len(), "DOM.querySelectorAll succeeded");
        Ok(node_ids)
    }

    /// `DOM.getOuterHTML` — get the outer HTML of a DOM node.
    ///
    /// `node_id` is the DOM node ID (from `DOM.getDocument` or `DOM.querySelector`).
    /// Returns the outer HTML as a string.
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if:
    /// - CDP call fails ([`CdpError::WebSocket`], [`CdpError::Timeout`], etc.)
    /// - Response is malformed ([`CdpError::MalformedResponse`])
    pub async fn get_outer_html(
        &self,
        node_id: i64,
        timeout: Duration,
    ) -> Result<String, CdpError> {
        let params = json!({"nodeId": node_id});
        // Use retry logic for this idempotent method
        let result = self
            .call_with_retry(
                "DOM.getOuterHTML",
                params,
                timeout,
                3,
                Duration::from_millis(100),
                Duration::from_secs(5),
            )
            .await?;
        let html = result
            .get("outerHTML")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                warn!(
                    ?node_id,
                    "DOM.getOuterHTML response missing 'outerHTML' field"
                );
                CdpError::MalformedResponse("DOM.getOuterHTML".to_string(), "outerHTML")
            })?
            .to_owned();
        debug!(
            ?node_id,
            html_len = html.len(),
            "DOM.getOuterHTML succeeded"
        );
        Ok(html)
    }

    /// `DOM.getAttributes` — get all attributes of a DOM node.
    ///
    /// `node_id` is the DOM node ID.
    /// Returns a map of attribute name -> value.
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if the CDP call fails or the response is malformed.
    pub async fn get_attributes(
        &self,
        node_id: i64,
        timeout: Duration,
    ) -> Result<HashMap<String, String>, CdpError> {
        let params = json!({"nodeId": node_id});
        // Use retry logic for this idempotent method
        let result = self
            .call_with_retry(
                "DOM.getAttributes",
                params,
                timeout,
                3,
                Duration::from_millis(100),
                Duration::from_secs(5),
            )
            .await?;
        let attrs_array = result
            .get("attributes")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                warn!(
                    ?node_id,
                    "DOM.getAttributes response missing 'attributes' field"
                );
                CdpError::MalformedResponse("DOM.getAttributes".to_string(), "attributes")
            })?;

        // CDP returns attributes as a flat array: ["name1", "value1", "name2", "value2", ...]
        let mut attrs = HashMap::new();
        let mut i = 0;
        while i + 1 < attrs_array.len() {
            if let (Some(name), Some(value)) =
                (attrs_array[i].as_str(), attrs_array[i + 1].as_str())
            {
                attrs.insert(name.to_owned(), value.to_owned());
            }
            i += 2;
        }
        debug!(?node_id, count = attrs.len(), "DOM.getAttributes succeeded");
        Ok(attrs)
    }

    /// `DOM.setNodeValue` — set the value of a text node.
    ///
    /// `node_id` is the DOM node ID (must be a text node).
    /// `value` is the new text value.
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if the CDP call fails.
    pub async fn set_node_value(
        &self,
        node_id: i64,
        value: &str,
        timeout: Duration,
    ) -> Result<(), CdpError> {
        let params = json!({
            "nodeId": node_id,
            "value": value,
        });
        self.call("DOM.setNodeValue", params, timeout).await?;
        debug!(?node_id, %value, "DOM.setNodeValue succeeded");
        Ok(())
    }

    /// `DOM.setAttributeValue` — set an attribute on a DOM node.
    ///
    /// `node_id` is the DOM node ID.
    /// `name` is the attribute name.
    /// `value` is the attribute value.
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if the CDP call fails.
    pub async fn set_attribute_value(
        &self,
        node_id: i64,
        name: &str,
        value: &str,
        timeout: Duration,
    ) -> Result<(), CdpError> {
        let params = json!({
            "nodeId": node_id,
            "name": name,
            "value": value,
        });
        self.call("DOM.setAttributeValue", params, timeout).await?;
        debug!(?node_id, %name, %value, "DOM.setAttributeValue succeeded");
        Ok(())
    }

    /// `DOM.removeAttribute` — remove an attribute from a DOM node.
    ///
    /// `node_id` is the DOM node ID.
    /// `name` is the attribute name to remove.
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if the CDP call fails.
    pub async fn remove_attribute(
        &self,
        node_id: i64,
        name: &str,
        timeout: Duration,
    ) -> Result<(), CdpError> {
        let params = json!({
            "nodeId": node_id,
            "name": name,
        });
        self.call("DOM.removeAttribute", params, timeout).await?;
        debug!(?node_id, %name, "DOM.removeAttribute succeeded");
        Ok(())
    }
}
