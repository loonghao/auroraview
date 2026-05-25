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
/// is correctly configured. In debug builds we `panic!` to surface the
/// misconfiguration loudly during development; in release builds we fall
/// back to positive-wins (last-wins semantics) rather than panicking, so
/// a future clap upgrade with changed semantics cannot crash the CLI in
/// production.
pub fn resolve_flag_pair(positive: bool, negative: bool) -> Option<bool> {
    match (positive, negative) {
        (false, false) => None,
        (true, false) => Some(true),
        (false, true) => Some(false),
        // Should be unreachable with correct `overrides_with` config.
        // Debug builds panic loudly to surface the misconfiguration;
        // release builds fall back to positive-wins (last-wins
        // semantics) so a misconfigured CLI never crashes a user shell.
        (true, true) => {
            #[cfg(debug_assertions)]
            {
                panic!("clap overrides_with should make (true, true) impossible");
            }
            #[cfg(not(debug_assertions))]
            {
                Some(true)
            }
        }
    }
}
