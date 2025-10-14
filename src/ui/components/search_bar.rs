//! Search bar component
//!
//! Provides real-time filtering of keybindings as the user types.

use gtk4::prelude::*;
use gtk4::SearchEntry;

/// Search bar for filtering keybindings
pub struct SearchBar {
    /// Root widget (search entry)
    widget: SearchEntry,
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
        let entry = SearchEntry::builder()
            .placeholder_text("Search keybindings...")
            .hexpand(true)
            .margin_start(10)
            .margin_end(10)
            .margin_top(10)
            .margin_bottom(10)
            .build();

        Self { widget: entry }
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
