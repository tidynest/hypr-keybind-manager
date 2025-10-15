use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Box as GtkBox, Button, Entry, Grid, Label, Orientation, Window};
use std::cell::Cell;
use std::rc::Rc;

use crate::core::types::{Keybinding, KeyCombo, Modifier, BindType};

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
            .text(&binding.key_combo.to_string())
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
            .text(&binding.bind_type.to_string())
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

        let cancel_button = Button::builder()
            .label("Cancel")
            .build();

        let save_button = Button::builder()
            .label("💾 Save")
            .build();
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
            cancel_button.connect_clicked(move |_| {
                eprintln!("🔍 DEBUG: Cancel button clicked!");
                eprintln!("🔍 DEBUG: Setting response to Cancel");
                response.set(Some(DialogResponse::Cancel));
                eprintln!("🔍 DEBUG: Response set, current value: {:?}", response.get());
                eprintln!("🔍 DEBUG: Closing window");
                window.close();
                eprintln!("🔍 DEBUG: Window closed");
            });
        }

        // Connect Save button
        {
            let response = response.clone();
            let window = dialog_window.clone();
            save_button.connect_clicked(move |_| {
                eprintln!("🔍 DEBUG: Save button clicked!");
                eprintln!("🔍 DEBUG: Setting response to Save");
                response.set(Some(DialogResponse::Save));
                eprintln!("🔍 DEBUG: Response set, current value: {:?}", response.get());
                eprintln!("🔍 DEBUG: Closing window");
                window.close();
                eprintln!("🔍 DEBUG: Window closed");
            });
        }

        // Handle window close (X button) as Cancel
        {
            let response = response.clone();
            dialog_window.connect_close_request(move |_| {
                eprintln!("🔍 DEBUG: Window X button clicked!");
                if response.get().is_none() {
                    eprintln!("🔍 DEBUG: No response set yet, setting to Cancel");
                    response.set(Some(DialogResponse::Cancel));
                } else {
                    eprintln!("🔍 DEBUG: Response already set to {:?}", response.get());
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
        // Reset response (in case of reuse, though we consume self)
        self.response.set(None);

        eprintln!("📝 DEBUG: Presenting window...");
        self.dialog_window.present();

        eprintln!("🔄 DEBUG: Entering event loop...");
        // Keep the GTK main loop until we get a response
        let main_context = glib::MainContext::default();

        while self.response.get().is_none() && self.dialog_window.is_visible() {
            main_context.iteration(true);
        }

        eprintln!("🔍 DEBUG: Loop exited!");
        eprintln!("🔍 DEBUG: self.response.get() = {:?}", self.response.get());
        eprintln!("🔍 DEBUG: window.is_visible() = {}", self.dialog_window.is_visible());

        // Now we have a response (or a window was closed)!
        match self.response.get() {
            Some(DialogResponse::Save) => {
                eprintln!("✅ DEBUG: Response is Save, parsing binding...");

                // User clicked Save - try to parse the binding
                match self.parse_binding() {
                    Ok(binding) => {
                        eprintln!("✅ DEBUG: Parsing successful!");
                        self.dialog_window.close();
                        Some(binding)
                    }
                    Err(e) => {
                        eprintln!("❌ DEBUG: Parsing failed: {}", e);

                        // Show error but don't close dialog
                        self.show_error(&e);

                        // Reset and wait again (recursive pattern)
                        self.response.set(None);
                        eprintln!("🔄 DEBUG: Recursing for user to fix input...");

                        // Try again recursively
                        self.show_and_wait()
                    }
                }
            }
            Some(DialogResponse::Cancel) => {
                eprintln!("🚫 DEBUG: Response is Cancel");
                self.dialog_window.close();
                None
            }
            None => {
                eprintln!("⚠️ DEBUG: Response is None (window closed via X)");
                self.dialog_window.close();
                None
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