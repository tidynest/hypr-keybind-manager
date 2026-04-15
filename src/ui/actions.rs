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

//! GTK Action setup for the application
//!
//! This module contains all GTK action definitions (quit, export, import)
//! and their setup functions

use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, CheckButton, EventControllerKey,
    FileDialog, Label, Orientation, Window, gdk,
    gio::{Cancellable, SimpleAction},
    prelude::*,
};
use std::{cell::Cell, rc::Rc};

use crate::ui::{Controller, controller::ImportMode};

/// Sets up the quit action
///
/// Creates a GTK action that quits the application when triggered.
pub fn setup_quit_action(app: &Application) {
    let quit_action = SimpleAction::new("quit", None);
    let app_for_quit = app.clone();

    quit_action.connect_activate(move |_, _| {
        app_for_quit.quit();
    });

    app.add_action(&quit_action);
    app.set_accels_for_action("app.quit", &["<Primary>q"]);
}

pub fn setup_history_actions(
    app: &Application,
    window: &ApplicationWindow,
    controller: Rc<Controller>,
    keybind_list: Rc<crate::ui::components::KeybindList>,
    details_panel: Rc<crate::ui::components::DetailsPanel>,
    conflict_panel: Rc<crate::ui::components::ConflictPanel>,
) {
    let undo_action = SimpleAction::new("undo", None);
    undo_action.set_enabled(controller.can_undo());

    let redo_action = SimpleAction::new("redo", None);
    redo_action.set_enabled(controller.can_redo());

    let controller_for_undo = controller.clone();
    let keybind_list_for_undo = keybind_list.clone();
    let details_panel_for_undo = details_panel.clone();
    let conflict_panel_for_undo = conflict_panel.clone();
    let window_for_undo = window.clone();
    let redo_action_for_undo = redo_action.clone();
    let undo_action_for_undo = undo_action.clone();

    undo_action.connect_activate(move |_, _| match controller_for_undo.undo() {
        Ok(()) => {
            refresh_main_view(
                &controller_for_undo,
                &keybind_list_for_undo,
                &details_panel_for_undo,
                &conflict_panel_for_undo,
            );
            update_history_action_state(
                &undo_action_for_undo,
                &redo_action_for_undo,
                &controller_for_undo,
            );
        }
        Err(e) => show_action_error(&window_for_undo, "Undo Failed", &e),
    });

    let controller_for_redo = controller.clone();
    let keybind_list_for_redo = keybind_list.clone();
    let details_panel_for_redo = details_panel.clone();
    let conflict_panel_for_redo = conflict_panel.clone();
    let window_for_redo = window.clone();
    let redo_action_for_redo = redo_action.clone();
    let undo_action_for_redo = undo_action.clone();

    redo_action.connect_activate(move |_, _| match controller_for_redo.redo() {
        Ok(()) => {
            refresh_main_view(
                &controller_for_redo,
                &keybind_list_for_redo,
                &details_panel_for_redo,
                &conflict_panel_for_redo,
            );
            update_history_action_state(
                &undo_action_for_redo,
                &redo_action_for_redo,
                &controller_for_redo,
            );
        }
        Err(e) => show_action_error(&window_for_redo, "Redo Failed", &e),
    });

    app.add_action(&undo_action);
    app.add_action(&redo_action);
    app.set_accels_for_action("app.undo", &["<Primary>z"]);
    app.set_accels_for_action("app.redo", &["<Primary><Shift>z", "<Primary>y"]);
}

pub fn sync_history_actions(app: &Application, controller: &Controller) {
    let undo = app
        .lookup_action("undo")
        .and_then(|action| action.downcast::<SimpleAction>().ok());
    let redo = app
        .lookup_action("redo")
        .and_then(|action| action.downcast::<SimpleAction>().ok());

    if let (Some(undo), Some(redo)) = (undo, redo) {
        update_history_action_state(&undo, &redo, controller);
    }
}

/// Sets up the export action
///
/// Creates a GTK action that opens a file save dialog and exports
/// keybindings to the selected file.
pub fn setup_export_action(
    app: &Application,
    window: &ApplicationWindow,
    controller: Rc<Controller>,
) {
    let export_action = SimpleAction::new("export", None);
    let controller_for_export = controller.clone();
    let window_for_export = window.clone();

    export_action.connect_activate(move |_, _| {
        eprintln!("💾 Export clicked");

        let file_dialog = FileDialog::builder()
            .title("Export Keybindings")
            .initial_name("hyprland-keybindings.conf")
            .build();

        let controller_clone = controller_for_export.clone();
        let window_clone = window_for_export.clone();

        file_dialog.save(
            Some(&window_clone),
            None::<&Cancellable>,
            move |result| match result {
                Ok(file) => {
                    let path = file.path().unwrap();
                    eprintln!("💾 Exporting to: {:?}", path);

                    match controller_clone.export_to(&path) {
                        Ok(()) => eprintln!("✅ Export successful!"),
                        Err(e) => eprintln!("❌ Export failed: {}", e),
                    }
                }
                Err(_) => eprintln!("🚫 Export cancelled"),
            },
        );
    });

    app.add_action(&export_action);
    app.set_accels_for_action("app.export", &["<Primary>e"]);
}

/// Sets up the import action
///
/// Creates a GTK action that opens a file open dialog and imports
/// keybindings from the selected file. Refreshes the UI after import.
pub fn setup_import_action(
    app: &Application,
    window: &ApplicationWindow,
    controller: Rc<Controller>,
    keybind_list: Rc<crate::ui::components::KeybindList>,
    details_panel: Rc<crate::ui::components::DetailsPanel>,
    conflict_panel: Rc<crate::ui::components::ConflictPanel>,
) {
    let import_action = SimpleAction::new("import", None);
    let controller_for_import = controller.clone();
    let window_for_import = window.clone();
    let keybind_list_for_import = keybind_list.clone();
    let details_panel_for_import = details_panel.clone();
    let conflict_panel_for_import = conflict_panel.clone();

    import_action.connect_activate(move |_, _| {
        eprintln!("📥 Import clicked");

        // Step 1: Show mode selection dialog
        let mode_choice = show_import_mode_dialog(&window_for_import);

        let chosen_mode = match mode_choice.get() {
            Some(mode) => mode,
            None => {
                eprintln!("🚫 Import cancelled (no mode selected)");
                return;
            }
        };

        eprintln!("📋 Import mode: {:?}", chosen_mode);

        // Step 2: Show file picker
        let file_dialog = FileDialog::builder().title("Import Keybindings").build();

        let controller_clone = controller_for_import.clone();
        let keybind_list_clone = keybind_list_for_import.clone();
        let details_panel_clone = details_panel_for_import.clone();
        let conflict_panel_clone = conflict_panel_for_import.clone();
        let window_clone = window_for_import.clone();
        let window_for_state_sync = window_for_import.clone();

        file_dialog.open(
            Some(&window_clone),
            None::<&Cancellable>,
            move |result| match result {
                Ok(file) => {
                    let path = file.path().unwrap();
                    eprintln!("📥 Importing from: {:?}", path);

                    match controller_clone.import_from(&path, chosen_mode) {
                        Ok(()) => {
                            eprintln!("✅ Import successful!");
                            refresh_main_view(
                                &controller_clone,
                                &keybind_list_clone,
                                &details_panel_clone,
                                &conflict_panel_clone,
                            );
                            if let Some(app) = window_for_state_sync.application() {
                                sync_history_actions(&app, &controller_clone);
                            }
                        }
                        Err(e) => eprintln!("❌ Import failed: {}", e),
                    }
                }
                Err(_) => eprintln!("🚫 Import cancelled"),
            },
        );
    });

    app.add_action(&import_action);
    app.set_accels_for_action("app.import", &["<Primary>o"]);

    /// Shows a dialog asking user to choose import mode
    ///
    /// Returns the chosen ImportMode wrapped in Rc<std::cell::Cell<Option<ImportMode>>>
    /// so it can be shared across GTK callbacks
    fn show_import_mode_dialog(parent: &ApplicationWindow) -> Rc<Cell<Option<ImportMode>>> {
        let response = Rc::new(Cell::new(None));

        // Create dialog window
        let dialog = Window::builder()
            .title("Import keybindings")
            .modal(true)
            .transient_for(parent)
            .default_width(400)
            .default_height(200)
            .build();

        let key_controller = EventControllerKey::new();
        let dialog_for_escape = dialog.clone();
        let response_for_escape = response.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if key == gdk::Key::Escape {
                response_for_escape.set(None);
                dialog_for_escape.close();
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        dialog.add_controller(key_controller);

        // Main container
        let vbox = GtkBox::new(Orientation::Vertical, 12);
        vbox.set_margin_top(20);
        vbox.set_margin_bottom(20);
        vbox.set_margin_start(20);
        vbox.set_margin_end(20);

        // Instruction label
        let label = Label::new(Some("How would you like to import keybindings?"));
        label.set_wrap(true);
        vbox.append(&label);

        // Radio button: Replace
        let replace_radio = CheckButton::with_label("Replace - Delete all existing bindings");
        replace_radio.set_tooltip_text(Some(
            "Replace all current keybindings with the imported file",
        ));
        vbox.append(&replace_radio);

        // Radio button: Merge
        let merge_radio =
            CheckButton::with_label("Merge - Keep existing, add imported (skip duplicates)");
        merge_radio.set_group(Some(&replace_radio));
        merge_radio.set_tooltip_text(Some(
            "Keep existing keybindings and only add new ones from the import",
        ));
        vbox.append(&merge_radio);

        // Button container
        let button_box = GtkBox::new(Orientation::Horizontal, 12);
        button_box.set_halign(gtk4::Align::End);
        button_box.set_margin_top(20);

        // Cancel button
        let cancel_button = Button::with_label("Cancel");
        let dialog_for_cancel = dialog.clone();
        cancel_button.connect_clicked(move |_| {
            dialog_for_cancel.close();
        });
        button_box.append(&cancel_button);

        // Import button
        let import_button = Button::with_label("Continue");
        import_button.add_css_class("suggested-action");
        import_button.set_receives_default(true);
        let dialog_for_import = dialog.clone();
        let response_clone = response.clone();
        let replace_clone = replace_radio.clone();
        import_button.connect_clicked(move |_| {
            let mode = if replace_clone.is_active() {
                ImportMode::Replace
            } else {
                ImportMode::Merge
            };
            response_clone.set(Some(mode));
            dialog_for_import.close();
        });
        button_box.append(&import_button);
        vbox.append(&button_box);
        dialog.set_child(Some(&vbox));
        dialog.set_default_widget(Some(&import_button));
        dialog.present();

        // Run modal loop
        let main_context = glib::MainContext::default();
        while response.get().is_none() && dialog.is_visible() {
            main_context.iteration(true);
        }

        response
    }
}

/// Sets up the "apply to Hyprland action"
///
/// Creates a GTK action that triggers Hyprland to reload its configuration,
/// applying all pending changes immediately without restart.
pub fn setup_apply_action(app: &Application, controller: Rc<Controller>) {
    let apply_action = SimpleAction::new("apply-to-hyprland", None);
    let controller_for_apply = controller.clone();

    apply_action.connect_activate(move |_, _| {
        eprintln!("🔄 Applying changes to Hyprland...");

        match controller_for_apply.apply_to_hyprland() {
            Ok(()) => {
                eprintln!("✅ Hyprland reloaded successfully!");
                // TODO: Show success notification in UI
            }
            Err(e) => {
                eprintln!("❌ Failed to reload Hyprland: {}", e);
                // TODO: Show error dialog
            }
        }
    });

    app.add_action(&apply_action);
    app.set_accels_for_action("app.apply-to-hyprland", &["<Primary>r"]);
}

pub fn refresh_main_view(
    controller: &Controller,
    keybind_list: &crate::ui::components::KeybindList,
    details_panel: &crate::ui::components::DetailsPanel,
    conflict_panel: &crate::ui::components::ConflictPanel,
) {
    let updated_bindings = controller.get_current_view();
    keybind_list.update_with_bindings(updated_bindings);
    details_panel.update_binding(None);
    conflict_panel.refresh();
}

fn update_history_action_state(
    undo_action: &SimpleAction,
    redo_action: &SimpleAction,
    controller: &Controller,
) {
    undo_action.set_enabled(controller.can_undo());
    redo_action.set_enabled(controller.can_redo());
}

fn show_action_error(window: &ApplicationWindow, title: &str, message: &str) {
    let error_dialog = gtk4::AlertDialog::builder()
        .modal(true)
        .message(title)
        .detail(message)
        .buttons(vec!["OK"])
        .build();
    error_dialog.show(Some(window));
}
