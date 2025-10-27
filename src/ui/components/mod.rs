// Copyright 2025 Eric Jingryd (tidynest@proton.me)
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! UI Components
//!
//! Reusable GTK4 widgets for the keybinding manager.
//!
//! # Components
//!
//! - `keybind_list.rs` - Scrollable list of keybindings
//! - `search_bar.rs` - Real-time search/filter
//! - `conflict_panel.rs` - Conflict warning banner
//! - `details_panel.rs` - Selected binding details
//! - `edit_dialog.rs` - Add/edit keybinding dialog
//! - `backup_dialog.rs` - Backup management dialog

mod conflict_panel;
mod edit_dialog;
mod keybind_list;
mod search_bar;
mod details_panel;
mod backup_dialog;
pub mod conflict_resolution_dialog;

pub use conflict_panel::ConflictPanel;
pub use edit_dialog::EditDialog;
pub use keybind_list::KeybindList;
pub use search_bar::SearchBar;
pub use details_panel::DetailsPanel;
pub use backup_dialog::BackupDialog;