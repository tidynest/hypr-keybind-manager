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
//! Provides a GTK4 window-based dialog for creating and updating keybindings.
//! The dialog includes:
//! - pre-filled form fields for editing
//! - inline key-combo availability feedback
//! - clickable replacement suggestions for busy combos
//! - modal save/cancel flow with validation

use crate::{
    core::{
        sandbox,
        types::{BindType, KeyCombo, Keybinding, Modifier},
    },
    ui::controller::KeyComboAvailability,
    ui::Controller,
};
use gtk4::{
    gdk, prelude::*, ApplicationWindow, Box as GtkBox, Button, Entry, EventControllerKey, Grid,
    Label, Orientation, Switch, Window,
};
use std::{cell::Cell, rc::Rc};

/// Dialog for editing an existing keybinding
pub struct EditDialog {
    dialog_window: Window,
    key_entry: Entry,
    dispatcher_entry: Entry,
    args_entry: Entry,
    bind_type_entry: Entry,
    sandbox_switch: Switch,
    sandbox_label: Label,
    availability_label: Label,
    suggestion_box: GtkBox,
    response: Rc<Cell<Option<DialogResponse>>>,
    controller: Rc<Controller>,
    original_binding: Option<Keybinding>,
}

#[derive(Clone, Debug, Copy, PartialEq)]
enum DialogResponse {
    Save,
    Cancel,
}

impl EditDialog {
    /// Creates a new edit dialog pre-filled with the binding's current values.
    pub fn new(
        parent: &ApplicationWindow,
        controller: Rc<Controller>,
        binding: &Keybinding,
        original_binding: Option<Keybinding>,
    ) -> Self {
        let title = if original_binding.is_some() {
            "✏️ Edit Keybinding"
        } else {
            "➕ Add Keybinding"
        };

        let dialog_window = Window::builder()
            .title(title)
            .modal(true)
            .transient_for(parent)
            .default_width(480)
            .default_height(360)
            .resizable(false)
            .build();

        let key_controller = EventControllerKey::new();
        let dialog_window_for_escape = dialog_window.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if key == gdk::Key::Escape {
                dialog_window_for_escape.close();
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        dialog_window.add_controller(key_controller);

        let grid = Grid::builder()
            .row_spacing(12)
            .column_spacing(12)
            .margin_start(20)
            .margin_end(20)
            .margin_top(20)
            .margin_bottom(12)
            .build();

        let key_label = Label::builder()
            .label("🎹 Key Combination:")
            .halign(gtk4::Align::End)
            .build();
        let key_entry = Entry::builder()
            .text(binding.key_combo.to_string())
            .placeholder_text("e.g., SUPER+SHIFT+M")
            .hexpand(true)
            .build();
        key_entry.set_tooltip_text(Some("Enter modifiers and key using MOD+KEY format"));
        grid.attach(&key_label, 0, 0, 1, 1);
        grid.attach(&key_entry, 1, 0, 1, 1);

        let availability_label = Label::builder()
            .label("Enter a key combination to check availability.")
            .halign(gtk4::Align::Start)
            .xalign(0.0)
            .wrap(true)
            .visible(true)
            .build();
        availability_label.add_css_class("availability-hint");
        grid.attach(&availability_label, 1, 1, 1, 1);

        let suggestion_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .hexpand(true)
            .visible(false)
            .build();
        suggestion_box.add_css_class("suggestion-row");
        grid.attach(&suggestion_box, 1, 2, 1, 1);

        let dispatcher_label = Label::builder()
            .label("⚡ Dispatcher:")
            .halign(gtk4::Align::End)
            .build();
        let dispatcher_entry = Entry::builder()
            .text(&binding.dispatcher)
            .placeholder_text("e.g., exec, workspace, killactive")
            .hexpand(true)
            .build();
        dispatcher_entry.set_tooltip_text(Some("Enter the Hyprland dispatcher to run"));
        grid.attach(&dispatcher_label, 0, 3, 1, 1);
        grid.attach(&dispatcher_entry, 1, 3, 1, 1);

        let args_label = Label::builder()
            .label("📝 Arguments:")
            .halign(gtk4::Align::End)
            .build();
        let args_entry = Entry::builder()
            .text(binding.args.as_deref().unwrap_or(""))
            .placeholder_text("Optional arguments")
            .hexpand(true)
            .build();
        args_entry.set_tooltip_text(Some("Optional dispatcher arguments"));
        grid.attach(&args_label, 0, 4, 1, 1);
        grid.attach(&args_entry, 1, 4, 1, 1);

        let bind_type_label = Label::builder()
            .label("🔗 Bind Type:")
            .halign(gtk4::Align::End)
            .build();
        let bind_type_entry = Entry::builder()
            .text(binding.bind_type.to_string())
            .placeholder_text("bind, binde, bindm, etc.")
            .hexpand(true)
            .build();
        bind_type_entry.set_tooltip_text(Some("Choose the Hyprland bind variant"));
        grid.attach(&bind_type_label, 0, 5, 1, 1);
        grid.attach(&bind_type_entry, 1, 5, 1, 1);

        let sandbox_label = Label::builder()
            .label("🛡️ Bubblewrap Sandbox:")
            .halign(gtk4::Align::End)
            .build();
        let sandbox_switch = Switch::builder()
            .halign(gtk4::Align::Start)
            .tooltip_text("Wrap exec commands in a Bubblewrap sandbox with no network access")
            .build();
        let sandbox_active = binding.args.as_deref().is_some_and(sandbox::is_wrapped)
            && binding.dispatcher == "exec";
        sandbox_switch.set_active(sandbox_active);
        grid.attach(&sandbox_label, 0, 6, 1, 1);
        grid.attach(&sandbox_switch, 1, 6, 1, 1);

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
        save_button.set_receives_default(true);

        button_box.append(&cancel_button);
        button_box.append(&save_button);

        let main_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .build();
        main_box.append(&grid);
        main_box.append(&button_box);
        dialog_window.set_child(Some(&main_box));
        dialog_window.set_default_widget(Some(&save_button));

        let response: Rc<Cell<Option<DialogResponse>>> = Rc::new(Cell::new(None));

        {
            let response = response.clone();
            let window = dialog_window.clone();
            let key_entry = key_entry.clone();
            let dispatcher_entry = dispatcher_entry.clone();
            let args_entry = args_entry.clone();
            let bind_type_entry = bind_type_entry.clone();

            cancel_button.connect_clicked(move |_| {
                key_entry.select_region(0, 0);
                dispatcher_entry.select_region(0, 0);
                args_entry.select_region(0, 0);
                bind_type_entry.select_region(0, 0);

                response.set(Some(DialogResponse::Cancel));
                window.close();
            });
        }

        {
            let response = response.clone();
            let key_entry = key_entry.clone();
            let dispatcher_entry = dispatcher_entry.clone();
            let args_entry = args_entry.clone();
            let bind_type_entry = bind_type_entry.clone();

            save_button.connect_clicked(move |_| {
                key_entry.select_region(0, 0);
                dispatcher_entry.select_region(0, 0);
                args_entry.select_region(0, 0);
                bind_type_entry.select_region(0, 0);

                response.set(Some(DialogResponse::Save));
            });
        }

        {
            let response = response.clone();
            dialog_window.connect_close_request(move |_| {
                if response.get().is_none() {
                    response.set(Some(DialogResponse::Cancel));
                }
                glib::Propagation::Proceed
            });
        }

        let visible_args = binding
            .args
            .as_deref()
            .and_then(sandbox::unwrap_command)
            .or_else(|| binding.args.clone())
            .unwrap_or_default();
        args_entry.set_text(&visible_args);

        let dialog = Self {
            dialog_window,
            key_entry,
            dispatcher_entry,
            args_entry,
            bind_type_entry,
            sandbox_switch,
            sandbox_label,
            availability_label,
            suggestion_box,
            response,
            controller,
            original_binding,
        };

        dialog.connect_key_feedback();
        dialog.connect_sandbox_feedback();
        dialog.refresh_sandbox_controls();
        dialog.refresh_key_combo_feedback();
        dialog
    }

    fn connect_key_feedback(&self) {
        let controller = self.controller.clone();
        let original_binding = self.original_binding.clone();
        let key_entry = self.key_entry.clone();
        let availability_label = self.availability_label.clone();
        let suggestion_box = self.suggestion_box.clone();

        self.key_entry.connect_changed(move |_| {
            refresh_key_combo_feedback_widgets(
                &controller,
                original_binding.as_ref(),
                &key_entry,
                &availability_label,
                &suggestion_box,
            );
        });
    }

    fn refresh_key_combo_feedback(&self) {
        refresh_key_combo_feedback_widgets(
            &self.controller,
            self.original_binding.as_ref(),
            &self.key_entry,
            &self.availability_label,
            &self.suggestion_box,
        );
    }

    fn connect_sandbox_feedback(&self) {
        let dispatcher_entry = self.dispatcher_entry.clone();
        let sandbox_switch = self.sandbox_switch.clone();
        let sandbox_label = self.sandbox_label.clone();

        self.dispatcher_entry.connect_changed(move |_| {
            refresh_sandbox_controls_widgets(&dispatcher_entry, &sandbox_switch, &sandbox_label);
        });
    }

    fn refresh_sandbox_controls(&self) {
        refresh_sandbox_controls_widgets(
            &self.dispatcher_entry,
            &self.sandbox_switch,
            &self.sandbox_label,
        );
    }

    /// Clears text selections in all entry fields.
    fn clear_selections(&self) {
        self.key_entry.select_region(0, 0);
        self.dispatcher_entry.select_region(0, 0);
        self.args_entry.select_region(0, 0);
        self.bind_type_entry.select_region(0, 0);
    }

    /// Parses the form fields and returns a new Keybinding if valid.
    fn parse_binding(&self) -> Result<Keybinding, String> {
        let key_text = self.key_entry.text().to_string();
        let dispatcher = self.dispatcher_entry.text().to_string();
        let args_text = self.args_entry.text().to_string();
        let bind_type_text = self.bind_type_entry.text().to_string();

        let key_combo = parse_key_combo_text(&key_text)?
            .ok_or_else(|| "Key combination cannot be empty".to_string())?;

        if dispatcher.trim().is_empty() {
            return Err("Dispatcher cannot be empty".to_string());
        }
        if bind_type_text.trim().is_empty() {
            return Err("Bind type cannot be empty".to_string());
        }

        let bind_type = match bind_type_text.to_lowercase().as_str() {
            "bind" => BindType::Bind,
            "binde" => BindType::BindE,
            "bindm" => BindType::BindM,
            "bindr" => BindType::BindR,
            "bindl" => BindType::BindL,
            "bindel" => BindType::BindEL,
            _ => return Err(format!("Invalid bind type: {}", bind_type_text)),
        };

        let args = if args_text.trim().is_empty() {
            None
        } else {
            let trimmed = args_text.trim();
            if self.sandbox_switch.is_active() && dispatcher.trim().eq_ignore_ascii_case("exec") {
                Some(sandbox::wrap_command(trimmed)?)
            } else {
                Some(trimmed.to_string())
            }
        };

        Ok(Keybinding {
            bind_type,
            key_combo,
            dispatcher: dispatcher.trim().to_string(),
            args,
        })
    }

    /// Shows the dialog and waits for user response.
    pub fn show_and_wait(self) -> Option<Keybinding> {
        self.response.set(None);
        self.dialog_window.present();

        let main_context = glib::MainContext::default();
        self.clear_selections();

        loop {
            while self.response.get().is_none() && self.dialog_window.is_visible() {
                main_context.iteration(true);
            }

            match self.response.get() {
                Some(DialogResponse::Save) => match self.parse_binding() {
                    Ok(binding) => {
                        self.dialog_window.close();
                        return Some(binding);
                    }
                    Err(e) => {
                        self.show_error(&e);
                        self.response.set(None);
                    }
                },
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

    /// Shows an error message in a modal dialog.
    fn show_error(&self, message: &str) {
        let error_window = Window::builder()
            .title("❌ Invalid Input")
            .modal(true)
            .transient_for(&self.dialog_window)
            .default_width(350)
            .default_height(150)
            .resizable(false)
            .build();

        let key_controller = EventControllerKey::new();
        let error_window_for_escape = error_window.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if key == gdk::Key::Escape {
                error_window_for_escape.close();
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });

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

        let main_context = glib::MainContext::default();
        while error_window.is_visible() {
            main_context.iteration(true);
        }
    }
}

fn parse_key_combo_text(input: &str) -> Result<Option<KeyCombo>, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let parts: Vec<&str> = trimmed.split('+').map(str::trim).collect();
    if parts.iter().any(|part| part.is_empty()) {
        return Err("Use format MOD+KEY without empty segments.".to_string());
    }

    let key = parts
        .last()
        .ok_or_else(|| "Invalid key combination format".to_string())?;

    let mut modifiers = Vec::new();
    for modifier in &parts[..parts.len() - 1] {
        let parsed = match modifier.to_uppercase().as_str() {
            "SUPER" => Modifier::Super,
            "SHIFT" => Modifier::Shift,
            "CTRL" | "CONTROL" => Modifier::Ctrl,
            "ALT" => Modifier::Alt,
            other => return Err(format!("Unknown modifier: {}", other)),
        };
        modifiers.push(parsed);
    }

    Ok(Some(KeyCombo::new(modifiers, key)))
}

fn refresh_key_combo_feedback_widgets(
    controller: &Rc<Controller>,
    original_binding: Option<&Keybinding>,
    key_entry: &Entry,
    availability_label: &Label,
    suggestion_box: &GtkBox,
) {
    clear_suggestion_box(suggestion_box);

    let key_text = key_entry.text();
    match parse_key_combo_text(&key_text) {
        Ok(None) => set_feedback_state(
            availability_label,
            "Enter a key combination to check availability.",
            "availability-hint",
        ),
        Err(message) => set_feedback_state(availability_label, &message, "availability-warning"),
        Ok(Some(key_combo)) => {
            let assistance =
                controller.get_key_combo_assistance(Some(&key_combo), original_binding);
            match assistance.availability {
                KeyComboAvailability::Incomplete => {
                    set_feedback_state(
                        availability_label,
                        "Enter a key combination to check availability.",
                        "availability-hint",
                    );
                }
                KeyComboAvailability::Available => {
                    set_feedback_state(
                        availability_label,
                        "This key combination is currently free.",
                        "availability-available",
                    );
                }
                KeyComboAvailability::InUse(bindings) => {
                    let preview = bindings
                        .iter()
                        .take(2)
                        .map(describe_binding)
                        .collect::<Vec<_>>()
                        .join(" | ");
                    let suffix = if bindings.len() > 2 { " | ..." } else { "" };
                    let message = format!("Already in use by {}{}", preview, suffix);
                    set_feedback_state(availability_label, &message, "availability-warning");

                    if !assistance.suggestions.is_empty() {
                        for suggestion in assistance.suggestions {
                            let suggestion_text = suggestion.to_string();
                            let button = Button::builder().label(&suggestion_text).build();
                            button.add_css_class("suggestion-button");

                            let key_entry = key_entry.clone();
                            button.connect_clicked(move |_| {
                                key_entry.set_text(&suggestion_text);
                            });

                            suggestion_box.append(&button);
                        }
                        suggestion_box.set_visible(true);
                    }
                }
            }
        }
    }
}

fn clear_suggestion_box(suggestion_box: &GtkBox) {
    while let Some(child) = suggestion_box.first_child() {
        suggestion_box.remove(&child);
    }
    suggestion_box.set_visible(false);
}

fn set_feedback_state(label: &Label, text: &str, css_class: &str) {
    for class in [
        "availability-hint",
        "availability-available",
        "availability-warning",
    ] {
        label.remove_css_class(class);
    }

    label.set_label(text);
    label.add_css_class(css_class);
}

fn describe_binding(binding: &Keybinding) -> String {
    match &binding.args {
        Some(args) if !args.is_empty() => format!("{} {}", binding.dispatcher, args),
        _ => binding.dispatcher.clone(),
    }
}

fn refresh_sandbox_controls_widgets(
    dispatcher_entry: &Entry,
    sandbox_switch: &Switch,
    sandbox_label: &Label,
) {
    let enabled = dispatcher_entry.text().trim().eq_ignore_ascii_case("exec");
    sandbox_switch.set_sensitive(enabled);
    sandbox_label.set_sensitive(enabled);

    if enabled {
        sandbox_switch.set_tooltip_text(Some(
            "Wrap this exec command with Bubblewrap using a read-only system view and no network",
        ));
    } else {
        sandbox_switch.set_active(false);
        sandbox_switch.set_tooltip_text(Some(
            "Bubblewrap sandboxing is only available for exec bindings",
        ));
    }
}
