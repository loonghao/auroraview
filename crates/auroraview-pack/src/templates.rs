//! Askama templates for code generation
//!
//! Uses compile-time type-safe templates for generating Rust code.

use askama::Template;

/// Template for Cargo.toml
#[derive(Template)]
#[template(path = "cargo_toml.txt")]
pub struct CargoTomlTemplate<'a> {
    pub name: &'a str,
    pub embed_assets: bool,
}

/// Template for URL mode main.rs
#[derive(Template)]
#[template(path = "main_url.rs.txt")]
pub struct MainUrlTemplate<'a> {
    pub title: &'a str,
    pub url: &'a str,
    pub width: u32,
    pub height: u32,
}

/// Template for frontend mode main.rs (with embedded assets)
#[derive(Template)]
#[template(path = "main_frontend.rs.txt")]
pub struct MainFrontendTemplate<'a> {
    pub title: &'a str,
    pub width: u32,
    pub height: u32,
}

/// Template for fullstack mode main.rs (with embedded assets + Python)
#[derive(Template)]
#[template(path = "main_fullstack.rs.txt")]
pub struct MainFullstackTemplate<'a> {
    pub title: &'a str,
    pub width: u32,
    pub height: u32,
    pub backend_entry: bool,
    pub backend_module: &'a str,
    pub backend_func: &'a str,
}

/// Template for Cargo.toml with PyEmbed
#[derive(Template)]
#[template(path = "cargo_toml_pyembed.txt")]
pub struct CargoTomlPyembedTemplate<'a> {
    pub name: &'a str,
    pub embed_assets: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cargo_toml_template() {
        let template = CargoTomlTemplate {
            name: "test-app",
            embed_assets: false,
        };
        let result = template.render().unwrap();
        assert!(result.contains("name = \"test-app\""));
        assert!(!result.contains("rust-embed"));
    }

    #[test]
    fn test_cargo_toml_with_embed() {
        let template = CargoTomlTemplate {
            name: "test-app",
            embed_assets: true,
        };
        let result = template.render().unwrap();
        assert!(result.contains("rust-embed"));
    }

    #[test]
    fn test_main_url_template() {
        let template = MainUrlTemplate {
            title: "My App",
            url: "https://example.com",
            width: 1024,
            height: 768,
        };
        let result = template.render().unwrap();
        assert!(result.contains("const APP_URL: &str = \"https://example.com\""));
        assert!(result.contains("const WINDOW_TITLE: &str = \"My App\""));
    }

    #[test]
    fn test_main_frontend_template() {
        let template = MainFrontendTemplate {
            title: "Frontend App",
            width: 800,
            height: 600,
        };
        let result = template.render().unwrap();
        assert!(result.contains("rust_embed::RustEmbed"));
        assert!(result.contains("struct Assets"));
    }
}
