//! AuroraView contracts: host adapters and pluggable render backends.
//!
//! This crate is the **dependency-direction firewall** for the AuroraView
//! package split. It contains traits, value types and registries -- and no
//! implementation, no host SDK, and no WebView engine.
//!
//! # The invariant
//!
//! ```text
//!   auroraview-contract        (this crate: traits only, zero dependencies)
//!        ^               ^
//!        |               |
//!   auroraview-core    auroraview-maya / -unreal / -unity / ...
//!        ^               ^
//!        |               |
//!        +---------------+
//!          adapters depend on core and on the contract;
//!          core depends on the contract and on NO adapter
//! ```
//!
//! Because the contract is a leaf with zero dependencies, an adapter crate can
//! depend on it without pulling in a WebView engine, and core can accept
//! adapters without naming them. A new host becomes a new crate that registers
//! itself, instead of an edit to a `DccType` enum in core.
//!
//! # What lives here
//!
//! | Module | Contract |
//! |---|---|
//! | [`host`] | how a host registers itself, is discovered, and embeds a surface |
//! | [`backend`] | how a render backend is selected and probed |
//! | [`capability`] | how unsupported features are reported |
//! | [`registry`] | the shared priority-registry mechanics |
//!
//! # Capability probing never fails
//!
//! Missing features are data, not exceptions. Probing returns
//! [`capability::CapabilitySupport`], which is `Supported`, `Unsupported { reason,
//! how_to_enable }`, or `Unknown { reason }`. The single fallible conversion is
//! `CapabilitySupport::require(..)`, used only by callers that cannot degrade
//! gracefully.
//!
//! # One extension idiom, two languages
//!
//! The registries here mirror
//! `auroraview.utils.thread_dispatcher.registry` on the Python side: priority
//! order, lazy construction, environment override
//! (`AURORAVIEW_HOST` / `AURORAVIEW_BACKEND`), and a failed override that
//! degrades to priority order with a warning instead of failing.

#![deny(missing_docs)]

pub mod backend;
pub mod capability;
pub mod host;
pub mod registry;

pub use backend::{
    default_backend_registry, BackendError, BackendFamily, BackendRegistry, BackendResult,
    ChromiumBackend, NativeWebviewBackend, RenderBackend, RenderSurface, SurfaceSpec, ENV_BACKEND,
};
pub use capability::{
    CapabilityEntry, CapabilityError, CapabilityProbe, CapabilityReport, CapabilitySupport,
    Features,
};
pub use host::{
    EmbedMode, HostAdapter, HostError, HostInfo, HostJob, HostRegistry, HostResult, ThreadModel,
    UiFramework, ENV_HOST,
};
pub use registry::{Entry, Registry, Selection};
