//! Navigation module - bookmarks and history

mod bookmarks;
mod history;

pub use bookmarks::{Bookmark, BookmarkFolder, BookmarkId, BookmarkManager};
pub use history::{HistoryEntry, HistoryManager};
