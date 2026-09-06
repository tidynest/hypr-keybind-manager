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

//! Keybinding list component
//!
//! Displays all keybindings in a scrollable list view.
//! Each row shows the key combination, dispatcher, and arguments.

use gtk4::{
    Box as GtkBox, Grid, Label, ListBox, Orientation, ScrolledWindow, pango::EllipsizeMode,
    prelude::*,
};
use std::{cell::RefCell, collections::HashSet, rc::Rc};

use crate::{
    core::types::{KeyCombo, Keybinding},
    ui::Controller,
};

const KEY_COLUMN_WIDTH: i32 = 190;
const DISPATCHER_COLUMN_WIDTH: i32 = 140;

/// Displays a scrollable list of keybindings
pub struct KeybindList {
    /// Root widget (list plus footer)
    widget: GtkBox,
    /// List box containing rows
    list_box: ListBox,
    /// Footer summarising shown/total/conflict counts
    footer: Label,
    /// Controller reference for data access
    controller: Rc<Controller>,
    /// Cache of currently displayed bindings
    current_bindings: Rc<RefCell<Vec<Keybinding>>>,
}

impl KeybindList {
    /// Creates a new keybinding list
    ///
    /// # Arguments
    /// * `controller` - Shared Controller reference
    ///
    /// # Example
    /// ```no_run
    /// use hypr_keybind_manager::ui::{components::KeybindList, Controller};
    /// use std::{path::PathBuf, rc::Rc};
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
            .selection_mode(gtk4::SelectionMode::Single) // Allow clicking rows
            .activate_on_single_click(false) // Double-click or Enter activates
            .build();

        let placeholder = Label::new(Some("No keybindings match"));
        placeholder.add_css_class("dim-label");
        placeholder.set_margin_top(24);
        list_box.set_placeholder(Some(&placeholder));

        // Add list to scrolled window
        scrolled_window.set_child(Some(&list_box));

        let footer = Label::builder().xalign(0.0).margin_start(8).build();
        footer.add_css_class("list-footer");

        let widget = GtkBox::new(Orientation::Vertical, 6);
        widget.append(&scrolled_window);
        widget.append(&footer);

        Self {
            widget,
            list_box,
            footer,
            controller,
            current_bindings: Rc::new(RefCell::new(Vec::new())),
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

        let conflicts = self.controller.get_conflicts();
        let conflict_keys: HashSet<KeyCombo> =
            conflicts.iter().map(|c| c.key_combo.clone()).collect();

        // Add new rows with alternating colours
        for (index, binding) in bindings.iter().enumerate() {
            let in_conflict = conflict_keys.contains(&binding.key_combo);
            let row = self.create_row(binding, index, in_conflict);
            self.list_box.append(&row);
        }

        self.footer.set_label(&format!(
            "{} of {} keybindings shown · {} conflicts",
            bindings.len(),
            self.controller.keybinding_count(),
            conflicts.len()
        ));
    }

    /// Create a single row widget for a keybinding
    fn create_row(&self, binding: &Keybinding, index: usize, in_conflict: bool) -> GtkBox {
        let row = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .margin_start(8)
            .margin_end(8)
            .margin_top(3)
            .margin_bottom(3)
            .build();

        if index % 2 == 0 {
            row.add_css_class("even-row");
        } else {
            row.add_css_class("odd-row");
        }
        if in_conflict {
            row.add_css_class("conflict-row");
            row.set_tooltip_text(Some("This key combination is bound more than once"));
        } else if let Some(description) = &binding.description {
            row.set_tooltip_text(Some(description));
        }

        let grid = Grid::builder()
            .column_spacing(16)
            .margin_start(10)
            .margin_end(10)
            .margin_top(8)
            .margin_bottom(8)
            .hexpand(true)
            .build();

        let key_text = match &binding.submap {
            Some(submap) => format!("[{submap}] {}", binding.key_combo),
            None => binding.key_combo.to_string(),
        };
        let key_label = Label::builder()
            .label(key_text)
            .xalign(0.0)
            .width_request(KEY_COLUMN_WIDTH)
            .build();
        key_label.add_css_class("list-key-column");

        let dispatcher_label = Label::builder()
            .label(&binding.dispatcher)
            .xalign(0.0)
            .width_request(DISPATCHER_COLUMN_WIDTH)
            .build();
        dispatcher_label.add_css_class("list-dispatcher-column");

        let args_text = binding.args.as_deref().unwrap_or("");
        let args_label = Label::builder()
            .label(args_text)
            .xalign(0.0)
            .hexpand(true)
            .ellipsize(EllipsizeMode::End)
            .build();
        args_label.add_css_class("list-args-column");

        if let Some(full_args) = &binding.args {
            if full_args.len() > 40 {
                args_label.set_can_target(true);
                args_label.set_has_tooltip(true);
                args_label.set_tooltip_text(Some(full_args));
            }
        }

        grid.attach(&key_label, 0, 0, 1, 1);
        grid.attach(&dispatcher_label, 1, 0, 1, 1);
        grid.attach(&args_label, 2, 0, 1, 1);
        row.append(&grid);

        row
    }

    /// Returns the root widget for adding to parent container
    pub fn widget(&self) -> &GtkBox {
        &self.widget
    }

    /// Runs `callback` with the binding of a row activated by double-click or Enter
    pub fn connect_activate<F>(&self, callback: F)
    where
        F: Fn(Keybinding) + 'static,
    {
        let current = self.current_bindings.clone();
        self.list_box.connect_row_activated(move |_, row| {
            let binding = current.borrow().get(row.index() as usize).cloned();
            if let Some(binding) = binding {
                callback(binding);
            }
        });
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
