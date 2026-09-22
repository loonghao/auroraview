//! Pluggable render-backend contract.
//!
//! A *render backend* owns the actual web surface. Two families must be
//! reachable through the same contract:
//!
//! * [`BackendFamily::Native`] -- the platform webview (WebView2 on Windows,
//!   WKWebView on macOS, WebKitGTK on Linux). Small, no bundled runtime.
//! * [`BackendFamily::Chromium`] -- a bundled Chromium-class engine. Heavy, but
//!   the only path that gives a real CDP endpoint, DevTools protocol parity and
//!   identical rendering across platforms.
//!
//! # Relationship to the existing `auroraview-core::backend`
//!
//! `auroraview-core::backend` already defines `WebViewBackend`, but its
//! `BackendFactory` is a closed `match` over a `BackendType` enum: adding a
//! backend means editing core, and `BackendType::WebView2` on Windows silently
//! returns the same `WryBackend` as `BackendType::Wry`. This module keeps the
//! trait design and replaces the closed factory with a registry, so a backend
//! becomes a registration instead of a core edit.
//!
//! # Probing never fails
//!
//! [`RenderBackend::probe`] returns [`CapabilitySupport`] -- see
//! [`crate::capability`]. A backend that is not linked into the build reports
//! `Unsupported` with a build flag to flip, rather than panicking or returning
//! an opaque internal error.

use std::fmt;
use std::sync::Arc;

use crate::capability::{CapabilityProbe, CapabilityReport, CapabilitySupport, Features};
use crate::registry::{Registry, Selection};

/// Environment variable used to pin a render backend.
pub const ENV_BACKEND: &str = "AURORAVIEW_BACKEND";

/// Engine family behind a backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendFamily {
    /// The platform's own webview control.
    Native,
    /// A bundled Chromium-class engine.
    Chromium,
    /// Anything else (embedded browser supplied by an adapter).
    Other,
}

impl BackendFamily {
    /// Stable snake-case name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Chromium => "chromium",
            Self::Other => "other",
        }
    }
}

/// Description of a surface to create.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceSpec {
    /// Window title.
    pub title: String,
    /// Initial width in pixels.
    pub width: u32,
    /// Initial height in pixels.
    pub height: u32,
    /// URL to load, if any.
    pub url: Option<String>,
    /// Inline HTML to load, if any.
    pub html: Option<String>,
    /// Native parent window handle, when embedding into a host window.
    pub parent_handle: Option<u64>,
    /// Request a transparent (per-pixel alpha) surface.
    pub transparent: bool,
    /// Enable the inspector / DevTools.
    pub devtools: bool,
}

impl Default for SurfaceSpec {
    fn default() -> Self {
        Self {
            title: "AuroraView".to_string(),
            width: 800,
            height: 600,
            url: None,
            html: None,
            parent_handle: None,
            transparent: false,
            devtools: true,
        }
    }
}

impl SurfaceSpec {
    /// Start a spec from a URL.
    pub fn with_url(url: impl Into<String>) -> Self {
        Self {
            url: Some(url.into()),
            ..Self::default()
        }
    }

    /// Start a spec from inline HTML.
    pub fn with_html(html: impl Into<String>) -> Self {
        Self {
            html: Some(html.into()),
            ..Self::default()
        }
    }

    /// Attach the surface to a host window.
    pub fn with_parent_handle(mut self, handle: u64) -> Self {
        self.parent_handle = Some(handle);
        self
    }
}

/// A backend operation that could not be performed.
#[derive(Debug, Clone)]
pub struct BackendError {
    /// Operation that failed.
    pub operation: &'static str,
    /// Why it failed.
    pub reason: String,
    /// What the integrator should do about it.
    pub how_to_enable: Option<String>,
}

impl BackendError {
    /// Build an "unsupported by this backend" error.
    pub fn unsupported(
        operation: &'static str,
        reason: impl Into<String>,
        how_to_enable: impl Into<String>,
    ) -> Self {
        Self {
            operation,
            reason: reason.into(),
            how_to_enable: Some(how_to_enable.into()),
        }
    }

    /// Build an error with no remediation hint.
    pub fn failed(operation: &'static str, reason: impl Into<String>) -> Self {
        Self {
            operation,
            reason: reason.into(),
            how_to_enable: None,
        }
    }
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.how_to_enable {
            Some(hint) => write!(
                f,
                "{} failed: {} (to enable: {})",
                self.operation, self.reason, hint
            ),
            None => write!(f, "{} failed: {}", self.operation, self.reason),
        }
    }
}

impl std::error::Error for BackendError {}

/// Result alias for backend operations.
pub type BackendResult<T> = Result<T, BackendError>;

/// A live web surface created by a [`RenderBackend`].
pub trait RenderSurface: Send + Sync {
    /// Id of the backend that created this surface.
    fn backend_id(&self) -> &'static str;

    /// Navigate to a URL.
    fn navigate(&self, url: &str) -> BackendResult<()>;

    /// Load inline HTML.
    fn load_html(&self, html: &str) -> BackendResult<()>;

    /// Evaluate a script, discarding the result.
    fn eval_js(&self, script: &str) -> BackendResult<()>;

    /// Move and resize the surface.
    fn set_bounds(&self, x: i32, y: i32, width: u32, height: u32) -> BackendResult<()>;

    /// Show or hide the surface.
    fn set_visible(&self, visible: bool) -> BackendResult<()>;

    /// Native window handle, once the surface has been realised.
    fn native_handle(&self) -> Option<u64>;

    /// Pump one round of events.
    ///
    /// Returns `Ok(true)` while the surface is alive, `Ok(false)` when it has
    /// been closed.
    fn process_events(&self) -> BackendResult<bool>;

    /// Tear the surface down.
    fn close(&self) -> BackendResult<()>;
}

/// The render-backend contract.
pub trait RenderBackend: Send + Sync {
    /// Stable machine-readable id (`"native"`, `"chromium"`).
    fn id(&self) -> &'static str;

    /// Human readable name.
    fn display_name(&self) -> &'static str;

    /// Engine family.
    fn family(&self) -> BackendFamily;

    /// Whether this backend can actually create surfaces in this build.
    ///
    /// Cheap and non-panicking: it reports linkage/runtime presence, it does not
    /// create anything.
    fn available(&self) -> bool;

    /// What is missing when [`RenderBackend::available`] is `false`.
    fn missing_requirement(&self) -> Option<String>;

    /// Capabilities this backend declares when available.
    fn capabilities(&self) -> Features;

    /// Probe one capability.
    fn probe(&self, feature: Features) -> CapabilitySupport;

    /// Create a surface.
    ///
    /// A backend that is registered but not linked must return
    /// [`BackendError::unsupported`] with a build hint, never panic.
    fn create_surface(&self, spec: &SurfaceSpec) -> BackendResult<Box<dyn RenderSurface>>;
}

impl CapabilityProbe for dyn RenderBackend {
    fn subject(&self) -> String {
        self.id().to_string()
    }

    fn capabilities(&self) -> Features {
        RenderBackend::capabilities(self)
    }

    fn probe(&self, feature: Features) -> CapabilitySupport {
        RenderBackend::probe(self, feature)
    }
}

/// Registry of render backends.
pub struct BackendRegistry {
    registry: Registry<Arc<dyn RenderBackend>>,
}

impl Default for BackendRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for BackendRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BackendRegistry")
            .field("backends", &self.registry.names())
            .finish()
    }
}

impl BackendRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            registry: Registry::new(),
        }
    }

    /// Register a backend built by `factory`.
    pub fn register(
        &mut self,
        name: impl Into<String>,
        priority: i32,
        factory: impl Fn() -> Arc<dyn RenderBackend> + Send + Sync + 'static,
    ) {
        self.registry.register(name, priority, factory);
    }

    /// Register a [`Default`]-constructible backend.
    pub fn register_default<B>(&mut self, priority: i32)
    where
        B: RenderBackend + Default + 'static,
    {
        let name = B::default().id().to_string();
        self.registry.register(name, priority, || {
            Arc::new(B::default()) as Arc<dyn RenderBackend>
        });
    }

    /// Registered backend ids in priority order.
    pub fn ids(&self) -> Vec<&str> {
        self.registry.names()
    }

    /// Number of registered backends.
    pub fn len(&self) -> usize {
        self.registry.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.registry.is_empty()
    }

    /// `(priority, id, available)` for every backend.
    pub fn list(&self) -> Vec<(i32, String, bool)> {
        self.registry
            .entries()
            .iter()
            .map(|entry| {
                let backend = entry.create();
                (
                    entry.priority(),
                    entry.name().to_string(),
                    backend.available(),
                )
            })
            .collect()
    }

    /// Select the best available backend, honouring an explicit override.
    pub fn select_with(
        &self,
        env_override: Option<&str>,
    ) -> Option<Selection<Arc<dyn RenderBackend>>> {
        self.registry
            .select(env_override, |backend| backend.available())
    }

    /// Select the best available backend, honouring [`ENV_BACKEND`].
    pub fn select(&self) -> Option<Selection<Arc<dyn RenderBackend>>> {
        self.select_with(std::env::var(ENV_BACKEND).ok().as_deref())
    }

    /// Capability report for one backend, by id.
    pub fn report(&self, id: &str) -> Option<CapabilityReport> {
        let backend = self.registry.get(id)?;
        let mut report = CapabilityReport::new(backend.id());
        for flag in Features::iter_flags() {
            report.push(flag, backend.probe(flag));
        }
        Some(report)
    }
}

/// The platform webview backend: WebView2 / WKWebView / WebKitGTK.
///
/// Declared here so both families are visible in one place. Surface creation is
/// delegated to the crate that links the engine (today: `auroraview`/wry); this
/// descriptor states the capabilities of that path and is what capability
/// probing reports.
#[derive(Debug, Default, Clone, Copy)]
pub struct NativeWebviewBackend;

impl RenderBackend for NativeWebviewBackend {
    fn id(&self) -> &'static str {
        "native"
    }

    fn display_name(&self) -> &'static str {
        "Platform WebView (WebView2 / WKWebView / WebKitGTK)"
    }

    fn family(&self) -> BackendFamily {
        BackendFamily::Native
    }

    fn available(&self) -> bool {
        true
    }

    fn missing_requirement(&self) -> Option<String> {
        None
    }

    fn capabilities(&self) -> Features {
        Features::NATIVE_EMBEDDING
            | Features::OUT_OF_PROCESS
            | Features::DEVTOOLS
            | Features::JS_EVAL_RESULT
            | Features::COOKIES
            | Features::FILE_PROTOCOL
            | Features::MAIN_THREAD_DISPATCH
            | Features::MULTI_WINDOW
    }

    fn probe(&self, feature: Features) -> CapabilitySupport {
        match feature {
            Features::CDP => CapabilitySupport::unsupported(
                "the platform webview does not expose a Chrome DevTools Protocol endpoint",
                "select the 'chromium' backend (AURORAVIEW_BACKEND=chromium)",
            ),
            Features::TRANSPARENCY => CapabilitySupport::unknown(
                "transparency depends on the platform compositor; verify at runtime",
            ),
            other if self.capabilities().contains(other) => CapabilitySupport::Supported,
            other => CapabilitySupport::unsupported(
                format!("the native backend does not provide {}", other),
                "select a different backend via AURORAVIEW_BACKEND",
            ),
        }
    }

    fn create_surface(&self, _spec: &SurfaceSpec) -> BackendResult<Box<dyn RenderSurface>> {
        Err(BackendError::unsupported(
            "create_surface",
            "this descriptor declares the native path; it is wired by the crate that links wry",
            "call BackendRegistry::select() from a build that links the native backend",
        ))
    }
}

/// A bundled Chromium-class backend.
///
/// Present so the second path is a first-class citizen of the contract rather
/// than a TODO. It is **not linked** in any current build, which is exactly the
/// situation the capability contract exists to describe: `available()` is
/// `false`, and every probe returns a structured `Unsupported` carrying the
/// build flag that would enable it.
#[derive(Debug, Default, Clone, Copy)]
pub struct ChromiumBackend;

impl RenderBackend for ChromiumBackend {
    fn id(&self) -> &'static str {
        "chromium"
    }

    fn display_name(&self) -> &'static str {
        "Chromium (bundled engine, CDP-capable)"
    }

    fn family(&self) -> BackendFamily {
        BackendFamily::Chromium
    }

    fn available(&self) -> bool {
        false
    }

    fn missing_requirement(&self) -> Option<String> {
        Some(
            "the Chromium backend is not linked into this build; \
             enable the `chromium` feature"
                .to_string(),
        )
    }

    fn capabilities(&self) -> Features {
        Features::CDP
            | Features::DEVTOOLS
            | Features::JS_EVAL_RESULT
            | Features::COOKIES
            | Features::MULTI_WINDOW
            | Features::OUT_OF_PROCESS
    }

    fn probe(&self, feature: Features) -> CapabilitySupport {
        // An unlinked backend cannot answer anything -- not even "unknown".
        // Short-circuiting here is what keeps probing honest: every answer from
        // a backend that is not in the build is a structured "unsupported, and
        // here is the flag that would enable it".
        if !self.available() {
            return CapabilitySupport::unsupported(
                format!(
                    "the chromium backend is not linked into this build, so {} \
                     cannot be provided",
                    feature
                ),
                self.missing_requirement()
                    .unwrap_or_else(|| "enable the `chromium` feature".to_string()),
            );
        }

        match feature {
            Features::NATIVE_EMBEDDING => CapabilitySupport::unsupported(
                "Chromium surfaces are not parented to a host window in this build",
                "run out-of-process and pass --parent-hwnd",
            ),
            Features::FILE_PROTOCOL => CapabilitySupport::unknown(
                "depends on how the Chromium build registers custom schemes; verify at runtime",
            ),
            other if self.capabilities().contains(other) => CapabilitySupport::Supported,
            other => CapabilitySupport::unsupported(
                format!("the chromium backend does not provide {}", other),
                "select a different backend via AURORAVIEW_BACKEND",
            ),
        }
    }

    fn create_surface(&self, _spec: &SurfaceSpec) -> BackendResult<Box<dyn RenderSurface>> {
        Err(BackendError::unsupported(
            "create_surface",
            "the chromium backend is not linked into this build",
            self.missing_requirement()
                .unwrap_or_else(|| "enable the `chromium` feature".to_string()),
        ))
    }
}

/// A registry preloaded with both reference backends: native first, chromium second.
pub fn default_backend_registry() -> BackendRegistry {
    let mut registry = BackendRegistry::new();
    registry.register_default::<NativeWebviewBackend>(100);
    registry.register_default::<ChromiumBackend>(50);
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_backend_declares_the_system_webview_path() {
        let backend = NativeWebviewBackend;
        assert_eq!(backend.id(), "native");
        assert_eq!(backend.family(), BackendFamily::Native);
        assert!(backend.available());
        assert_eq!(backend.missing_requirement(), None);
    }

    #[test]
    fn native_backend_reports_cdp_as_unsupported_with_a_remediation() {
        let support = NativeWebviewBackend.probe(Features::CDP);
        assert!(!support.is_supported());
        assert_eq!(
            support.how_to_enable(),
            Some("select the 'chromium' backend (AURORAVIEW_BACKEND=chromium)")
        );
        assert!(support.reason().expect("reason").contains("DevTools"));
    }

    #[test]
    fn native_backend_reports_transparency_as_unknown_not_unsupported() {
        let support = NativeWebviewBackend.probe(Features::TRANSPARENCY);
        assert!(support.is_unknown(), "must not guess: {}", support);
        assert_eq!(support.how_to_enable(), None);
    }

    #[test]
    fn native_backend_reports_declared_capabilities_as_supported() {
        assert!(NativeWebviewBackend
            .probe(Features::NATIVE_EMBEDDING)
            .is_supported());
        assert!(NativeWebviewBackend.probe(Features::COOKIES).is_supported());
    }

    #[test]
    fn chromium_backend_is_not_available_and_says_how_to_enable_it() {
        let backend = ChromiumBackend;
        assert_eq!(backend.family(), BackendFamily::Chromium);
        assert!(!backend.available());

        let requirement = backend.missing_requirement().expect("requirement");
        assert!(requirement.contains("chromium"), "{}", requirement);
    }

    #[test]
    fn chromium_backend_probing_never_guesses_and_never_panics() {
        let backend = ChromiumBackend;
        for flag in Features::iter_flags() {
            let support = backend.probe(flag);
            assert!(
                !support.is_supported(),
                "an unlinked backend must not claim {} as supported",
                flag
            );
            assert!(
                support.how_to_enable().is_some(),
                "every unsupported answer must carry a remediation hint ({})",
                flag
            );
        }
    }

    #[test]
    fn chromium_backend_declares_cdp_but_it_stays_unsupported_until_linked() {
        assert!(ChromiumBackend.capabilities().contains(Features::CDP));
        let support = ChromiumBackend.probe(Features::CDP);
        assert!(!support.is_supported());
        assert!(support.reason().expect("reason").contains("not linked"));
    }

    #[test]
    fn unlinked_backends_fail_surface_creation_with_a_hint_instead_of_panicking() {
        for backend in [
            &NativeWebviewBackend as &dyn RenderBackend,
            &ChromiumBackend,
        ] {
            let err = backend
                .create_surface(&SurfaceSpec::with_url("about:blank"))
                .err()
                .expect("descriptors must not create surfaces");
            assert_eq!(err.operation, "create_surface");
            assert!(err.how_to_enable.is_some());
        }
    }

    #[test]
    fn surface_spec_builders() {
        let spec = SurfaceSpec::with_url("https://example.com").with_parent_handle(42);
        assert_eq!(spec.url.as_deref(), Some("https://example.com"));
        assert_eq!(spec.parent_handle, Some(42));
        assert!(spec.devtools);

        let spec = SurfaceSpec::with_html("<h1>hi</h1>");
        assert_eq!(spec.html.as_deref(), Some("<h1>hi</h1>"));
        assert_eq!(spec.width, 800);
        assert_eq!(spec.height, 600);
    }

    #[test]
    fn default_registry_prefers_native_but_lists_both() {
        let registry = default_backend_registry();
        assert_eq!(registry.ids(), vec!["native", "chromium"]);
        assert_eq!(
            registry.list(),
            vec![
                (100, "native".to_string(), true),
                (50, "chromium".to_string(), false)
            ]
        );

        let selection = registry
            .select_with(None)
            .expect("native must be available");
        assert_eq!(selection.name, "native");
        assert!(!selection.has_warnings());
    }

    #[test]
    fn backend_override_falls_back_with_a_warning_when_unavailable() {
        let registry = default_backend_registry();
        let selection = registry
            .select_with(Some("chromium"))
            .expect("must fall back to native");
        assert_eq!(selection.name, "native");
        assert!(selection.has_warnings());
        assert!(selection.warnings[0].contains("not available"));
    }

    #[test]
    fn registry_reports_capabilities_per_backend() {
        let registry = default_backend_registry();
        let report = registry.report("native").expect("native report");
        assert!(report.supported().contains(&"NATIVE_EMBEDDING"));
        assert!(report
            .unknown()
            .iter()
            .any(|entry| entry.feature == "TRANSPARENCY"));
        assert!(report
            .unsupported()
            .iter()
            .any(|entry| entry.feature == "CDP"));
    }

    #[test]
    fn unknown_backend_report_is_none() {
        assert!(default_backend_registry().report("nope").is_none());
    }
}
