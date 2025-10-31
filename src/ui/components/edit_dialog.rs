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

//! Edit dialog component for adding and editing keybindings.
//!
//! Provides a modern GTK4 Window-based dialog for creating new keybindings
//! or editing existing ones. The dialog includes:
//!
//! - Pre-filled form fields for editing
//! - Real-time validation with error feedback
//! - Recursive error handling (user can fix and retry)
//! - Modal dialog pattern with Save/Cancel buttons
//!
//! # Architecture
//!
//! Uses GTK4 Window instead of deprecated Dialog API, with manual button
//! layout and response tracking via `Rc<Cell<Option<DialogResponse>>>`.
//!
//! # Example
//!
//! ```no_run
//! use hypr_keybind_manager::{core::types::{Keybinding, KeyCombo, Modifier, BindType}, ui::components::EditDialog};
//! use gtk4::ApplicationWindow;
//!
//! # fn example(parent: &ApplicationWindow) {
//! let binding = Keybinding {
//!     key_combo: KeyCombo::new(vec![Modifier::Super], "K"),
//!     bind_type: BindType::Bind,
//!     dispatcher: "exec".to_string(),
//!     args: Some("firefox".to_string()),
//! };
//!
//! let dialog = EditDialog::new(parent, &binding);
//! if let Some(updated_binding) = dialog.show_and_wait() {
//!     println!("User saved: {}", updated_binding.key_combo);
//! }
//! # }
//! ```

use gtk4::{
    prelude::*, ApplicationWindow, Box as GtkBox, Button, Entry, Grid, Label, Orientation, Window,
};
use std::{cell::Cell, rc::Rc};

use crate::core::types::{BindType, KeyCombo, Keybinding, Modifier};

/// Dialog for editing an existing keybinding
pub struct EditDialog {
    dialog_window: Window,
    key_entry: Entry,
    dispatcher_entry: Entry,
    args_entry: Entry,
    bind_type_entry: Entry,
    response: Rc<Cell<Option<DialogResponse>>>,
}

#[derive(Clone, Debug, Copy, PartialEq)]
enum DialogResponse {
    Save,
    Cancel,
}

impl EditDialog {
    /// Creates a new edit dialog pre-filled with the binding's current values
    pub fn new(parent: &ApplicationWindow, binding: &Keybinding) -> Self {
        // Create the window
        let dialog_window = Window::builder()
            .title("✏️ Edit Keybinding")
            .modal(true)
            .transient_for(parent)
            .default_width(450)
            .default_height(300)
            .resizable(false)
            .build();

        // Create a grid for the form layout
        let grid = Grid::builder()
            .row_spacing(12)
            .column_spacing(12)
            .margin_start(20)
            .margin_end(20)
            .margin_top(20)
            .margin_bottom(20)
            .build();

        // Row 0: Key Combination
        let key_label = Label::builder()
            .label("🎹 Key Combination:")
            .halign(gtk4::Align::End)
            .build();
        let key_entry = Entry::builder()
            .text(binding.key_combo.to_string())
            .placeholder_text("e.g., SUPER+SHIFT+M")
            .hexpand(true)
            .build();
        grid.attach(&key_label, 0, 0, 1, 1);
        grid.attach(&key_entry, 1, 0, 1, 1);

        // Row 1: Dispatcher
        let dispatcher_label = Label::builder()
            .label("⚡ Dispatcher:")
            .halign(gtk4::Align::End)
            .build();
        let dispatcher_entry = Entry::builder()
            .text(&binding.dispatcher)
            .placeholder_text("e.g., exec, workspace, killactive")
            .hexpand(true)
            .build();
        grid.attach(&dispatcher_label, 0, 1, 1, 1);
        grid.attach(&dispatcher_entry, 1, 1, 1, 1);

        // Row 2: Arguments
        let args_label = Label::builder()
            .label("📝 Arguments:")
            .halign(gtk4::Align::End)
            .build();
        let args_entry = Entry::builder()
            .text(binding.args.as_deref().unwrap_or(""))
            .placeholder_text("Optional arguments")
            .hexpand(true)
            .build();
        grid.attach(&args_label, 0, 2, 1, 1);
        grid.attach(&args_entry, 1, 2, 1, 1);

        // Row 3: Bind Type
        let bind_type_label = Label::builder()
            .label("🔗 Bind Type:")
            .halign(gtk4::Align::End)
            .build();
        let bind_type_entry = Entry::builder()
            .text(binding.bind_type.to_string())
            .placeholder_text("bind, binde, bindm, etc.")
            .hexpand(true)
            .build();
        grid.attach(&bind_type_label, 0, 3, 1, 1);
        grid.attach(&bind_type_entry, 1, 3, 1, 1);

        // Create button box at the bottom (replaces dialog.add_button)
        let button_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .halign(gtk4::Align::End)
            .margin_start(20)
            .margin_end(20)
            .margin_bottom(20)
            .build();

        let cancel_button = Button::builder().label("Cancel").build();

        let save_button = Button::builder().label("💾 Save").build();
        save_button.add_css_class("suggested-action");

        button_box.append(&cancel_button);
        button_box.append(&save_button);

        // Create main vertical box
        let main_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .build();

        main_box.append(&grid);
        main_box.append(&button_box);

        dialog_window.set_child(Some(&main_box));

        // Set up response tracking
        let response: Rc<Cell<Option<DialogResponse>>> = Rc::new(Cell::new(None));

        // Connect Cancel button
        {
            let response = response.clone();
            let window = dialog_window.clone();
            let key_entry = key_entry.clone();
            let dispatcher_entry = dispatcher_entry.clone();
            let args_entry = args_entry.clone();
            let bind_type_entry = bind_type_entry.clone();

            cancel_button.connect_clicked(move |_| {
                // Clear selections
                key_entry.select_region(0, 0);
                dispatcher_entry.select_region(0, 0);
                args_entry.select_region(0, 0);
                bind_type_entry.select_region(0, 0);

                response.set(Some(DialogResponse::Cancel));
                window.close();
            });
        }

        // Connect Save button
        {
            let response = response.clone();
            let key_entry = key_entry.clone();
            let dispatcher_entry = dispatcher_entry.clone();
            let args_entry = args_entry.clone();
            let bind_type_entry = bind_type_entry.clone();

            save_button.connect_clicked(move |_| {
                // Clear selections
                key_entry.select_region(0, 0);
                dispatcher_entry.select_region(0, 0);
                args_entry.select_region(0, 0);
                bind_type_entry.select_region(0, 0);

                response.set(Some(DialogResponse::Save));
            });
        }

        // Handle window close (X button) as Cancel
        {
            let response = response.clone();
            dialog_window.connect_close_request(move |_| {
                if response.get().is_none() {
                    response.set(Some(DialogResponse::Cancel));
                }
                glib::Propagation::Proceed
            });
        }

        Self {
            dialog_window,
            key_entry,
            dispatcher_entry,
            args_entry,
            bind_type_entry,
            response,
        }
    }

    /// Clears text selections in all entry fields
    fn clear_selections(&self) {
        self.key_entry.select_region(0, 0);
        self.dispatcher_entry.select_region(0, 0);
        self.args_entry.select_region(0, 0);
        self.bind_type_entry.select_region(0, 0);
    }

    /// Parses the form fields and returns a new Keybinding if valid
    fn parse_binding(&self) -> Result<Keybinding, String> {
        // Get values from entries
        let key_text = self.key_entry.text().to_string();
        let dispatcher = self.dispatcher_entry.text().to_string();
        let args_text = self.args_entry.text().to_string();
        let bind_type_text = self.bind_type_entry.text().to_string();

        // Validate required fields
        if key_text.trim().is_empty() {
            return Err("Key combination cannot be empty".to_string());
        }
        if dispatcher.trim().is_empty() {
            return Err("Dispatcher cannot be empty".to_string());
        }
        if bind_type_text.trim().is_empty() {
            return Err("Bind type cannot be empty".to_string());
        }

        // Parse key combination
        let parts: Vec<&str> = key_text.split('+').map(|s| s.trim()).collect();
        if parts.is_empty() {
            return Err("Invalid key combination format".to_string());
        }

        let key = parts.last().unwrap().to_string();
        let modifier_strings = &parts[..parts.len() - 1];

        let mut modifiers = Vec::new();
        for mod_str in modifier_strings {
            let modifier = match mod_str.to_uppercase().as_str() {
                "SUPER" => Modifier::Super,
                "SHIFT" => Modifier::Shift,
                "CTRL" | "CONTROL" => Modifier::Ctrl,
                "ALT" => Modifier::Alt,
                other => return Err(format!("Unknown modifier: {}", other)),
            };
            modifiers.push(modifier);
        }

        let key_combo = KeyCombo { modifiers, key };

        // Parse bind type - match the string manually
        let bind_type = match bind_type_text.to_lowercase().as_str() {
            "bind" => BindType::Bind,
            "binde" => BindType::BindE,
            "bindm" => BindType::BindM,
            "bindr" => BindType::BindR,
            "bindl" => BindType::BindL,
            _ => return Err(format!("Invalid bind type: {}", bind_type_text)),
        };

        // Optional arguments
        let args = if args_text.trim().is_empty() {
            None
        } else {
            Some(args_text.trim().to_string())
        };

        // Build the new keybinding (only include fields that exist!)
        Ok(Keybinding {
            bind_type,
            key_combo,
            dispatcher: dispatcher.trim().to_string(),
            args,
        })
    }

    /// Shows the dialog and waits for user response
    pub fn show_and_wait(self) -> Option<Keybinding> {
        self.response.set(None);
        self.dialog_window.present();

        let main_context = glib::MainContext::default();
        self.clear_selections();

        // Keep looping until we get a valid response or user cancels
        loop {
            // Wait for a response
            while self.response.get().is_none() && self.dialog_window.is_visible() {
                main_context.iteration(true);
            }

            // Check what the user did
            match self.response.get() {
                Some(DialogResponse::Save) => {
                    match self.parse_binding() {
                        Ok(binding) => {
                            self.dialog_window.close();
                            return Some(binding);
                        }
                        Err(e) => {
                            self.show_error(&e);
                            // Reset response and continue the outer loop
                            self.response.set(None);
                        }
                    }
                }
                Some(DialogResponse::Cancel) => {
                    self.dialog_window.close();
                    return None;
                }
                None => {
                    self.dialog_window.close();
                    return None;
                }
            }
        }
    }

    /// Shows an error message in a modal dialog
    fn show_error(&self, message: &str) {
        let error_window = Window::builder()
            .title("❌ Invalid Input")
            .modal(true)
            .transient_for(&self.dialog_window)
            .default_width(350)
            .default_height(150)
            .resizable(false)
            .build();

        let vbox = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .margin_start(20)
            .margin_end(20)
            .margin_top(20)
            .margin_bottom(20)
            .build();

        let label = Label::builder()
            .label(message)
            .wrap(true)
            .justify(gtk4::Justification::Center)
            .build();

        let ok_button = Button::builder()
            .label("Ok")
            .halign(gtk4::Align::Center)
            .build();

        vbox.append(&label);
        vbox.append(&ok_button);

        error_window.set_child(Some(&vbox));

        let error_window_clone = error_window.clone();
        ok_button.connect_clicked(move |_| {
            error_window_clone.close();
        });

        error_window.present();

        // Wait for error dialog to close
        let main_context = glib::MainContext::default();
        while error_window.is_visible() {
            main_context.iteration(true);
        }
    }
}
