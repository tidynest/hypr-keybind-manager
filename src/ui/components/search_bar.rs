//! Search bar component
//!
//! Provides real-time filtering of keybindings as the user types.

use gtk4::prelude::*;
use gtk4::SearchEntry;
use std::cell::RefCell;
use std::rc::Rc;

use crate::ui::components::KeybindList;
use crate::ui::Controller;

/// Search bar for filtering keybindings
pub struct SearchBar {
    /// Root widget (search entry)
    widget: SearchEntry,
}

impl SearchBar {
    /// Creates a new search bar
    ///
    /// # Arguments
    /// * `keybind_list` - Shared KeybindList reference to update
    /// * `controller` - Shared Controller reference for filtering
    ///
    /// # Example
    /// ```no_run
    /// let controller = Rc::new(Controller::new(config_path)?);
    /// let list = Rc::new(RefCell::new(KeybindList::new(controller.clone())));
    /// let search = SearchBar::new(list, controller);
    /// ```
    pub fn new(
        keybind_list: Rc<RefCell<KeybindList>>,
        controller: Rc<Controller>,
    ) -> Self {
        // Creates search entry widget
        let entry = SearchEntry::builder()
            .placeholder_text("Search keybindings...")
            .hexpand(true)
            .margin_start(10)
            .margin_end(10)
            .margin_top(10)
            .margin_bottom(10)
            .build();

        // Connect search-changed signal for real-time filtering
        let keybind_list_clone = Rc::clone(&keybind_list);
        let controller_clone = Rc::clone(&controller);

        entry.connect_search_changed(move |entry| {
            // Get query text
            let query = entry.text().to_string();
            // Filter via controller
            let filered = controller_clone.filter_keybindings(&query);
            // Update list with filtered results
            keybind_list_clone.borrow().update_with_bindings(filered);
        });

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
