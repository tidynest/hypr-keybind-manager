//! UI Components
//!
//! Reusable GTK4 widgets for the keybinding manager.
//!
//! # Components (Day 2+)
//!
//! - `keybind_list.rs` - Scrollable list of keybindings
//! - `search_bar.rs` - Real-time search/filter
//! - `conflict_panel.rs` - Conflict warning banner
//! - `details_panel.rs` - Selected binding details

// Components will be added in Day 2

mod conflict_panel;
mod edit_dialog;
mod keybind_list;
mod search_bar;
mod details_panel;
mod backup_dialog;

pub use conflict_panel::ConflictPanel;
pub use edit_dialog::EditDialog;
pub use keybind_list::KeybindList;
pub use search_bar::SearchBar;
pub use details_panel::DetailsPanel;
pub use backup_dialog::BackupDialog;