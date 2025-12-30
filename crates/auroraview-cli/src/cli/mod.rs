//! CLI command modules
//!
//! This module organizes CLI commands into separate files for better maintainability.

mod icon;
mod info;
mod pack;
mod run;
mod self_update;

pub use icon::{run_icon, IconArgs};
pub use info::run_info;
pub use pack::{run_pack, PackArgs};
pub use run::{run_webview, RunArgs};
pub use self_update::{run_self_update, SelfUpdateArgs};
