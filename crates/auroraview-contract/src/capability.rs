//! Capability probing.
//!
//! # The rule this module exists to enforce
//!
//! Probing a capability **never fails**. A missing feature is data, not an
//! exception:
//!
//! ```text
//! probe(feature) -> CapabilitySupport   // always returns
//! support.require(..) -> Result<..>     // the single fallible conversion
//! ```
//!
//! Every "unsupported" answer carries a `reason` and a `how_to_enable`
//! string so a host integrator can act on it without reading AuroraView
//! source. `Unknown` is a first-class answer: it is how a backend says
//! "this depends on the runtime, verify at runtime" without guessing.

use std::fmt;

/// A set of optional capabilities declared by an adapter or a backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Features(u32);

impl Features {
    /// The surface can be parented into a host-provided native window.
    pub const NATIVE_EMBEDDING: Self = Self(1 << 0);
    /// The surface can run detached in its own process.
    pub const OUT_OF_PROCESS: Self = Self(1 << 1);
    /// DevTools / inspector is reachable.
    pub const DEVTOOLS: Self = Self(1 << 2);
    /// Chrome DevTools Protocol endpoint is exposed.
    pub const CDP: Self = Self(1 << 3);
    /// Transparent (per-pixel alpha) surfaces.
    pub const TRANSPARENCY: Self = Self(1 << 4);
    /// `eval_js` can return the script result to the caller.
    pub const JS_EVAL_RESULT: Self = Self(1 << 5);
    /// Cookie read / write / clear.
    pub const COOKIES: Self = Self(1 << 6);
    /// Custom `file://` (or equivalent) protocol handler.
    pub const FILE_PROTOCOL: Self = Self(1 << 7);
    /// The host can marshal work onto its own UI thread.
    pub const MAIN_THREAD_DISPATCH: Self = Self(1 << 8);
    /// More than one surface per process.
    pub const MULTI_WINDOW: Self = Self(1 << 9);

    /// Every capability declared in this crate.
    pub const ALL: Self = Self(
        Self::NATIVE_EMBEDDING.0
            | Self::OUT_OF_PROCESS.0
            | Self::DEVTOOLS.0
            | Self::CDP.0
            | Self::TRANSPARENCY.0
            | Self::JS_EVAL_RESULT.0
            | Self::COOKIES.0
            | Self::FILE_PROTOCOL.0
            | Self::MAIN_THREAD_DISPATCH.0
            | Self::MULTI_WINDOW.0,
    );

    /// An empty set.
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Raw bit mask.
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Whether no capability is set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Whether `other` is fully contained in `self`.
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Whether the two sets share at least one capability.
    pub const fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    /// Set union.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Set intersection.
    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    /// Set difference.
    pub const fn difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    /// One entry per declared capability, in declaration order.
    pub fn iter_flags() -> Vec<Self> {
        vec![
            Self::NATIVE_EMBEDDING,
            Self::OUT_OF_PROCESS,
            Self::DEVTOOLS,
            Self::CDP,
            Self::TRANSPARENCY,
            Self::JS_EVAL_RESULT,
            Self::COOKIES,
            Self::FILE_PROTOCOL,
            Self::MAIN_THREAD_DISPATCH,
            Self::MULTI_WINDOW,
        ]
    }

    /// Human readable names of every capability set on `self`.
    pub fn names(self) -> Vec<&'static str> {
        Self::iter_flags()
            .into_iter()
            .filter(|flag| self.contains(*flag))
            .map(Self::flag_name)
            .collect()
    }

    /// Human readable name of a single-bit capability set.
    pub fn flag_name(self) -> &'static str {
        match self {
            Self::NATIVE_EMBEDDING => "NATIVE_EMBEDDING",
            Self::OUT_OF_PROCESS => "OUT_OF_PROCESS",
            Self::DEVTOOLS => "DEVTOOLS",
            Self::CDP => "CDP",
            Self::TRANSPARENCY => "TRANSPARENCY",
            Self::JS_EVAL_RESULT => "JS_EVAL_RESULT",
            Self::COOKIES => "COOKIES",
            Self::FILE_PROTOCOL => "FILE_PROTOCOL",
            Self::MAIN_THREAD_DISPATCH => "MAIN_THREAD_DISPATCH",
            Self::MULTI_WINDOW => "MULTI_WINDOW",
            _ => "MULTI_FLAG",
        }
    }
}

impl fmt::Display for Features {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names = self.names();
        if names.is_empty() {
            f.write_str("<none>")
        } else {
            f.write_str(&names.join("|"))
        }
    }
}

impl std::ops::BitOr for Features {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

impl std::ops::BitOrAssign for Features {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.union(rhs);
    }
}

impl std::ops::BitAnd for Features {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        self.intersection(rhs)
    }
}

impl std::ops::Sub for Features {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self.difference(rhs)
    }
}

/// The answer to "do you support this capability?".
///
/// Never an error: see the module docs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilitySupport {
    /// Supported.
    Supported,
    /// Known to be unsupported, with a remediation hint.
    Unsupported {
        /// Why it is unsupported.
        reason: String,
        /// What the integrator should do to enable it.
        how_to_enable: String,
    },
    /// Cannot be decided at declaration time; verify at runtime.
    Unknown {
        /// Why it cannot be decided here.
        reason: String,
    },
}

impl CapabilitySupport {
    /// Create a `Supported` answer.
    pub fn supported() -> Self {
        Self::Supported
    }

    /// Create an `Unsupported` answer with a remediation hint.
    pub fn unsupported(reason: impl Into<String>, how_to_enable: impl Into<String>) -> Self {
        Self::Unsupported {
            reason: reason.into(),
            how_to_enable: how_to_enable.into(),
        }
    }

    /// Create an `Unknown` answer.
    pub fn unknown(reason: impl Into<String>) -> Self {
        Self::Unknown {
            reason: reason.into(),
        }
    }

    /// Whether this is [`CapabilitySupport::Supported`].
    pub fn is_supported(&self) -> bool {
        matches!(self, Self::Supported)
    }

    /// Whether this is [`CapabilitySupport::Unknown`].
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown { .. })
    }

    /// Why the capability is missing, if it is.
    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Supported => None,
            Self::Unsupported { reason, .. } | Self::Unknown { reason } => Some(reason.as_str()),
        }
    }

    /// Remediation hint, if one was supplied.
    pub fn how_to_enable(&self) -> Option<&str> {
        match self {
            Self::Unsupported { how_to_enable, .. } => Some(how_to_enable.as_str()),
            _ => None,
        }
    }

    /// The one and only fallible conversion: turn an answer into a `Result`.
    ///
    /// Callers that cannot proceed without the capability use this. Callers
    /// that can degrade gracefully keep the [`CapabilitySupport`] value.
    pub fn require(self, subject: &str, feature: &str) -> Result<(), CapabilityError> {
        match self {
            Self::Supported => Ok(()),
            other => Err(CapabilityError {
                subject: subject.to_string(),
                feature: feature.to_string(),
                support: other,
            }),
        }
    }
}

impl fmt::Display for CapabilitySupport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Supported => write!(f, "supported"),
            Self::Unsupported {
                reason,
                how_to_enable,
            } => write!(f, "unsupported: {} (to enable: {})", reason, how_to_enable),
            Self::Unknown { reason } => write!(f, "unknown: {}", reason),
        }
    }
}

/// Error produced by [`CapabilitySupport::require`].
#[derive(Debug, Clone)]
pub struct CapabilityError {
    /// What was probed (a host id or a backend id).
    pub subject: String,
    /// The capability that was required.
    pub feature: String,
    /// The answer that failed the requirement.
    pub support: CapabilitySupport,
}

impl fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "'{}' cannot provide {}: {}",
            self.subject, self.feature, self.support
        )
    }
}

impl std::error::Error for CapabilityError {}

/// One row of a [`CapabilityReport`].
#[derive(Debug, Clone)]
pub struct CapabilityEntry {
    /// Capability name.
    pub feature: &'static str,
    /// The capability that was probed.
    pub flag: Features,
    /// The answer.
    pub support: CapabilitySupport,
}

/// A complete, human readable capability snapshot.
#[derive(Debug, Clone)]
pub struct CapabilityReport {
    /// Host id or backend id this report describes.
    pub subject: String,
    /// One row per capability declared by this crate.
    pub entries: Vec<CapabilityEntry>,
}

impl CapabilityReport {
    /// Create an empty report for `subject`.
    pub fn new(subject: impl Into<String>) -> Self {
        Self {
            subject: subject.into(),
            entries: Vec::new(),
        }
    }

    /// Append a row.
    pub fn push(&mut self, flag: Features, support: CapabilitySupport) {
        self.entries.push(CapabilityEntry {
            feature: flag.flag_name(),
            flag,
            support,
        });
    }

    /// Look up a row by capability name.
    pub fn get(&self, feature: &str) -> Option<&CapabilitySupport> {
        self.entries
            .iter()
            .find(|entry| entry.feature == feature)
            .map(|entry| &entry.support)
    }

    /// Names of every supported capability.
    pub fn supported(&self) -> Vec<&'static str> {
        self.entries
            .iter()
            .filter(|entry| entry.support.is_supported())
            .map(|entry| entry.feature)
            .collect()
    }

    /// Every unsupported row.
    pub fn unsupported(&self) -> Vec<&CapabilityEntry> {
        self.entries
            .iter()
            .filter(|entry| matches!(entry.support, CapabilitySupport::Unsupported { .. }))
            .collect()
    }

    /// Every unknown row.
    pub fn unknown(&self) -> Vec<&CapabilityEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.support.is_unknown())
            .collect()
    }

    /// Render as a multi-line report.
    pub fn render(&self) -> String {
        let mut out = format!("capability report: {}\n", self.subject);
        out.push_str(&format!(
            "  supported   : {}\n",
            self.supported().join(", ")
        ));

        let unsupported = self.unsupported();
        if unsupported.is_empty() {
            out.push_str("  unsupported : none\n");
        } else {
            out.push_str("  unsupported :\n");
            for entry in unsupported {
                out.push_str(&format!("    - {}: {}\n", entry.feature, entry.support));
            }
        }

        let unknown = self.unknown();
        if !unknown.is_empty() {
            out.push_str("  unknown     :\n");
            for entry in unknown {
                out.push_str(&format!("    - {}: {}\n", entry.feature, entry.support));
            }
        }

        out
    }
}

/// Anything whose capabilities can be probed.
pub trait CapabilityProbe {
    /// Name used as the report subject.
    fn subject(&self) -> String;

    /// Capabilities this implementation declares.
    fn capabilities(&self) -> Features;

    /// Probe a single capability. Must not panic; return [`CapabilitySupport::Unknown`]
    /// when the answer depends on runtime state.
    fn probe(&self, feature: Features) -> CapabilitySupport;

    /// Build a full report across every capability declared by this crate.
    fn report(&self) -> CapabilityReport {
        let mut report = CapabilityReport::new(self.subject());
        for flag in Features::iter_flags() {
            report.push(flag, self.probe(flag));
        }
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn features_bit_operations() {
        let set = Features::NATIVE_EMBEDDING | Features::DEVTOOLS;
        assert!(set.contains(Features::NATIVE_EMBEDDING));
        assert!(set.contains(Features::DEVTOOLS));
        assert!(!set.contains(Features::CDP));
        assert!(set.intersects(Features::CDP | Features::DEVTOOLS));

        let reduced = set - Features::DEVTOOLS;
        assert_eq!(reduced, Features::NATIVE_EMBEDDING);
        assert_eq!(set & Features::DEVTOOLS, Features::DEVTOOLS);
        assert!(Features::empty().is_empty());
    }

    #[test]
    fn features_all_contains_every_flag() {
        for flag in Features::iter_flags() {
            assert!(
                Features::ALL.contains(flag),
                "Features::ALL is missing {}",
                flag
            );
        }
    }

    #[test]
    fn features_names_and_display() {
        assert_eq!(Features::CDP.names(), vec!["CDP"]);
        assert_eq!(Features::empty().names(), Vec::<&str>::new());
        assert_eq!(
            (Features::CDP | Features::DEVTOOLS).to_string(),
            "DEVTOOLS|CDP"
        );
        assert_eq!(Features::empty().to_string(), "<none>");
    }

    #[test]
    fn unsupported_answer_carries_remediation() {
        let support =
            CapabilitySupport::unsupported("not linked", "build with --features chromium");
        assert!(!support.is_supported());
        assert_eq!(support.reason(), Some("not linked"));
        assert_eq!(
            support.how_to_enable(),
            Some("build with --features chromium")
        );
        assert!(support.to_string().contains("to enable: build with"));
    }

    #[test]
    fn unknown_answer_has_no_remediation() {
        let support = CapabilitySupport::unknown("depends on the runtime window manager");
        assert!(support.is_unknown());
        assert!(!support.is_supported());
        assert_eq!(support.how_to_enable(), None);
    }

    #[test]
    fn require_is_the_only_fallible_conversion() {
        assert!(CapabilitySupport::supported()
            .require("native", "CDP")
            .is_ok());

        let err = CapabilitySupport::unsupported("not linked", "rebuild")
            .require("native", "CDP")
            .expect_err("unsupported must fail require()");
        assert_eq!(err.subject, "native");
        assert_eq!(err.feature, "CDP");
        assert!(err.to_string().contains("cannot provide CDP"));
    }

    struct Probe {
        capabilities: Features,
    }

    impl CapabilityProbe for Probe {
        fn subject(&self) -> String {
            "probe".to_string()
        }

        fn capabilities(&self) -> Features {
            self.capabilities
        }

        fn probe(&self, feature: Features) -> CapabilitySupport {
            if self.capabilities.contains(feature) {
                CapabilitySupport::Supported
            } else {
                CapabilitySupport::Unknown {
                    reason: "not declared".to_string(),
                }
            }
        }
    }

    #[test]
    fn report_groups_answers() {
        let probe = Probe {
            capabilities: Features::NATIVE_EMBEDDING | Features::DEVTOOLS,
        };
        let report = probe.report();
        assert_eq!(report.subject, "probe");
        assert_eq!(report.supported(), vec!["NATIVE_EMBEDDING", "DEVTOOLS"]);
        assert!(report.unsupported().is_empty());
        assert_eq!(report.unknown().len(), 8);
        assert!(report.get("CDP").expect("CDP row").is_unknown());

        let rendered = report.render();
        assert!(rendered.contains("capability report: probe"));
        assert!(rendered.contains("- CDP: unknown: not declared"));
    }
}
