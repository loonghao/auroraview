//! Parent/owner window attachment for the `run` command.
//!
//! Hosts that embed AuroraView out-of-process (Unity, Unreal, PowerPoint,
//! Qt, ...) pass the native handle of the container window. On Windows that is
//! an `HWND`, which tao can consume either as a **parent** (`WS_CHILD`, clipped
//! to the parent's client area) or as an **owner** (a separate top-level window
//! that stays above and is destroyed with its owner).
//!
//! Embedding a panel inside a DCC viewport wants `parent`; a floating tool
//! window wants `owner`. Both are exposed so a host picks the one that matches
//! its windowing model. They are mutually exclusive — a Win32 window is either
//! a child or an owned top-level, never both — so `--parent-hwnd` wins when
//! both are resolvable.
//!
//! Handles may come from the command line or from `AURORAVIEW_PARENT_HWND`,
//! so a host that already speaks the child-window environment contract does
//! not need to build an argv string.

use auroraview_core::parent_ipc::{parse_hwnd, ENV_PARENT_HWND};

/// Where a parent/owner handle came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleSource {
    /// `--parent-hwnd` / `--owner-hwnd`.
    Flag,
    /// `AURORAVIEW_PARENT_HWND`.
    Env,
}

/// A resolved embedding target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowHandle {
    /// Raw `HWND` value on Windows.
    pub raw: isize,
    /// Where the value came from, for logging.
    pub source: HandleSource,
}

impl WindowHandle {
    /// Resolve a handle from an optional flag value, falling back to
    /// `AURORAVIEW_PARENT_HWND`.
    ///
    /// `0` (a null `HWND`) is treated as "not supplied".
    pub fn resolve(flag: Option<&str>) -> Option<Self> {
        if let Some(raw) = flag.and_then(parse_hwnd).filter(|raw| *raw != 0) {
            return Some(Self {
                raw,
                source: HandleSource::Flag,
            });
        }

        let env_value = std::env::var(ENV_PARENT_HWND).ok()?;
        parse_hwnd(&env_value)
            .filter(|raw| *raw != 0)
            .map(|raw| Self {
                raw,
                source: HandleSource::Env,
            })
    }
}

/// Embedding targets requested for a window, if any.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ParentTargets {
    /// `WS_CHILD` embedding target.
    pub parent: Option<WindowHandle>,
    /// Owned top-level window target.
    pub owner: Option<WindowHandle>,
}

impl ParentTargets {
    /// Resolve both targets from the CLI flags.
    pub fn resolve(parent: Option<&str>, owner: Option<&str>) -> Self {
        Self {
            parent: WindowHandle::resolve(parent),
            owner: WindowHandle::resolve(owner),
        }
    }

    /// `true` when the window is being attached to a foreign window.
    pub fn is_embedded(&self) -> bool {
        self.parent.is_some() || self.owner.is_some()
    }
}

/// Apply the resolved targets to a tao window builder.
///
/// `--parent-hwnd` takes precedence: a child window is already destroyed with
/// its parent, which is what an embedding host wants.
///
/// On non-Windows targets the handles are logged and ignored: there is no
/// portable cross-process window reparenting API. The flags are accepted on
/// every platform so host scripts stay portable.
pub fn apply(
    builder: tao::window::WindowBuilder,
    targets: &ParentTargets,
) -> tao::window::WindowBuilder {
    if !targets.is_embedded() {
        return builder;
    }

    #[cfg(target_os = "windows")]
    {
        use tao::platform::windows::WindowBuilderExtWindows;

        if let Some(handle) = targets.parent {
            tracing::info!(
                "[CLI] Attaching as child of HWND 0x{:X} (source: {:?})",
                handle.raw,
                handle.source
            );
            return builder.with_parent_window(handle.raw);
        }

        if let Some(handle) = targets.owner {
            tracing::info!(
                "[CLI] Attaching as owned window of HWND 0x{:X} (source: {:?})",
                handle.raw,
                handle.source
            );
            return builder.with_owner_window(handle.raw);
        }

        builder
    }

    #[cfg(not(target_os = "windows"))]
    {
        tracing::warn!("[CLI] --parent-hwnd/--owner-hwnd require Windows; running detached");
        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn flag_beats_environment() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var(ENV_PARENT_HWND, "0x999");
        let handle = WindowHandle::resolve(Some("123")).expect("resolved");
        std::env::remove_var(ENV_PARENT_HWND);
        assert_eq!(handle.raw, 123);
        assert_eq!(handle.source, HandleSource::Flag);
    }

    #[test]
    fn environment_is_used_when_flag_absent() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var(ENV_PARENT_HWND, "0x1A2B");
        let handle = WindowHandle::resolve(None).expect("resolved");
        std::env::remove_var(ENV_PARENT_HWND);
        assert_eq!(handle.raw, 0x1A2B);
        assert_eq!(handle.source, HandleSource::Env);
    }

    #[test]
    fn garbage_flag_falls_through_to_environment() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var(ENV_PARENT_HWND, "42");
        let handle = WindowHandle::resolve(Some("not-a-handle")).expect("resolved");
        std::env::remove_var(ENV_PARENT_HWND);
        assert_eq!(handle.raw, 42);
        assert_eq!(handle.source, HandleSource::Env);
    }

    #[test]
    fn null_handles_are_ignored() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var(ENV_PARENT_HWND, "0x0");
        assert!(WindowHandle::resolve(None).is_none());
        std::env::remove_var(ENV_PARENT_HWND);
        assert!(WindowHandle::resolve(Some("0")).is_none());
    }

    #[test]
    fn nothing_resolves_when_both_missing() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var(ENV_PARENT_HWND);
        assert!(WindowHandle::resolve(None).is_none());
        assert!(WindowHandle::resolve(Some("")).is_none());
    }

    #[test]
    fn targets_track_both_roles() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var(ENV_PARENT_HWND);
        let targets = ParentTargets::resolve(Some("7"), None);
        assert!(targets.is_embedded());
        assert_eq!(targets.parent.map(|h| h.raw), Some(7));
        assert_eq!(targets.owner, None);

        let detached = ParentTargets::resolve(None, None);
        assert!(!detached.is_embedded());
    }

    #[test]
    fn environment_feeds_both_roles() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var(ENV_PARENT_HWND, "0x10");
        let targets = ParentTargets::resolve(None, None);
        std::env::remove_var(ENV_PARENT_HWND);
        assert_eq!(targets.parent.map(|h| h.raw), Some(0x10));
        assert_eq!(targets.owner.map(|h| h.source), Some(HandleSource::Env));
    }
}
