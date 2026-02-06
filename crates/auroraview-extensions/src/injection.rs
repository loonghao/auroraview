//! Script Injection - Content script management
//!
//! Handles injection of content scripts into web pages based on
//! manifest configuration and URL matching.

use parking_lot::RwLock;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::{ExtensionError, ExtensionResult};
use crate::host::LoadedExtension;
use crate::manifest::{ContentScriptConfig, RunAt, ScriptWorld};
use crate::ExtensionId;

/// Compiled content script ready for injection
#[derive(Debug, Clone)]
pub struct CompiledContentScript {
    /// Extension ID
    pub extension_id: ExtensionId,
    /// URL patterns to match (compiled regex)
    pub match_patterns: Vec<Regex>,
    /// URL patterns to exclude (compiled regex)
    pub exclude_patterns: Vec<Regex>,
    /// JavaScript code to inject
    pub js_code: String,
    /// CSS code to inject
    pub css_code: String,
    /// When to run
    pub run_at: RunAt,
    /// Whether to run in all frames
    pub all_frames: bool,
    /// Script world
    pub world: ScriptWorld,
}

/// Script injector - manages content script injection
pub struct ScriptInjector {
    /// Compiled content scripts by extension
    scripts: Arc<RwLock<HashMap<ExtensionId, Vec<CompiledContentScript>>>>,
}

impl ScriptInjector {
    /// Create a new script injector
    pub fn new() -> Self {
        Self {
            scripts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register content scripts for an extension
    pub fn register_extension(&self, extension: &LoadedExtension) -> ExtensionResult<()> {
        let mut compiled_scripts = Vec::new();

        for config in &extension.manifest.content_scripts {
            let compiled = self.compile_content_script(extension, config)?;
            compiled_scripts.push(compiled);
        }

        if !compiled_scripts.is_empty() {
            let mut scripts = self.scripts.write();
            scripts.insert(extension.id.clone(), compiled_scripts);
            tracing::info!(
                "Registered {} content scripts for extension: {}",
                extension.manifest.content_scripts.len(),
                extension.id
            );
        }

        Ok(())
    }

    /// Unregister content scripts for an extension
    pub fn unregister_extension(&self, extension_id: &str) {
        let mut scripts = self.scripts.write();
        scripts.remove(extension_id);
    }

    /// Compile a content script configuration
    fn compile_content_script(
        &self,
        extension: &LoadedExtension,
        config: &ContentScriptConfig,
    ) -> ExtensionResult<CompiledContentScript> {
        // Compile match patterns
        let match_patterns: Vec<Regex> = config
            .matches
            .iter()
            .filter_map(|pattern| compile_url_pattern(pattern).ok())
            .collect();

        // Compile exclude patterns
        let exclude_patterns: Vec<Regex> = config
            .exclude_matches
            .iter()
            .filter_map(|pattern| compile_url_pattern(pattern).ok())
            .collect();

        // Load JavaScript files
        let mut js_code = String::new();
        for js_file in &config.js {
            match extension.read_resource(js_file) {
                Ok(code) => {
                    js_code.push_str(&code);
                    js_code.push('\n');
                }
                Err(e) => {
                    tracing::warn!("Failed to load JS file {}: {}", js_file, e);
                }
            }
        }

        // Load CSS files
        let mut css_code = String::new();
        for css_file in &config.css {
            match extension.read_resource(css_file) {
                Ok(code) => {
                    css_code.push_str(&code);
                    css_code.push('\n');
                }
                Err(e) => {
                    tracing::warn!("Failed to load CSS file {}: {}", css_file, e);
                }
            }
        }

        Ok(CompiledContentScript {
            extension_id: extension.id.clone(),
            match_patterns,
            exclude_patterns,
            js_code,
            css_code,
            run_at: config.run_at.clone().unwrap_or_default(),
            all_frames: config.all_frames,
            world: config.world.clone().unwrap_or_default(),
        })
    }

    /// Get scripts to inject for a URL
    pub fn get_scripts_for_url(&self, url: &str, run_at: &RunAt) -> Vec<CompiledContentScript> {
        let scripts = self.scripts.read();
        let mut result = Vec::new();

        for extension_scripts in scripts.values() {
            for script in extension_scripts {
                // Check run_at timing
                if std::mem::discriminant(&script.run_at) != std::mem::discriminant(run_at) {
                    continue;
                }

                // Check if URL matches
                if !self.url_matches_script(url, script) {
                    continue;
                }

                result.push(script.clone());
            }
        }

        result
    }

    /// Check if a URL matches a content script's patterns
    fn url_matches_script(&self, url: &str, script: &CompiledContentScript) -> bool {
        // Check exclude patterns first
        for pattern in &script.exclude_patterns {
            if pattern.is_match(url) {
                return false;
            }
        }

        // Check match patterns
        for pattern in &script.match_patterns {
            if pattern.is_match(url) {
                return true;
            }
        }

        false
    }

    /// Generate injection code for a URL
    pub fn generate_injection_code(&self, url: &str, run_at: &RunAt) -> String {
        let scripts = self.get_scripts_for_url(url, run_at);

        if scripts.is_empty() {
            return String::new();
        }

        let mut code = String::new();

        // Inject CSS first
        for script in &scripts {
            if !script.css_code.is_empty() {
                code.push_str(&format!(
                    r#"
(function() {{
    const style = document.createElement('style');
    style.textContent = {};
    (document.head || document.documentElement).appendChild(style);
}})();
"#,
                    serde_json::to_string(&script.css_code).unwrap_or_default()
                ));
            }
        }

        // Then inject JavaScript
        for script in &scripts {
            if !script.js_code.is_empty() {
                // Wrap in IIFE for isolation (unless main world)
                let wrapped = match script.world {
                    ScriptWorld::Isolated => format!(
                        r#"
(function() {{
    'use strict';
    // Content script from extension: {}
    {}
}})();
"#,
                        script.extension_id, script.js_code
                    ),
                    ScriptWorld::Main => script.js_code.clone(),
                };
                code.push_str(&wrapped);
            }
        }

        code
    }
}

impl Default for ScriptInjector {
    fn default() -> Self {
        Self::new()
    }
}

/// Compile a Chrome extension URL pattern to a regex
fn compile_url_pattern(pattern: &str) -> ExtensionResult<Regex> {
    // Handle special patterns
    if pattern == "<all_urls>" {
        return Regex::new(r"^(https?|file|ftp)://.*$")
            .map_err(|e| ExtensionError::InvalidArgument(e.to_string()));
    }

    // Parse pattern: scheme://host/path
    // Convert to regex:
    // - * in scheme matches http or https
    // - *. in host matches any subdomain
    // - * in host matches any host
    // - * in path matches any path

    let mut regex_str = String::from("^");

    // Handle scheme
    if pattern.starts_with("*://") {
        regex_str.push_str(r"https?://");
    } else if let Some(idx) = pattern.find("://") {
        let scheme = &pattern[..idx];
        regex_str.push_str(&regex::escape(scheme));
        regex_str.push_str("://");
    } else {
        return Err(ExtensionError::InvalidArgument(format!(
            "Invalid URL pattern: {}",
            pattern
        )));
    }

    // Get the rest after scheme
    let rest = if let Some(stripped) = pattern.strip_prefix("*://") {
        stripped
    } else if let Some(idx) = pattern.find("://") {
        &pattern[idx + 3..]
    } else {
        pattern
    };

    // Split host and path
    let (host, path) = if let Some(idx) = rest.find('/') {
        (&rest[..idx], &rest[idx..])
    } else {
        (rest, "/*")
    };

    // Handle host
    if host == "*" {
        regex_str.push_str(r"[^/]+");
    } else if let Some(domain) = host.strip_prefix("*.") {
        // *.example.com matches example.com and any subdomain
        regex_str.push_str(&format!(r"([^/]+\.)?{}", regex::escape(domain)));
    } else {
        // Handle port wildcards like localhost:*
        let escaped = regex::escape(host).replace(r"\*", r"[^/]*");
        regex_str.push_str(&escaped);
    }

    // Handle path
    if path == "/*" || path == "/" {
        regex_str.push_str(r"(/.*)?");
    } else {
        let escaped = regex::escape(path).replace(r"\*", r".*");
        regex_str.push_str(&escaped);
    }

    regex_str.push('$');

    Regex::new(&regex_str).map_err(|e| ExtensionError::InvalidArgument(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_pattern_all_urls() {
        let re = compile_url_pattern("<all_urls>").unwrap();
        assert!(re.is_match("https://example.com/page"));
        assert!(re.is_match("http://localhost:3000/api"));
        assert!(re.is_match("file:///path/to/file.html"));
    }

    #[test]
    fn test_url_pattern_wildcard_scheme() {
        let re = compile_url_pattern("*://*.example.com/*").unwrap();
        assert!(re.is_match("https://example.com/page"));
        assert!(re.is_match("http://sub.example.com/page"));
        assert!(re.is_match("https://deep.sub.example.com/"));
        assert!(!re.is_match("https://notexample.com/"));
    }

    #[test]
    fn test_url_pattern_localhost() {
        let re = compile_url_pattern("http://localhost:*/*").unwrap();
        assert!(re.is_match("http://localhost:3000/api"));
        assert!(re.is_match("http://localhost:8080/"));
        assert!(!re.is_match("https://localhost:3000/"));
    }

    #[test]
    fn test_url_pattern_specific() {
        let re = compile_url_pattern("https://specific.site.com/page/*").unwrap();
        assert!(re.is_match("https://specific.site.com/page/"));
        assert!(re.is_match("https://specific.site.com/page/sub"));
        assert!(!re.is_match("https://specific.site.com/other"));
    }
}
