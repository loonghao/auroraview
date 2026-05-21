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
pub use run::{
    resolve_capture_file_drop as resolve_run_capture_file_drop, run_webview, RunArgs,
};
pub use self_update::{run_self_update, SelfUpdateArgs};
pub use skills::{run_skills, SkillsArgs};
