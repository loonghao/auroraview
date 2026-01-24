//! Event loop management for desktop mode

mod handler;
mod user_event;

pub use handler::{run, run_with_router};
pub use user_event::UserEvent;
