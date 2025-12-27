//! Chrome Downloads API Implementation
//!
//! Provides download management functionality for extensions.
//!
//! ## Features
//! - Initiate downloads
//! - Monitor download progress
//! - Search and manage downloads
//! - Event notifications

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::error::{ExtensionError, ExtensionResult};

/// Download state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum DownloadState {
    #[default]
    InProgress,
    Interrupted,
    Complete,
}

/// Danger type for downloads
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum DangerType {
    File,
    Url,
    Content,
    Uncommon,
    Host,
    Unwanted,
    #[default]
    Safe,
    Accepted,
}

/// Interrupt reason
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InterruptReason {
    FileFailed,
    FileAccessDenied,
    FileNoSpace,
    FileNameTooLong,
    FileTooLarge,
    FileVirusInfected,
    FileTransientError,
    FileBlocked,
    FileSecurityCheckFailed,
    FileTooShort,
    NetworkFailed,
    NetworkTimeout,
    NetworkDisconnected,
    NetworkServerDown,
    NetworkInvalidRequest,
    ServerFailed,
    ServerNoRange,
    ServerBadContent,
    ServerUnauthorized,
    ServerCertProblem,
    ServerForbidden,
    ServerUnreachable,
    UserCanceled,
    UserShutdown,
    Crash,
}

/// Download item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadItem {
    /// Download ID
    pub id: i32,
    /// URL being downloaded
    pub url: String,
    /// Final URL after redirects
    pub final_url: String,
    /// Referrer URL
    pub referrer: String,
    /// Local filename
    pub filename: String,
    /// Whether download is in incognito mode
    pub incognito: bool,
    /// Danger type
    pub danger: DangerType,
    /// MIME type
    pub mime: String,
    /// Start time (ISO 8601)
    pub start_time: String,
    /// End time (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
    /// Estimated end time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_end_time: Option<String>,
    /// Current state
    pub state: DownloadState,
    /// Whether paused
    pub paused: bool,
    /// Whether can resume
    pub can_resume: bool,
    /// Error reason
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<InterruptReason>,
    /// Bytes received
    pub bytes_received: i64,
    /// Total bytes (-1 if unknown)
    pub total_bytes: i64,
    /// File size after decompression
    pub file_size: i64,
    /// Whether file exists
    pub exists: bool,
    /// Extension that initiated download
    #[serde(skip_serializing_if = "Option::is_none")]
    pub by_extension_id: Option<String>,
    /// Extension name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub by_extension_name: Option<String>,
}

/// Download options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadOptions {
    /// URL to download
    pub url: String,
    /// Filename to save as
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    /// Conflict action
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflict_action: Option<FilenameConflictAction>,
    /// Whether to show save as dialog
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save_as: Option<bool>,
    /// HTTP method
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<HttpMethod>,
    /// HTTP headers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<HeaderNameValuePair>>,
    /// POST body
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

/// Filename conflict action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilenameConflictAction {
    Uniquify,
    Overwrite,
    Prompt,
}

/// HTTP method
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HttpMethod {
    Get,
    Post,
}

/// Header name-value pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderNameValuePair {
    pub name: String,
    pub value: String,
}

/// Download query
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DownloadQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_regex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename_regex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<DownloadState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub danger: Option<DangerType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_after: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_after: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_bytes_greater: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_bytes_less: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes_received: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exists: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<Vec<String>>,
}

/// Downloads API handler
pub struct DownloadsApi {
    /// In-memory download storage
    downloads: Arc<RwLock<HashMap<i32, DownloadItem>>>,
    /// Next ID counter
    next_id: Arc<RwLock<i32>>,
}

impl Default for DownloadsApi {
    fn default() -> Self {
        Self::new()
    }
}

impl DownloadsApi {
    /// Create a new DownloadsApi instance
    pub fn new() -> Self {
        Self {
            downloads: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
        }
    }

    /// Generate next ID
    fn next_id(&self) -> i32 {
        let mut id = self.next_id.write().unwrap();
        let current = *id;
        *id += 1;
        current
    }

    /// Initiate a download
    pub fn download(
        &self,
        options: DownloadOptions,
        extension_id: Option<&str>,
    ) -> ExtensionResult<Value> {
        let id = self.next_id();
        let now = chrono::Utc::now().to_rfc3339();

        // Extract filename from URL if not provided
        let filename = options.filename.unwrap_or_else(|| {
            options
                .url
                .split('/')
                .next_back()
                .unwrap_or("download")
                .to_string()
        });

        let item = DownloadItem {
            id,
            url: options.url.clone(),
            final_url: options.url,
            referrer: String::new(),
            filename,
            incognito: false,
            danger: DangerType::Safe,
            mime: "application/octet-stream".to_string(),
            start_time: now,
            end_time: None,
            estimated_end_time: None,
            state: DownloadState::InProgress,
            paused: false,
            can_resume: true,
            error: None,
            bytes_received: 0,
            total_bytes: -1,
            file_size: -1,
            exists: false,
            by_extension_id: extension_id.map(|s| s.to_string()),
            by_extension_name: None,
        };

        let mut downloads = self.downloads.write().unwrap();
        downloads.insert(id, item);

        // In a real implementation, we would start the actual download here
        // For now, we just return the ID

        Ok(json!(id))
    }

    /// Search downloads
    pub fn search(&self, query: DownloadQuery) -> ExtensionResult<Value> {
        let downloads = self.downloads.read().unwrap();
        let limit = query.limit.unwrap_or(1000) as usize;

        let mut results: Vec<&DownloadItem> = downloads
            .values()
            .filter(|item| {
                // ID filter
                if let Some(id) = query.id {
                    if item.id != id {
                        return false;
                    }
                }

                // URL filter
                if let Some(ref url) = query.url {
                    if &item.url != url {
                        return false;
                    }
                }

                // Filename filter
                if let Some(ref filename) = query.filename {
                    if &item.filename != filename {
                        return false;
                    }
                }

                // State filter
                if let Some(ref state) = query.state {
                    if &item.state != state {
                        return false;
                    }
                }

                // Paused filter
                if let Some(paused) = query.paused {
                    if item.paused != paused {
                        return false;
                    }
                }

                // Exists filter
                if let Some(exists) = query.exists {
                    if item.exists != exists {
                        return false;
                    }
                }

                // Query text filter
                if let Some(ref queries) = query.query {
                    for q in queries {
                        if let Some(q) = q.strip_prefix('-') {
                            // Negative match
                            if item.filename.contains(q) || item.url.contains(q) {
                                return false;
                            }
                        } else {
                            // Positive match
                            if !item.filename.contains(q) && !item.url.contains(q) {
                                return false;
                            }
                        }
                    }
                }

                true
            })
            .collect();

        // Sort by start time descending by default
        results.sort_by(|a, b| b.start_time.cmp(&a.start_time));
        results.truncate(limit);

        Ok(serde_json::to_value(results)?)
    }

    /// Pause a download
    pub fn pause(&self, download_id: i32) -> ExtensionResult<Value> {
        let mut downloads = self.downloads.write().unwrap();

        let item = downloads.get_mut(&download_id).ok_or_else(|| {
            ExtensionError::NotFound(format!("Download {} not found", download_id))
        })?;

        if item.state != DownloadState::InProgress {
            return Err(ExtensionError::InvalidParams(
                "Can only pause in-progress downloads".into(),
            ));
        }

        item.paused = true;
        Ok(json!(null))
    }

    /// Resume a download
    pub fn resume(&self, download_id: i32) -> ExtensionResult<Value> {
        let mut downloads = self.downloads.write().unwrap();

        let item = downloads.get_mut(&download_id).ok_or_else(|| {
            ExtensionError::NotFound(format!("Download {} not found", download_id))
        })?;

        if !item.can_resume {
            return Err(ExtensionError::InvalidParams(
                "Download cannot be resumed".into(),
            ));
        }

        item.paused = false;
        Ok(json!(null))
    }

    /// Cancel a download
    pub fn cancel(&self, download_id: i32) -> ExtensionResult<Value> {
        let mut downloads = self.downloads.write().unwrap();

        let item = downloads.get_mut(&download_id).ok_or_else(|| {
            ExtensionError::NotFound(format!("Download {} not found", download_id))
        })?;

        item.state = DownloadState::Interrupted;
        item.error = Some(InterruptReason::UserCanceled);
        item.can_resume = false;

        Ok(json!(null))
    }

    /// Get file icon
    pub fn get_file_icon(
        &self,
        download_id: i32,
        _options: Option<Value>,
    ) -> ExtensionResult<Value> {
        let downloads = self.downloads.read().unwrap();

        let _item = downloads.get(&download_id).ok_or_else(|| {
            ExtensionError::NotFound(format!("Download {} not found", download_id))
        })?;

        // Return a placeholder data URL for the icon
        // In a real implementation, this would get the actual file icon
        Ok(json!("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="))
    }

    /// Open downloaded file
    pub fn open(&self, download_id: i32) -> ExtensionResult<Value> {
        let downloads = self.downloads.read().unwrap();

        let item = downloads.get(&download_id).ok_or_else(|| {
            ExtensionError::NotFound(format!("Download {} not found", download_id))
        })?;

        if item.state != DownloadState::Complete {
            return Err(ExtensionError::InvalidParams(
                "Can only open completed downloads".into(),
            ));
        }

        // In a real implementation, we would open the file with the default application
        // For now, just return success
        Ok(json!(null))
    }

    /// Show downloaded file in folder
    pub fn show(&self, download_id: i32) -> ExtensionResult<Value> {
        let downloads = self.downloads.read().unwrap();

        let _item = downloads.get(&download_id).ok_or_else(|| {
            ExtensionError::NotFound(format!("Download {} not found", download_id))
        })?;

        // In a real implementation, we would open the containing folder
        Ok(json!(null))
    }

    /// Show default downloads folder
    pub fn show_default_folder(&self) -> ExtensionResult<Value> {
        // In a real implementation, we would open the downloads folder
        Ok(json!(null))
    }

    /// Erase downloads from history
    pub fn erase(&self, query: DownloadQuery) -> ExtensionResult<Value> {
        let mut downloads = self.downloads.write().unwrap();

        let ids_to_remove: Vec<i32> = downloads
            .iter()
            .filter(|(_, item)| {
                // Apply same filters as search
                if let Some(id) = query.id {
                    if item.id != id {
                        return false;
                    }
                }
                if let Some(ref state) = query.state {
                    if &item.state != state {
                        return false;
                    }
                }
                true
            })
            .map(|(id, _)| *id)
            .collect();

        for id in &ids_to_remove {
            downloads.remove(id);
        }

        Ok(serde_json::to_value(ids_to_remove)?)
    }

    /// Remove downloaded file
    pub fn remove_file(&self, download_id: i32) -> ExtensionResult<Value> {
        let downloads = self.downloads.read().unwrap();

        let item = downloads.get(&download_id).ok_or_else(|| {
            ExtensionError::NotFound(format!("Download {} not found", download_id))
        })?;

        if item.state != DownloadState::Complete {
            return Err(ExtensionError::InvalidParams(
                "Can only remove completed downloads".into(),
            ));
        }

        // In a real implementation, we would delete the actual file
        Ok(json!(null))
    }

    /// Accept dangerous download
    pub fn accept_danger(&self, download_id: i32) -> ExtensionResult<Value> {
        let mut downloads = self.downloads.write().unwrap();

        let item = downloads.get_mut(&download_id).ok_or_else(|| {
            ExtensionError::NotFound(format!("Download {} not found", download_id))
        })?;

        item.danger = DangerType::Accepted;
        Ok(json!(null))
    }

    /// Set UI options
    pub fn set_ui_options(&self, _options: Value) -> ExtensionResult<Value> {
        // In a real implementation, this would control the download shelf
        Ok(json!(null))
    }

    /// Handle API call
    pub fn handle(
        &self,
        method: &str,
        params: Value,
        extension_id: &str,
    ) -> ExtensionResult<Value> {
        match method {
            "download" => {
                let options: DownloadOptions = serde_json::from_value(params)?;
                self.download(options, Some(extension_id))
            }
            "search" => {
                let query: DownloadQuery = serde_json::from_value(params).unwrap_or_default();
                self.search(query)
            }
            "pause" => {
                let id = params
                    .get("downloadId")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing downloadId".into()))?
                    as i32;
                self.pause(id)
            }
            "resume" => {
                let id = params
                    .get("downloadId")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing downloadId".into()))?
                    as i32;
                self.resume(id)
            }
            "cancel" => {
                let id = params
                    .get("downloadId")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing downloadId".into()))?
                    as i32;
                self.cancel(id)
            }
            "getFileIcon" => {
                let id = params
                    .get("downloadId")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing downloadId".into()))?
                    as i32;
                let options = params.get("options").cloned();
                self.get_file_icon(id, options)
            }
            "open" => {
                let id = params
                    .get("downloadId")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing downloadId".into()))?
                    as i32;
                self.open(id)
            }
            "show" => {
                let id = params
                    .get("downloadId")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing downloadId".into()))?
                    as i32;
                self.show(id)
            }
            "showDefaultFolder" => self.show_default_folder(),
            "erase" => {
                let query: DownloadQuery = serde_json::from_value(params).unwrap_or_default();
                self.erase(query)
            }
            "removeFile" => {
                let id = params
                    .get("downloadId")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing downloadId".into()))?
                    as i32;
                self.remove_file(id)
            }
            "acceptDanger" => {
                let id = params
                    .get("downloadId")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| ExtensionError::InvalidParams("Missing downloadId".into()))?
                    as i32;
                self.accept_danger(id)
            }
            "setUiOptions" => self.set_ui_options(params),
            _ => Err(ExtensionError::UnknownMethod(method.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download() {
        let api = DownloadsApi::new();
        let result = api
            .download(
                DownloadOptions {
                    url: "https://example.com/file.zip".to_string(),
                    filename: None,
                    conflict_action: None,
                    save_as: None,
                    method: None,
                    headers: None,
                    body: None,
                },
                Some("test-extension"),
            )
            .unwrap();

        let id: i32 = serde_json::from_value(result).unwrap();
        assert!(id > 0);
    }

    #[test]
    fn test_search() {
        let api = DownloadsApi::new();

        api.download(
            DownloadOptions {
                url: "https://example.com/file.zip".to_string(),
                filename: None,
                conflict_action: None,
                save_as: None,
                method: None,
                headers: None,
                body: None,
            },
            None,
        )
        .unwrap();

        let result = api.search(DownloadQuery::default()).unwrap();
        let items: Vec<DownloadItem> = serde_json::from_value(result).unwrap();
        assert_eq!(items.len(), 1);
    }
}
