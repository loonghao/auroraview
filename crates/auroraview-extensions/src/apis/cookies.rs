//! Chrome Cookies API Implementation
//!
//! Provides cookie management functionality for extensions.
//!
//! ## Features
//! - Get, set, remove cookies
//! - Query cookies by domain, name, path
//! - Cookie store management
//! - Event notifications for changes

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::error::{ExtensionError, ExtensionResult};

/// Same-site status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum SameSiteStatus {
    NoRestriction,
    Lax,
    Strict,
    #[default]
    Unspecified,
}

/// Cookie object
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cookie {
    /// Cookie name
    pub name: String,
    /// Cookie value
    pub value: String,
    /// Domain
    pub domain: String,
    /// Whether host-only
    pub host_only: bool,
    /// Path
    pub path: String,
    /// Whether secure
    pub secure: bool,
    /// Whether HTTP only
    pub http_only: bool,
    /// Same-site status
    pub same_site: SameSiteStatus,
    /// Whether session cookie
    pub session: bool,
    /// Expiration date (seconds since epoch)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_date: Option<f64>,
    /// Cookie store ID
    pub store_id: String,
}

/// Cookie details for get/remove
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CookieDetails {
    /// URL associated with the cookie
    pub url: String,
    /// Cookie name
    pub name: String,
    /// Cookie store ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_id: Option<String>,
}

/// Cookie set details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetDetails {
    /// URL associated with the cookie
    pub url: String,
    /// Cookie name
    pub name: String,
    /// Cookie value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Domain
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// Path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Whether secure
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secure: Option<bool>,
    /// Whether HTTP only
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_only: Option<bool>,
    /// Same-site status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub same_site: Option<SameSiteStatus>,
    /// Expiration date (seconds since epoch)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_date: Option<f64>,
    /// Cookie store ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_id: Option<String>,
}

/// Cookie query
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CookieQuery {
    /// URL to match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Name to match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Domain to match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// Path to match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Secure filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secure: Option<bool>,
    /// Session filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<bool>,
    /// Cookie store ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_id: Option<String>,
}

/// Cookie store
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CookieStore {
    /// Store ID
    pub id: String,
    /// Tab IDs using this store
    pub tab_ids: Vec<i32>,
    /// Whether incognito
    pub incognito: bool,
}

/// Cookie key for storage
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct CookieKey {
    domain: String,
    path: String,
    name: String,
    store_id: String,
}

/// Cookies API handler
pub struct CookiesApi {
    /// In-memory cookie storage
    cookies: Arc<RwLock<HashMap<CookieKey, Cookie>>>,
}

impl Default for CookiesApi {
    fn default() -> Self {
        Self::new()
    }
}

impl CookiesApi {
    /// Create a new CookiesApi instance
    pub fn new() -> Self {
        Self {
            cookies: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Extract domain from URL
    fn domain_from_url(url: &str) -> Option<String> {
        url::Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|s| s.to_string()))
    }

    /// Extract path from URL
    fn path_from_url(url: &str) -> String {
        url::Url::parse(url)
            .ok()
            .map(|u| u.path().to_string())
            .unwrap_or_else(|| "/".to_string())
    }

    /// Get a cookie
    pub fn get(&self, details: CookieDetails) -> ExtensionResult<Value> {
        let cookies = self.cookies.read().unwrap();
        let store_id = details.store_id.unwrap_or_else(|| "0".to_string());

        let domain = Self::domain_from_url(&details.url)
            .ok_or_else(|| ExtensionError::InvalidParams("Invalid URL".into()))?;

        let key = CookieKey {
            domain: domain.clone(),
            path: "/".to_string(),
            name: details.name.clone(),
            store_id: store_id.clone(),
        };

        // Try exact match first
        if let Some(cookie) = cookies.get(&key) {
            return Ok(serde_json::to_value(cookie)?);
        }

        // Try with domain prefix (for subdomain cookies)
        for (k, cookie) in cookies.iter() {
            if k.name == details.name
                && k.store_id == store_id
                && (k.domain == domain || domain.ends_with(&format!(".{}", k.domain)))
            {
                return Ok(serde_json::to_value(cookie)?);
            }
        }

        Ok(json!(null))
    }

    /// Get all cookies matching query
    pub fn get_all(&self, query: CookieQuery) -> ExtensionResult<Value> {
        let cookies = self.cookies.read().unwrap();
        let store_id = query.store_id.clone().unwrap_or_else(|| "0".to_string());

        let url_domain = query.url.as_ref().and_then(|u| Self::domain_from_url(u));

        let results: Vec<&Cookie> = cookies
            .values()
            .filter(|cookie| {
                // Store ID filter
                if cookie.store_id != store_id {
                    return false;
                }

                // Name filter
                if let Some(ref name) = query.name {
                    if &cookie.name != name {
                        return false;
                    }
                }

                // Domain filter
                if let Some(ref domain) = query.domain {
                    if &cookie.domain != domain && !cookie.domain.ends_with(&format!(".{}", domain))
                    {
                        return false;
                    }
                }

                // URL domain filter
                if let Some(ref url_domain) = url_domain {
                    if &cookie.domain != url_domain
                        && !url_domain.ends_with(&format!(".{}", cookie.domain))
                    {
                        return false;
                    }
                }

                // Path filter
                if let Some(ref path) = query.path {
                    if &cookie.path != path {
                        return false;
                    }
                }

                // Secure filter
                if let Some(secure) = query.secure {
                    if cookie.secure != secure {
                        return false;
                    }
                }

                // Session filter
                if let Some(session) = query.session {
                    if cookie.session != session {
                        return false;
                    }
                }

                true
            })
            .collect();

        Ok(serde_json::to_value(results)?)
    }

    /// Set a cookie
    pub fn set(&self, details: SetDetails) -> ExtensionResult<Value> {
        let mut cookies = self.cookies.write().unwrap();
        let store_id = details.store_id.unwrap_or_else(|| "0".to_string());

        let domain = details
            .domain
            .clone()
            .or_else(|| Self::domain_from_url(&details.url))
            .ok_or_else(|| ExtensionError::InvalidParams("Invalid URL or domain".into()))?;

        let path = details
            .path
            .clone()
            .unwrap_or_else(|| Self::path_from_url(&details.url));

        let cookie = Cookie {
            name: details.name.clone(),
            value: details.value.unwrap_or_default(),
            domain: domain.clone(),
            host_only: !domain.starts_with('.'),
            path: path.clone(),
            secure: details.secure.unwrap_or(false),
            http_only: details.http_only.unwrap_or(false),
            same_site: details.same_site.unwrap_or_default(),
            session: details.expiration_date.is_none(),
            expiration_date: details.expiration_date,
            store_id: store_id.clone(),
        };

        let key = CookieKey {
            domain,
            path,
            name: details.name,
            store_id,
        };

        cookies.insert(key, cookie.clone());
        Ok(serde_json::to_value(cookie)?)
    }

    /// Remove a cookie
    pub fn remove(&self, details: CookieDetails) -> ExtensionResult<Value> {
        let mut cookies = self.cookies.write().unwrap();
        let store_id = details.store_id.unwrap_or_else(|| "0".to_string());

        let domain = Self::domain_from_url(&details.url)
            .ok_or_else(|| ExtensionError::InvalidParams("Invalid URL".into()))?;

        // Find and remove matching cookie
        let key_to_remove: Option<CookieKey> = cookies
            .keys()
            .find(|k| {
                k.name == details.name
                    && k.store_id == store_id
                    && (k.domain == domain || domain.ends_with(&format!(".{}", k.domain)))
            })
            .cloned();

        if let Some(key) = key_to_remove {
            cookies.remove(&key);
            Ok(json!({
                "url": details.url,
                "name": details.name,
                "storeId": store_id
            }))
        } else {
            Ok(json!(null))
        }
    }

    /// Get all cookie stores
    pub fn get_all_cookie_stores(&self) -> ExtensionResult<Value> {
        // Return default store
        let stores = vec![CookieStore {
            id: "0".to_string(),
            tab_ids: vec![],
            incognito: false,
        }];
        Ok(serde_json::to_value(stores)?)
    }

    /// Handle API call
    pub fn handle(&self, method: &str, params: Value) -> ExtensionResult<Value> {
        match method {
            "get" => {
                let details: CookieDetails = serde_json::from_value(params)?;
                self.get(details)
            }
            "getAll" => {
                let query: CookieQuery = serde_json::from_value(params).unwrap_or_default();
                self.get_all(query)
            }
            "set" => {
                let details: SetDetails = serde_json::from_value(params)?;
                self.set(details)
            }
            "remove" => {
                let details: CookieDetails = serde_json::from_value(params)?;
                self.remove(details)
            }
            "getAllCookieStores" => self.get_all_cookie_stores(),
            _ => Err(ExtensionError::UnknownMethod(method.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get_cookie() {
        let api = CookiesApi::new();

        // Set cookie
        api.set(SetDetails {
            url: "https://example.com".to_string(),
            name: "test".to_string(),
            value: Some("value".to_string()),
            domain: None,
            path: None,
            secure: Some(true),
            http_only: None,
            same_site: None,
            expiration_date: None,
            store_id: None,
        })
        .unwrap();

        // Get cookie
        let result = api
            .get(CookieDetails {
                url: "https://example.com".to_string(),
                name: "test".to_string(),
                store_id: None,
            })
            .unwrap();

        let cookie: Cookie = serde_json::from_value(result).unwrap();
        assert_eq!(cookie.name, "test");
        assert_eq!(cookie.value, "value");
    }

    #[test]
    fn test_remove_cookie() {
        let api = CookiesApi::new();

        api.set(SetDetails {
            url: "https://example.com".to_string(),
            name: "test".to_string(),
            value: Some("value".to_string()),
            domain: None,
            path: None,
            secure: None,
            http_only: None,
            same_site: None,
            expiration_date: None,
            store_id: None,
        })
        .unwrap();

        api.remove(CookieDetails {
            url: "https://example.com".to_string(),
            name: "test".to_string(),
            store_id: None,
        })
        .unwrap();

        let result = api
            .get(CookieDetails {
                url: "https://example.com".to_string(),
                name: "test".to_string(),
                store_id: None,
            })
            .unwrap();

        assert!(result.is_null());
    }
}
