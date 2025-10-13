//! Keybinding list component
//!
//! Displays all keybindings in a scrollable list view.
//! Each row shows the key combination, dispatcher, and arguments.

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, ListBox, Orientation, ScrolledWindow};
use std::cell::RefCell;
use std::rc::Rc;

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
    /// let controller = Rc::new(Controller::new(config_path)?);
    /// let list = KeybindList::new(controller);
    /// list.refresh(); // Load initial data
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

        // Add new rows
        for binding in bindings {
            let row = self.create_row(&binding);
            self.list_box.append(&row);
        }
    }

    /// Creates a single row widget for a keybinding
    fn create_row(&self, binding: &Keybinding) -> GtkBox {
        // Horizontalbox for row layout
        let row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(20)
            .margin_start(10)
            .margin_end(10)
            .margin_top(5)
            .margin_bottom(5)
            .build();

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

    /// Returns count of currently displayed bindings
    pub fn count(&self) -> usize {
        self.current_bindings.borrow().len()
    }
}