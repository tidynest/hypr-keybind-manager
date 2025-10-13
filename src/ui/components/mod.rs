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

pub mod keybind_list;
pub mod search_bar;

pub use keybind_list::KeybindList;
pub use search_bar::SearchBar;