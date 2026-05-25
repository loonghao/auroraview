//! CLI command modules
//!
//! This module organizes CLI commands into separate files for better maintainability.

mod icon;
mod info;
mod inspect;
mod pack;
mod run;
mod self_update;
mod skills;

pub use icon::{run_icon, IconArgs};
pub use info::run_info;
pub use inspect::{run_inspect, InspectArgs};
pub use pack::{resolve_capture_file_drop, run_pack, PackArgs};
pub use run::{resolve_capture_file_drop as resolve_run_capture_file_drop, run_webview, RunArgs};
pub use self_update::{run_self_update, SelfUpdateArgs};
pub use skills::{run_skills, SkillsArgs};

/// Resolve a pair of `--flag` / `--no-flag` clap booleans (both using
/// `SetTrue` + `overrides_with`) into a tri-state `Option<bool>`.
///
/// - both absent → `None` (defer to lower layer / code default)
/// - `--flag` only → `Some(true)`
/// - `--no-flag` only → `Some(false)`
///
/// The `(true, true)` arm should be unreachable when clap's `overrides_with`
/// is correctly configured. In debug builds we trip a `debug_assert!` to
/// surface the misconfiguration loudly during development; release builds
/// fall back to positive-wins (last-wins semantics) rather than crashing
/// the user shell on a future clap upgrade with changed semantics.
#[cold]
#[inline(never)]
fn resolve_flag_pair_conflict() -> Option<bool> {
    debug_assert!(
        false,
        "clap overrides_with should make (true, true) impossible"
    );
    Some(true)
}

pub fn resolve_flag_pair(positive: bool, negative: bool) -> Option<bool> {
    match (positive, negative) {
        (false, false) => None,
        (true, false) => Some(true),
        (false, true) => Some(false),
        (true, true) => resolve_flag_pair_conflict(),
    }
}
