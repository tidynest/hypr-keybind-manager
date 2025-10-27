// Copyright 2025 bakri (tidynest@proton.me)
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

//! Search bar component
//!
//! Provides real-time filtering of keybindings as the user types.

use gtk4::{prelude::*, SearchEntry};

/// Search bar for filtering keybindings
pub struct SearchBar {
    /// Root widget (search entry)
    widget: SearchEntry,
}

impl Default for SearchBar {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchBar {
    /// Creates a new search bar
    ///
    /// Returns just the widget - parent is responsible for wiring
    /// up the search functionality to avoid instance sharing bugs.
    ///
    /// # Example
    /// ```no_run
    /// use hypr_keybind_manager::ui::components::SearchBar;
    /// use gtk4::prelude::*;
    ///
    /// let search_bar = SearchBar::new();
    ///
    /// // Parent wires up search functionality:
    /// search_bar.widget().connect_search_changed(move |entry| {
    ///     let query = entry.text().to_string();
    ///     // ... filter logic here
    /// });
    /// ```
    pub fn new() -> Self {
        // Create search entry widget
        let widget = SearchEntry::builder()
            .placeholder_text("Search keybindings...")
            .build();

        Self { widget }
    }
    /// Returns the root widget for adding to parent container
    pub fn widget(&self) -> &SearchEntry {
        &self.widget
    }

    /// Clears the search query and resets the list
    pub fn clear(&self) {
        self.widget.set_text("");
    }
}
