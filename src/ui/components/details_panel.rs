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

//! Details panel component for displaying selected keybinding information.
//!
//! Shows the selected binding's fields, the exact config line that
//! represents it, and which other bindings it conflicts with.

use gtk4::{
    Align, Box as GtkBox, Button, Frame, Grid, Label, Orientation, Separator,
    pango::WrapMode::WordChar, prelude::*,
};
use std::{cell::RefCell, rc::Rc};

use crate::{
    config::ConfigFormat,
    core::{
        lua_config::render_lua_bind,
        types::{BIND_FLAGS, Keybinding},
    },
    ui::{Controller, builders::header::icon_button},
};

/// Field rows in display order
const FIELDS: [&str; 7] = [
    "Key combo",
    "Dispatcher",
    "Arguments",
    "Bind type",
    "Description",
    "Submap",
    "Config line",
];

/// A panel that displays detailed information about a selected keybinding.
///
/// The panel width is enforced by the parent Paned widget in app.rs
pub struct DetailsPanel {
    /// Root widget (Frame)
    widget: Frame,
    /// Value labels in `FIELDS` order
    values: Vec<Label>,
    /// Label displaying conflict status
    status_label: Label,
    /// Edit button
    edit_button: Button,
    /// Delete button
    delete_button: Button,
    /// Controller for accessing conflict information
    controller: Rc<Controller>,
    /// Currently displayed binding (for edit and delete)
    current_binding: Rc<RefCell<Option<Keybinding>>>,
}

impl DetailsPanel {
    /// Creates a header/value label pair for one row of the details grid
    fn create_label_row(header_text: &str) -> (Label, Label) {
        let header = Label::builder()
            .label(header_text)
            .halign(Align::End)
            .valign(Align::Start)
            .xalign(1.0)
            .build();
        header.add_css_class("field-header");

        let value = Label::builder()
            .halign(Align::Start)
            .xalign(0.0)
            .wrap(true)
            .wrap_mode(WordChar)
            .max_width_chars(28)
            .selectable(true)
            .build();
        value.add_css_class("field-value");

        (header, value)
    }

    /// Create a new details panel.
    pub fn new(controller: Rc<Controller>) -> Self {
        let frame = Frame::builder()
            .label("Selected Keybinding")
            .margin_start(10)
            .margin_end(10)
            .margin_top(10)
            .margin_bottom(10)
            .width_request(300)
            .build();

        let vbox = GtkBox::new(Orientation::Vertical, 10);
        vbox.set_margin_start(15);
        vbox.set_margin_end(15);
        vbox.set_margin_top(15);
        vbox.set_margin_bottom(15);

        let grid = Grid::builder().row_spacing(10).column_spacing(15).build();

        let mut values = Vec::with_capacity(FIELDS.len());
        for (row, field) in FIELDS.iter().enumerate() {
            let (header, value) = Self::create_label_row(field);
            grid.attach(&header, 0, row as i32, 1, 1);
            grid.attach(&value, 1, row as i32, 1, 1);
            values.push(value);
        }
        if let Some(config_line) = values.last() {
            config_line.add_css_class("config-line");
        }

        let (status_header, status_label) = Self::create_label_row("Status");
        grid.attach(&status_header, 0, FIELDS.len() as i32, 1, 1);
        grid.attach(&status_label, 1, FIELDS.len() as i32, 1, 1);

        vbox.append(&grid);

        let separator = Separator::new(Orientation::Horizontal);
        separator.set_margin_top(10);
        separator.set_margin_bottom(10);
        vbox.append(&separator);

        let edit_button = icon_button("document-edit-symbolic", "Edit");
        edit_button.set_sensitive(false);
        edit_button.set_tooltip_text(Some("Edit the selected keybinding (Enter)"));
        vbox.append(&edit_button);

        let delete_button = icon_button("user-trash-symbolic", "Delete");
        delete_button.set_sensitive(false);
        delete_button.add_css_class("destructive-action");
        delete_button.set_tooltip_text(Some("Delete the selected keybinding (Delete)"));
        vbox.append(&delete_button);

        frame.set_child(Some(&vbox));

        let panel = Self {
            widget: frame,
            values,
            status_label,
            edit_button,
            delete_button,
            controller,
            current_binding: Rc::new(RefCell::new(None)),
        };
        panel.update_binding(None);
        panel
    }

    /// Update the panel to display information about a specific keybinding.
    ///
    /// If `None` is passed, the panel shows a placeholder.
    pub fn update_binding(&self, binding: Option<&Keybinding>) {
        *self.current_binding.borrow_mut() = binding.cloned();

        let origin = binding.and_then(|b| self.controller.origin_note(b));
        self.edit_button.set_sensitive(binding.is_some());
        self.delete_button.set_sensitive(binding.is_some());

        let Some(b) = binding else {
            for (index, value) in self.values.iter().enumerate() {
                value.set_label(if index == 0 { "Select a binding" } else { "" });
                value.set_tooltip_text(None);
            }
            self.status_label.set_label("");
            self.status_label.set_tooltip_text(None);
            return;
        };

        let texts = [
            b.key_combo.to_string(),
            b.dispatcher.clone(),
            b.args.clone().unwrap_or_else(|| "(none)".to_string()),
            format!("{}{}", b.bind_type, describe_flags(&b.bind_type.flags())),
            b.description
                .clone()
                .unwrap_or_else(|| "(none)".to_string()),
            b.submap.clone().unwrap_or_else(|| "(global)".to_string()),
            match self.controller.config_format() {
                ConfigFormat::Lua => {
                    render_lua_bind(b).unwrap_or_else(|_| "(Lua code, see the file)".to_string())
                }
                ConfigFormat::Hyprlang => b.to_string(),
            },
        ];
        for (value, text) in self.values.iter().zip(texts) {
            value.set_tooltip_text(Some(&text));
            value.set_label(&text);
        }

        let others: Vec<Keybinding> = self
            .controller
            .get_conflicts()
            .into_iter()
            .filter(|c| c.submap == b.submap && c.key_combo == b.key_combo)
            .flat_map(|c| c.conflicting_bindings)
            .filter(|cb| cb != b)
            .collect();

        let read_only_note = origin
            .map(|reason| {
                format!("From Lua code: it {reason}. Edit or Delete appends an override at the end of the file.\n")
            })
            .unwrap_or_default();

        if others.is_empty() {
            self.status_label
                .set_label(&format!("{read_only_note}No conflicts"));
            self.status_label
                .set_tooltip_text(Some("This key combination is bound once"));
            return;
        }

        let lines: Vec<String> = others
            .iter()
            .map(|cb| {
                format!("{} {}", cb.dispatcher, cb.args.as_deref().unwrap_or(""))
                    .trim()
                    .to_string()
            })
            .collect();
        let shown = lines.iter().take(2).cloned().collect::<Vec<_>>().join("\n");
        let more = if lines.len() > 2 {
            format!("\n(and {} more)", lines.len() - 2)
        } else {
            String::new()
        };
        self.status_label
            .set_label(&format!("{read_only_note}Conflicts with:\n{shown}{more}"));
        self.status_label
            .set_tooltip_text(Some(&format!("Conflicts with:\n{}", lines.join("\n"))));
    }

    /// Connects the delete button to a callback
    pub fn connect_delete<F>(&self, callback: F)
    where
        F: Fn(&Keybinding) + 'static,
    {
        let current_binding = self.current_binding.clone();
        self.delete_button.connect_clicked(move |_| {
            // Clone out first so no borrow is held while the callback refreshes the UI
            let binding = current_binding.borrow().clone();
            if let Some(binding) = binding {
                callback(&binding);
            }
        });
    }

    /// Connects a callback to the edit button
    pub fn connect_edit<F>(&self, callback: F)
    where
        F: Fn(&Keybinding) + 'static,
    {
        let current_binding = self.current_binding.clone();
        self.edit_button.connect_clicked(move |_| {
            let binding = current_binding.borrow().clone();
            if let Some(binding) = binding {
                callback(&binding);
            }
        });
    }

    /// Get the root widget for adding to a container.
    pub fn widget(&self) -> &Frame {
        &self.widget
    }

    /// Acts as if the user clicked Edit (no-op when nothing is selected)
    pub fn trigger_edit(&self) {
        if self.edit_button.is_sensitive() {
            self.edit_button.emit_clicked();
        }
    }

    /// Acts as if the user clicked Delete (no-op when nothing is selected)
    pub fn trigger_delete(&self) {
        if self.delete_button.is_sensitive() {
            self.delete_button.emit_clicked();
        }
    }
}

/// Short explanation of flag letters, e.g. " (repeat while held, works on the lock screen)"
fn describe_flags(flags: &str) -> String {
    let parts: Vec<&str> = BIND_FLAGS
        .iter()
        .filter(|(c, _)| flags.contains(*c))
        .map(|(_, text)| *text)
        .collect();
    if parts.is_empty() {
        String::new()
    } else {
        format!(" ({})", parts.join(", "))
    }
}
