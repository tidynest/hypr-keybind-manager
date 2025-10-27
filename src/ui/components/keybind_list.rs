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

//! Keybinding list component
//!
//! Displays all keybindings in a scrollable list view.
//! Each row shows the key combination, dispatcher, and arguments.

use gtk4::{prelude::*, Box as GtkBox, Label, ListBox, Orientation, ScrolledWindow};
use std::{cell::RefCell, rc::Rc};

use crate::core::types::Keybinding;
use crate::ui::Controller;

/// Displays a scrollable list of keybindings
pub struct KeybindList {
    /// Root widget (scrollable container)
    widget: ScrolledWindow,
    /// List box containing rows
    list_box: ListBox,
    /// Controller reference for data access
    controller: Rc<Controller>,
    /// Cache of currently displayed bindings
    current_bindings: RefCell<Vec<Keybinding>>,
}

impl KeybindList {
    /// Creates a new keybinding list
    ///
    /// # Arguments
    /// * `controller` - Shared Controller reference
    ///
    /// # Example
    /// ```no_run
    /// use hypr_keybind_manager::ui::components::KeybindList;
    /// use hypr_keybind_manager::ui::Controller;
    /// use std::rc::Rc;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config_path = PathBuf::from("~/.config/hypr/hyprland.conf");
    /// let controller = Rc::new(Controller::new(config_path)?);
    /// let list = KeybindList::new(controller);
    /// list.refresh(); // Load initial data
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(controller: Rc<Controller>) -> Self {
        // Create scrollable container
        let scrolled_window = ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .build();

        // Create list box
        let list_box = ListBox::builder()
            .selection_mode(gtk4::SelectionMode::Single)  // Allow clicking rows
            .build();

        // Add list to scrolled window
        scrolled_window.set_child(Some(&list_box));

        Self {
            widget: scrolled_window,
            list_box,
            controller,
            current_bindings: RefCell::new(Vec::new()),
        }
    }

    /// Refreshes the list with all keybindings from Controller
    pub fn refresh(&self) {
        let bindings = self.controller.get_keybindings();
        self.update_with_bindings(bindings);
    }

    /// Updates the list with specific keybindings (used for filtering)
    ///
    /// # Arguments
    /// * `bindings` - Keybindings to display
    pub fn update_with_bindings(&self, bindings: Vec<Keybinding>) {
        // Clear existing rows
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        // Cache the bindings
        *self.current_bindings.borrow_mut() = bindings.clone();

        // Add new rows with alternating colours
        for (index, binding) in bindings.iter().enumerate() {
            let row = self.create_row(binding, index);
            self.list_box.append(&row);
        }
    }

    /// Create a single row widget for a keybinding
    fn create_row(&self, binding: &Keybinding, index: usize) -> GtkBox {
        // Horizontal box for row layout
        let row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(20)
            .margin_start(10)
            .margin_end(10)
            .margin_top(5)
            .margin_bottom(5)
            .build();

        // Add subtle alternating background colour for every other row
        if index % 2 == 0 {
            // Even rows: Slightly lighter background
            row.add_css_class("even-row");
        } else {
            // Odd rows: Slightly darker background (Default background colour)
            row.add_css_class("odd-row");
        }

        // Key combination (e.g., "SUPER+K")
        let key_label = Label::builder()
            .label(format!("{}", binding.key_combo))
            .width_chars(15)  // Fixed width for alignment
            .xalign(0.0)
            .build();

        // Dispatcher (e.g., "exec", "killactive")
        let dispatcher_label = Label::builder()
            .label(&binding.dispatcher)
            .width_chars(15)
            .xalign(0.0)
            .build();

        // Arguments (e.g., "firefox") - optional
        let args_text = binding
            .args
            .as_deref()
            .unwrap_or("");
        let args_label = Label::builder()
            .label(args_text)
            .xalign(0.0)
            .hexpand(true)  // Take remaining space
            .build();

        // Add tooltip to show full arguments on hover
        if let Some(full_args) = &binding.args {
            if full_args.len() > 40 {
                args_label.set_can_target(true);
                args_label.set_has_tooltip(true);
                args_label.set_tooltip_text(Some(full_args));
            }
        }

        // Assemble row
        row.append(&key_label);
        row.append(&dispatcher_label);
        row.append(&args_label);

        row
    }

    /// Returns the root widget for adding to parent container
    pub fn widget(&self) -> &ScrolledWindow {
        &self.widget
    }

    /// Get a binding by its current display index.
    ///
    /// Returns the keybinding at the specified index in the currently displayed list.
    /// This accounts for any active search filters.
    ///
    /// # Arguments
    ///
    /// * `index` - The zero-based index in the current display
    ///
    /// # Returns
    ///
    /// * `Some(Keybinding)` if the index is valid
    /// * `None` if the index is out of bounds
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(binding) = keybind_list.get_binding_at_index(2) {
    ///     println!("Third binding: {}", binding.key_combo);
    /// }
    /// ```
    pub fn get_binding_at_index(&self, index: usize) -> Option<Keybinding> {
        let bindings = self.current_bindings.borrow();
        bindings.get(index).cloned()
    }

    /// Get a reference to the internal ListBox widget.
    ///
    /// This is used for connecting signals (e.g., row selection).
    ///
    /// # Returns
    ///
    /// Reference to the `ListBox` widget
    pub fn list_box(&self) -> &ListBox {
        &self.list_box
    }

    /// Returns count of currently displayed bindings
    pub fn count(&self) -> usize {
        self.current_bindings.borrow().len()
    }
}