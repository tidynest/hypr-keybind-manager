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
//! This module contains all GTK action definitions (quit, export, import,
//! undo, redo, apply) and their setup functions

use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, CheckButton, EventControllerKey,
    FileDialog, Label, Orientation, Window, gdk,
    gio::{Cancellable, SimpleAction},
    prelude::*,
};
use std::{cell::Cell, rc::Rc};

use crate::ui::{
    Controller,
    builders::layout::MainLayout,
    components::{ConflictPanel, DetailsPanel, KeybindList},
    controller::ImportMode,
};

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

/// Sets up undo and redo, enabled only while there is something to undo or redo
pub fn setup_history_actions(
    app: &Application,
    window: &ApplicationWindow,
    controller: Rc<Controller>,
    layout: &MainLayout,
) {
    let undo_action = SimpleAction::new("undo", None);
    undo_action.set_enabled(controller.can_undo());

    let redo_action = SimpleAction::new("redo", None);
    redo_action.set_enabled(controller.can_redo());

    for (action, is_undo) in [(&undo_action, true), (&redo_action, false)] {
        let controller = controller.clone();
        let keybind_list = layout.keybind_list.clone();
        let details_panel = layout.details_panel.clone();
        let conflict_panel = layout.conflict_panel.clone();
        let window = window.clone();
        let undo_action = undo_action.clone();
        let redo_action = redo_action.clone();

        action.connect_activate(move |_, _| {
            let result = if is_undo {
                controller.undo()
            } else {
                controller.redo()
            };
            match result {
                Ok(()) => {
                    refresh_main_view(&controller, &keybind_list, &details_panel, &conflict_panel);
                    update_history_action_state(&undo_action, &redo_action, &controller);
                }
                Err(e) => show_action_error(
                    &window,
                    if is_undo {
                        "Undo Failed"
                    } else {
                        "Redo Failed"
                    },
                    &e,
                ),
            }
        });
    }

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
    let window_for_export = window.clone();

    export_action.connect_activate(move |_, _| {
        let file_dialog = FileDialog::builder()
            .title("Export Keybindings")
            .initial_name("hyprland-keybindings.conf")
            .build();

        let controller = controller.clone();
        let window = window_for_export.clone();

        let parent = window.clone();
        file_dialog.save(Some(&parent), None::<&Cancellable>, move |result| {
            let Some(path) = result.ok().and_then(|file| file.path()) else {
                return;
            };
            if let Err(e) = controller.export_to(&path) {
                show_action_error(&window, "Export Failed", &e);
            }
        });
    });

    app.add_action(&export_action);
    app.set_accels_for_action("app.export", &["<Primary>e"]);
}

/// Sets up the import action
///
/// Opens a file chooser first, then asks whether to replace or merge, and
/// refreshes the UI after the import.
pub fn setup_import_action(
    app: &Application,
    window: &ApplicationWindow,
    controller: Rc<Controller>,
    layout: &MainLayout,
) {
    let import_action = SimpleAction::new("import", None);
    let window_for_import = window.clone();
    let keybind_list = layout.keybind_list.clone();
    let details_panel = layout.details_panel.clone();
    let conflict_panel = layout.conflict_panel.clone();
    let status_banner = layout.status_banner.clone();

    import_action.connect_activate(move |_, _| {
        let file_dialog = FileDialog::builder().title("Import Keybindings").build();

        let controller = controller.clone();
        let keybind_list = keybind_list.clone();
        let details_panel = details_panel.clone();
        let conflict_panel = conflict_panel.clone();
        let status_banner = status_banner.clone();
        let window = window_for_import.clone();

        let parent = window.clone();
        file_dialog.open(Some(&parent), None::<&Cancellable>, move |result| {
            let Some(path) = result.ok().and_then(|file| file.path()) else {
                return;
            };

            let Some(mode) = show_import_mode_dialog(&window).get() else {
                return;
            };

            match controller.import_from(&path, mode) {
                Ok(()) => {
                    refresh_main_view(&controller, &keybind_list, &details_panel, &conflict_panel);
                    if let Some(app) = window.application() {
                        sync_history_actions(&app, &controller);
                    }
                    status_banner.show(&format!(
                        "Imported {} ({}). Undo reverts it.",
                        path.display(),
                        match mode {
                            ImportMode::Replace => "replaced existing bindings",
                            ImportMode::Merge => "merged with existing bindings",
                        }
                    ));
                }
                Err(e) => show_action_error(&window, "Import Failed", &e),
            }
        });
    });

    app.add_action(&import_action);
    app.set_accels_for_action("app.import", &["<Primary>o"]);
}

/// Asks whether to replace or merge; `None` when cancelled
fn show_import_mode_dialog(parent: &ApplicationWindow) -> Rc<Cell<Option<ImportMode>>> {
    let response = Rc::new(Cell::new(None));

    let dialog = Window::builder()
        .title("Import keybindings")
        .modal(true)
        .transient_for(parent)
        .default_width(400)
        .default_height(200)
        .build();

    let key_controller = EventControllerKey::new();
    let dialog_for_escape = dialog.clone();
    key_controller.connect_key_pressed(move |_, key, _, _| {
        if key == gdk::Key::Escape {
            dialog_for_escape.close();
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    dialog.add_controller(key_controller);

    let vbox = GtkBox::new(Orientation::Vertical, 12);
    vbox.set_margin_top(20);
    vbox.set_margin_bottom(20);
    vbox.set_margin_start(20);
    vbox.set_margin_end(20);

    let label = Label::new(Some("How should the imported keybindings be applied?"));
    label.set_wrap(true);
    vbox.append(&label);

    let merge_radio = CheckButton::with_label("Merge: keep existing bindings, add new ones");
    merge_radio.set_active(true);
    merge_radio.set_tooltip_text(Some(
        "Bindings whose key combination already exists are skipped",
    ));
    vbox.append(&merge_radio);

    let replace_radio = CheckButton::with_label("Replace: remove all existing bindings first");
    replace_radio.set_group(Some(&merge_radio));
    replace_radio.set_tooltip_text(Some(
        "Every current keybinding is removed, then the file is imported",
    ));
    vbox.append(&replace_radio);

    let button_box = GtkBox::new(Orientation::Horizontal, 12);
    button_box.set_halign(gtk4::Align::End);
    button_box.set_margin_top(20);

    let cancel_button = Button::with_label("Cancel");
    let dialog_for_cancel = dialog.clone();
    cancel_button.connect_clicked(move |_| dialog_for_cancel.close());
    button_box.append(&cancel_button);

    let import_button = Button::with_label("Import");
    import_button.add_css_class("suggested-action");
    let dialog_for_import = dialog.clone();
    let response_clone = response.clone();
    import_button.connect_clicked(move |_| {
        let mode = if replace_radio.is_active() {
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

    let main_context = glib::MainContext::default();
    while response.get().is_none() && dialog.is_visible() {
        main_context.iteration(true);
    }

    response
}

/// Sets up the "apply to Hyprland" action
///
/// Runs `hyprctl reload`, reports the outcome in the status banner and shows
/// a dialog when it fails.
pub fn setup_apply_action(
    app: &Application,
    window: &ApplicationWindow,
    controller: Rc<Controller>,
    layout: &MainLayout,
) {
    let apply_action = SimpleAction::new("apply-to-hyprland", None);
    let window_for_apply = window.clone();
    let status_banner = layout.status_banner.clone();

    apply_action.connect_activate(move |_, _| match controller.apply_to_hyprland() {
        Ok(()) => status_banner.show(&format!(
            "Hyprland reloaded at {}.",
            chrono::Local::now().format("%H:%M:%S")
        )),
        Err(e) => show_action_error(&window_for_apply, "Apply Failed", &e),
    });

    app.add_action(&apply_action);
    app.set_accels_for_action("app.apply-to-hyprland", &["<Primary>r"]);
}

pub fn refresh_main_view(
    controller: &Controller,
    keybind_list: &KeybindList,
    details_panel: &DetailsPanel,
    conflict_panel: &ConflictPanel,
) {
    keybind_list.update_with_bindings(controller.get_current_view());
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

pub fn show_action_error(window: &ApplicationWindow, title: &str, message: &str) {
    let error_dialog = gtk4::AlertDialog::builder()
        .modal(true)
        .message(title)
        .detail(message)
        .buttons(vec!["OK"])
        .build();
    error_dialog.show(Some(window));
}
