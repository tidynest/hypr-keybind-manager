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
//! - a key combination entry with a "record" button that fills it from a key press
//! - inline key-combo availability feedback and suggested free alternatives
//! - a searchable dispatcher list, limited to what the validator accepts
//! - description (saved as `bindd`), submap and bind type fields
//! - modal save/cancel flow with validation

use crate::{
    core::{
        sandbox,
        types::{BIND_FLAGS, BindType, KeyCombo, Keybinding, Modifier},
        validator::ALLOWED_DISPATCHERS,
    },
    ui::{Controller, builders::header::icon_button, controller::KeyComboAvailability},
};
use gtk4::{
    AlertDialog, ApplicationWindow, Box as GtkBox, Button, DropDown, Entry, EventControllerKey,
    Grid, Label, Orientation, PropagationPhase, StringList, StringObject, Switch, ToggleButton,
    Window, gdk, prelude::*,
};
use std::{cell::Cell, rc::Rc};

/// Bind variants offered by the bind type dropdown, in display order.
const BIND_TYPES: [BindType; 6] = [
    BindType::Bind,
    BindType::BindE,
    BindType::BindL,
    BindType::BindM,
    BindType::BindR,
    BindType::BindEL,
];

/// Human-readable labels for `BIND_TYPES`, same order.
const BIND_TYPE_LABELS: [&str; 6] = [
    "bind (standard)",
    "binde (repeat while held)",
    "bindl (works on lock screen)",
    "bindm (mouse binding)",
    "bindr (trigger on release)",
    "bindel (repeat + lock screen)",
];

/// Dialog for adding or editing a keybinding
pub struct EditDialog {
    dialog_window: Window,
    key_entry: Entry,
    submap_entry: Entry,
    dispatcher_dropdown: DropDown,
    args_entry: Entry,
    description_entry: Entry,
    bind_type_dropdown: DropDown,
    /// Flags outside the six presets, shown as an extra last dropdown item
    custom_bind_type: Option<BindType>,
    sandbox_switch: Switch,
    response: Rc<Cell<Option<DialogResponse>>>,
}

#[derive(Clone, Debug, Copy, PartialEq)]
enum DialogResponse {
    Save,
    Cancel,
}

impl EditDialog {
    /// Creates a new dialog pre-filled with the binding's current values.
    ///
    /// `original_binding` is `Some` when editing, so the availability check
    /// does not report the binding as conflicting with itself.
    pub fn new(
        parent: &ApplicationWindow,
        controller: Rc<Controller>,
        binding: &Keybinding,
        original_binding: Option<Keybinding>,
    ) -> Self {
        let title = if original_binding.is_some() {
            "Edit Keybinding"
        } else {
            "Add Keybinding"
        };

        let dialog_window = Window::builder()
            .title(title)
            .modal(true)
            .transient_for(parent)
            .default_width(540)
            .resizable(false)
            .build();

        let grid = Grid::builder()
            .row_spacing(12)
            .column_spacing(12)
            .margin_start(20)
            .margin_end(20)
            .margin_top(20)
            .margin_bottom(12)
            .build();

        let mut row = 0;

        // Key combination with a record button
        let key_entry = Entry::builder()
            .text(binding.key_combo.to_string())
            .placeholder_text("e.g. SUPER+SHIFT+M")
            .hexpand(true)
            .tooltip_text("Modifiers and key joined with +, e.g. SUPER+SHIFT+M")
            .build();
        let record_button = ToggleButton::builder()
            .icon_name("media-record-symbolic")
            .tooltip_text(
                "Record: press the key combination to fill the field. Combinations Hyprland \
                 already binds are intercepted by the compositor and cannot be recorded.",
            )
            .build();
        let key_row = GtkBox::new(Orientation::Horizontal, 6);
        key_row.append(&key_entry);
        key_row.append(&record_button);
        attach_row(&grid, &mut row, "Key combination", &key_row);

        let availability_label = Label::builder()
            .label("Enter a key combination to check availability.")
            .halign(gtk4::Align::Start)
            .xalign(0.0)
            .wrap(true)
            .build();
        availability_label.add_css_class("availability-hint");
        grid.attach(&availability_label, 1, row, 1, 1);
        row += 1;

        let suggestion_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .hexpand(true)
            .visible(false)
            .build();
        suggestion_box.add_css_class("suggestion-row");
        grid.attach(&suggestion_box, 1, row, 1, 1);
        row += 1;

        let submap_entry = Entry::builder()
            .text(binding.submap.as_deref().unwrap_or(""))
            .placeholder_text("Empty for the global map")
            .hexpand(true)
            .tooltip_text("Name of the submap this binding belongs to")
            .build();
        attach_row(&grid, &mut row, "Submap", &submap_entry);

        // Dispatcher: searchable list of what the validator accepts
        let mut dispatchers: Vec<&str> = ALLOWED_DISPATCHERS.to_vec();
        if !binding.dispatcher.is_empty() && !dispatchers.contains(&binding.dispatcher.as_str()) {
            dispatchers.push(binding.dispatcher.as_str());
        }
        let dispatcher_dropdown = DropDown::builder()
            .model(&StringList::new(&dispatchers))
            .enable_search(true)
            .hexpand(true)
            .tooltip_text("The Hyprland dispatcher to run; type to search")
            .build();
        let selected = dispatchers
            .iter()
            .position(|d| *d == binding.dispatcher)
            .unwrap_or(0);
        dispatcher_dropdown.set_selected(selected as u32);
        attach_row(&grid, &mut row, "Dispatcher", &dispatcher_dropdown);

        let visible_args = binding
            .args
            .as_deref()
            .and_then(sandbox::unwrap_command)
            .or_else(|| binding.args.clone())
            .unwrap_or_default();
        let args_entry = Entry::builder()
            .text(visible_args)
            .placeholder_text("Optional arguments")
            .hexpand(true)
            .tooltip_text("Optional dispatcher arguments")
            .build();
        attach_row(&grid, &mut row, "Arguments", &args_entry);

        let description_entry = Entry::builder()
            .text(binding.description.as_deref().unwrap_or(""))
            .placeholder_text("Optional, saved as a bindd line")
            .hexpand(true)
            .tooltip_text("Shown in the list and in tools that read bindd descriptions")
            .build();
        attach_row(&grid, &mut row, "Description", &description_entry);

        // Bind type presets, plus the binding's own flags when they are not a preset
        let preset_type = binding.bind_type.with_description(false);
        let mut labels: Vec<String> = BIND_TYPE_LABELS.iter().map(|s| s.to_string()).collect();
        let custom_bind_type = (!BIND_TYPES.contains(&preset_type)).then_some(preset_type);
        if let Some(custom) = custom_bind_type {
            labels.push(format!("{custom} (as in config)"));
        }
        let label_refs: Vec<&str> = labels.iter().map(String::as_str).collect();
        let bind_type_dropdown = DropDown::builder()
            .model(&StringList::new(&label_refs))
            .hexpand(true)
            .tooltip_text(flag_tooltip())
            .build();
        let selected = BIND_TYPES
            .iter()
            .position(|t| *t == preset_type)
            .unwrap_or(BIND_TYPES.len());
        bind_type_dropdown.set_selected(selected as u32);
        attach_row(&grid, &mut row, "Bind type", &bind_type_dropdown);

        let sandbox_switch = Switch::builder().halign(gtk4::Align::Start).build();
        let sandbox_active = binding.args.as_deref().is_some_and(sandbox::is_wrapped)
            && binding.dispatcher == "exec";
        sandbox_switch.set_active(sandbox_active);
        let sandbox_label = attach_row(&grid, &mut row, "Bubblewrap sandbox", &sandbox_switch);

        let button_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .halign(gtk4::Align::End)
            .margin_start(20)
            .margin_end(20)
            .margin_bottom(20)
            .build();

        let cancel_button = Button::with_label("Cancel");
        let save_button = icon_button("document-save-symbolic", "Save");
        save_button.add_css_class("suggested-action");
        save_button.set_receives_default(true);
        button_box.append(&cancel_button);
        button_box.append(&save_button);

        let main_box = GtkBox::new(Orientation::Vertical, 0);
        main_box.append(&grid);
        main_box.append(&button_box);
        dialog_window.set_child(Some(&main_box));
        dialog_window.set_default_widget(Some(&save_button));

        let response: Rc<Cell<Option<DialogResponse>>> = Rc::new(Cell::new(None));

        {
            let response = response.clone();
            let window = dialog_window.clone();
            cancel_button.connect_clicked(move |_| {
                response.set(Some(DialogResponse::Cancel));
                window.close();
            });
        }
        {
            let response = response.clone();
            save_button.connect_clicked(move |_| response.set(Some(DialogResponse::Save)));
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

        // Escape closes, unless a recording is in progress (then it just stops it)
        {
            let key_controller = EventControllerKey::new();
            let window = dialog_window.clone();
            key_controller.connect_key_pressed(move |_, key, _, _| {
                if key == gdk::Key::Escape {
                    window.close();
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            });
            dialog_window.add_controller(key_controller);
        }
        connect_key_recording(&dialog_window, &record_button, &key_entry);

        // Availability feedback follows the key and submap entries
        {
            let refresh = {
                let controller = controller.clone();
                let key_entry = key_entry.clone();
                let submap_entry = submap_entry.clone();
                let availability_label = availability_label.clone();
                let suggestion_box = suggestion_box.clone();
                move || {
                    refresh_key_combo_feedback_widgets(
                        &controller,
                        original_binding.as_ref(),
                        &key_entry,
                        &submap_entry,
                        &availability_label,
                        &suggestion_box,
                    )
                }
            };
            let refresh_for_key = refresh.clone();
            key_entry.connect_changed(move |_| refresh_for_key());
            let refresh_for_submap = refresh.clone();
            submap_entry.connect_changed(move |_| refresh_for_submap());
            refresh();
        }

        // The sandbox switch only applies to exec
        {
            let refresh = {
                let dropdown = dispatcher_dropdown.clone();
                let switch = sandbox_switch.clone();
                move || {
                    refresh_sandbox_controls_widgets(
                        &dropdown_text(&dropdown),
                        &switch,
                        &sandbox_label,
                    )
                }
            };
            let refresh_for_dropdown = refresh.clone();
            dispatcher_dropdown.connect_selected_notify(move |_| refresh_for_dropdown());
            refresh();
        }

        Self {
            dialog_window,
            key_entry,
            submap_entry,
            dispatcher_dropdown,
            args_entry,
            description_entry,
            bind_type_dropdown,
            custom_bind_type,
            sandbox_switch,
            response,
        }
    }

    /// Parses the form fields and returns a new Keybinding if valid.
    fn parse_binding(&self) -> Result<Keybinding, String> {
        let key_combo = parse_key_combo_text(&self.key_entry.text())?
            .ok_or_else(|| "Key combination cannot be empty".to_string())?;

        let dispatcher = dropdown_text(&self.dispatcher_dropdown);
        if dispatcher.is_empty() {
            return Err("Choose a dispatcher".to_string());
        }

        let submap = optional_text(&self.submap_entry);
        let description = optional_text(&self.description_entry);

        let args = match optional_text(&self.args_entry) {
            Some(args) if self.sandbox_switch.is_active() && dispatcher == "exec" => {
                Some(sandbox::wrap_command(&args)?)
            }
            other => other,
        };

        let bind_type = BIND_TYPES
            .get(self.bind_type_dropdown.selected() as usize)
            .copied()
            .or(self.custom_bind_type)
            .unwrap_or(BindType::Bind)
            .with_description(description.is_some());

        Ok(Keybinding {
            bind_type,
            key_combo,
            dispatcher,
            args,
            description,
            submap,
        })
    }

    /// Shows the dialog and waits for user response.
    pub fn show_and_wait(self) -> Option<Keybinding> {
        self.response.set(None);
        self.dialog_window.present();
        self.key_entry.select_region(0, 0);

        let main_context = glib::MainContext::default();
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
                        AlertDialog::builder()
                            .modal(true)
                            .message("Invalid Input")
                            .detail(e)
                            .buttons(vec!["OK"])
                            .build()
                            .show(Some(&self.dialog_window));
                        self.response.set(None);
                    }
                },
                _ => {
                    self.dialog_window.close();
                    return None;
                }
            }
        }
    }
}

/// Adds a labelled widget as the next grid row and returns the label
fn attach_row(grid: &Grid, row: &mut i32, text: &str, widget: &impl IsA<gtk4::Widget>) -> Label {
    let label = Label::builder()
        .label(text)
        .halign(gtk4::Align::End)
        .build();
    grid.attach(&label, 0, *row, 1, 1);
    grid.attach(widget, 1, *row, 1, 1);
    *row += 1;
    label
}

/// Trimmed entry text, `None` when empty
fn optional_text(entry: &Entry) -> Option<String> {
    let text = entry.text();
    let trimmed = text.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

/// The selected string of a `DropDown` backed by a `StringList`
fn dropdown_text(dropdown: &DropDown) -> String {
    dropdown
        .selected_item()
        .and_downcast::<StringObject>()
        .map(|item| item.string().to_string())
        .unwrap_or_default()
}

/// One line per flag letter, for the bind type tooltip
fn flag_tooltip() -> String {
    let mut lines = vec!["Flag letters after \"bind\":".to_string()];
    lines.extend(BIND_FLAGS.iter().map(|(c, text)| format!("{c}: {text}")));
    lines.join("\n")
}

/// Fills `key_entry` from the next key press while `record` is active
///
/// Runs in the capture phase so the key never reaches the focused entry.
/// Pure modifier presses are ignored, Escape stops recording.
fn connect_key_recording(window: &Window, record: &ToggleButton, key_entry: &Entry) {
    let controller = EventControllerKey::new();
    controller.set_propagation_phase(PropagationPhase::Capture);

    let record = record.clone();
    let key_entry = key_entry.clone();
    controller.connect_key_pressed(move |_, key, _, state| {
        if !record.is_active() {
            return glib::Propagation::Proceed;
        }
        if key == gdk::Key::Escape {
            record.set_active(false);
            return glib::Propagation::Stop;
        }
        let Some(name) = key.name() else {
            return glib::Propagation::Stop;
        };
        if is_modifier_keysym(&name) {
            return glib::Propagation::Stop;
        }

        let masks = [
            (
                gdk::ModifierType::SUPER_MASK
                    | gdk::ModifierType::META_MASK
                    | gdk::ModifierType::HYPER_MASK,
                "SUPER",
            ),
            (gdk::ModifierType::CONTROL_MASK, "CTRL"),
            (gdk::ModifierType::ALT_MASK, "ALT"),
            (gdk::ModifierType::SHIFT_MASK, "SHIFT"),
        ];
        let mut parts: Vec<String> = masks
            .iter()
            .filter(|(mask, _)| state.intersects(*mask))
            .map(|(_, label)| label.to_string())
            .collect();
        parts.push(if name.chars().count() == 1 {
            name.to_uppercase()
        } else {
            name.to_string()
        });

        key_entry.set_text(&parts.join("+"));
        record.set_active(false);
        glib::Propagation::Stop
    });

    window.add_controller(controller);
}

/// Whether a keysym name is a modifier or lock key on its own
fn is_modifier_keysym(name: &str) -> bool {
    [
        "Shift",
        "Control",
        "Alt",
        "Super",
        "Meta",
        "Hyper",
        "ISO_Level",
        "Caps_Lock",
        "Num_Lock",
        "Mode_switch",
    ]
    .iter()
    .any(|prefix| name.starts_with(prefix))
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
            "SUPER" | "MOD4" | "WIN" => Modifier::Super,
            "SHIFT" => Modifier::Shift,
            "CTRL" | "CONTROL" => Modifier::Ctrl,
            "ALT" | "MOD1" => Modifier::Alt,
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
    submap_entry: &Entry,
    availability_label: &Label,
    suggestion_box: &GtkBox,
) {
    clear_suggestion_box(suggestion_box);

    let submap = optional_text(submap_entry);
    match parse_key_combo_text(&key_entry.text()) {
        Ok(None) => set_feedback_state(
            availability_label,
            "Enter a key combination to check availability.",
            "availability-hint",
        ),
        Err(message) => set_feedback_state(availability_label, &message, "availability-warning"),
        Ok(Some(key_combo)) => {
            let assistance = controller.get_key_combo_assistance(
                Some(&key_combo),
                submap.as_deref(),
                original_binding,
            );
            match assistance.availability {
                KeyComboAvailability::Incomplete => set_feedback_state(
                    availability_label,
                    "Enter a key combination to check availability.",
                    "availability-hint",
                ),
                KeyComboAvailability::Available => set_feedback_state(
                    availability_label,
                    "This key combination is currently free.",
                    "availability-available",
                ),
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

                    for suggestion in assistance.suggestions {
                        let suggestion_text = suggestion.to_string();
                        let button = Button::builder().label(&suggestion_text).build();
                        button.add_css_class("suggestion-button");
                        let key_entry = key_entry.clone();
                        button.connect_clicked(move |_| key_entry.set_text(&suggestion_text));
                        suggestion_box.append(&button);
                    }
                    suggestion_box.set_visible(suggestion_box.first_child().is_some());
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
    dispatcher: &str,
    sandbox_switch: &Switch,
    sandbox_label: &Label,
) {
    let enabled = dispatcher.eq_ignore_ascii_case("exec");
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
