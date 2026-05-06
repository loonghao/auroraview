//! Runtime.* CDP methods.
//!
//! This module provides JavaScript runtime CDP methods:
//! - `evaluate_script`
//! - `get_properties`
//! - `call_function_on`

use std::time::Duration;

use serde_json::{json, Value};
use tracing::debug;

use super::CdpClient;
use crate::cdp::CdpError;

impl CdpClient {
    /// `Runtime.evaluate` — execute JavaScript and return the result.
    ///
    /// Returns the JSON value of the expression result.
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if the CDP call fails or the response is malformed.
    #[tracing::instrument(skip(self, timeout), fields(script_len = script.len(), timeout_ms = ?timeout.as_millis()))]
    pub async fn evaluate_script(
        &self,
        script: &str,
        timeout: Duration,
    ) -> Result<Value, CdpError> {
        let params = json!({
            "expression": script,
            "returnByValue": true,
            "awaitPromise": true,
        });
        let result = self.call("Runtime.evaluate", params, timeout).await?;
        let value = result
            .get("result")
            .and_then(|v| v.get("value"))
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        Ok(value)
    }

    /// `Runtime.getProperties` — get object properties (for inspecting JS objects).
    ///
    /// `object_id` is the unique object ID (from `Runtime.evaluate` result with `objectId`).
    /// Returns a list of property descriptors.
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if the CDP call fails or the response is malformed.
    pub async fn get_properties(
        &self,
        object_id: &str,
        timeout: Duration,
    ) -> Result<Vec<Value>, CdpError> {
        let params = json!({
            "objectId": object_id,
            "ownProperties": true,
        });
        // Use retry logic for this idempotent method
        let result = self
            .call_with_retry(
                "Runtime.getProperties",
                params,
                timeout,
                3,
                Duration::from_millis(100),
                Duration::from_secs(5),
            )
            .await?;
        let props = result
            .get("result")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        debug!(
            ?object_id,
            count = props.len(),
            "Runtime.getProperties succeeded"
        );
        Ok(props)
    }

    /// `Runtime.callFunctionOn` — call a JavaScript function on a remote object.
    ///
    /// `object_id` is the unique object ID (from `Runtime.evaluate` with `objectId`).
    /// `function_declaration` is the JS function to call (e.g., `"function() { return this.length; }"`).
    /// `arguments` is optional array of call arguments.
    /// Returns the JSON value result.
    ///
    /// # Errors
    ///
    /// Returns [`CdpError`] if the CDP call fails or the response is malformed.
    pub async fn call_function_on(
        &self,
        object_id: &str,
        function_declaration: &str,
        arguments: Option<&[Value]>,
        timeout: Duration,
    ) -> Result<Value, CdpError> {
        let mut params = json!({
            "objectId": object_id,
            "functionDeclaration": function_declaration,
            "returnByValue": true,
            "awaitPromise": true,
        });
        if let Some(args) = arguments {
            params["arguments"] = serde_json::json!(args);
        }
        let result = self.call("Runtime.callFunctionOn", params, timeout).await?;
        let value = result
            .get("result")
            .and_then(|v| v.get("value"))
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        debug!(
            ?object_id,
            ?function_declaration,
            "Runtime.callFunctionOn succeeded"
        );
        Ok(value)
    }
}
