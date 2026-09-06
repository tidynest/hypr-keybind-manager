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

//! Event handler setup
//!
//! Wires up all event handlers for the main UI:
//! - Row selection and activation
//! - Keyboard navigation and shortcuts
//! - Delete/Edit/Add buttons
//! - Backup manager

use crate::{
    core::types::Keybinding,
    ui::{
        Controller,
        actions::{refresh_main_view, show_action_error, sync_history_actions},
        builders::{header::HeaderWidgets, layout::MainLayout},
        components::{BackupDialog, ConflictPanel, DetailsPanel, EditDialog, KeybindList},
    },
};
use gtk4::{
    ApplicationWindow, Button, CallbackAction, EventControllerKey, Shortcut, ShortcutController,
    ShortcutScope, ShortcutTrigger, gdk, gio, prelude::*,
};
use std::rc::Rc;

/// Wires up all event handlers for the main UI
pub fn wire_up_handlers(
    window: &ApplicationWindow,
    controller: Rc<Controller>,
    layout: &MainLayout,
    header: &HeaderWidgets,
) {
    let list_box = layout.keybind_list.list_box().clone();
    let details_panel = layout.details_panel.clone();

    // Row selection
    {
        let details_panel = details_panel.clone();
        let keybind_list = layout.keybind_list.clone();
        list_box.connect_row_selected(move |_, row| {
            let binding = row.and_then(|r| keybind_list.get_binding_at_index(r.index() as usize));
            details_panel.update_binding(binding.as_ref());
        });
    }

    // Keyboard: arrows move, Delete deletes, Enter is left to the ListBox (row-activated)
    {
        let list_box_for_keys = list_box.clone();
        let details_panel = details_panel.clone();
        let key_controller = EventControllerKey::new();
        key_controller.connect_key_pressed(move |_, key, _, _| match key {
            gdk::Key::Up | gdk::Key::Down => {
                let step = if key == gdk::Key::Up { -1 } else { 1 };
                let next = list_box_for_keys
                    .selected_row()
                    .map(|row| row.index() + step)
                    .unwrap_or(0);
                if let Some(row) = list_box_for_keys.row_at_index(next) {
                    list_box_for_keys.select_row(Some(&row));
                }
                glib::Propagation::Stop
            }
            gdk::Key::Delete => {
                details_panel.trigger_delete();
                glib::Propagation::Stop
            }
            _ => glib::Propagation::Proceed,
        });
        list_box.add_controller(key_controller);
        list_box.set_can_focus(true);
        list_box.grab_focus();
    }

    // Double-click or Enter on a row opens the editor
    {
        let details_panel = details_panel.clone();
        layout
            .keybind_list
            .connect_activate(move |_| details_panel.trigger_edit());
    }

    // Delete button
    {
        let window = window.clone();
        let controller = controller.clone();
        let refs = LayoutRefs::from(layout);
        details_panel.connect_delete(move |binding| {
            let controller = controller.clone();
            let refs = refs.clone();
            let binding = binding.clone();
            let window = window.clone();

            let dialog = gtk4::AlertDialog::builder()
                .modal(true)
                .message("Delete Keybinding?")
                .detail(format!(
                    "{} → {} {}",
                    binding.key_combo,
                    binding.dispatcher,
                    binding.args.as_deref().unwrap_or("")
                ))
                .buttons(vec!["Cancel", "Delete"])
                .cancel_button(0)
                .default_button(0)
                .build();

            let window_for_inner = window.clone();
            dialog.choose(Some(&window), None::<&gio::Cancellable>, move |response| {
                if response != Ok(1) {
                    return;
                }
                match controller.delete_keybinding(&binding) {
                    Ok(()) => refs.refresh(&window_for_inner, &controller),
                    Err(e) => show_action_error(&window_for_inner, "Delete Failed", &e),
                }
            });
        });
    }

    // Edit button
    {
        let window = window.clone();
        let controller = controller.clone();
        let refs = LayoutRefs::from(layout);
        details_panel.connect_edit(move |binding| {
            let dialog =
                EditDialog::new(&window, controller.clone(), binding, Some(binding.clone()));
            if let Some(new_binding) = dialog.show_and_wait() {
                match controller.update_keybinding(binding, new_binding) {
                    Ok(()) => refs.refresh(&window, &controller),
                    Err(e) => show_action_error(&window, "Edit Failed", &e),
                }
            }
        });
    }

    // Add button, also on Ctrl+N
    {
        let window = window.clone();
        let controller = controller.clone();
        let refs = LayoutRefs::from(layout);
        header.add_button.connect_clicked(move |_| {
            let dialog = EditDialog::new(&window, controller.clone(), &Keybinding::default(), None);
            if let Some(new_binding) = dialog.show_and_wait() {
                match controller.add_keybinding(new_binding) {
                    Ok(()) => refs.refresh(&window, &controller),
                    Err(e) => show_action_error(&window, "Add Failed", &e),
                }
            }
        });
        bind_shortcut(&header.add_button, "<Control>n", &layout.main_vbox);
    }

    // Backup manager
    {
        let window = window.clone();
        let controller = controller.clone();
        let refs = LayoutRefs::from(layout);
        header.backup_button.connect_clicked(move |_| {
            let backups = match controller.list_backups() {
                Ok(b) => b,
                Err(e) => {
                    show_action_error(&window, "Backups Unavailable", &e);
                    return;
                }
            };

            let controller_for_restore = controller.clone();
            let controller_for_delete = controller.clone();
            let refs = refs.clone();
            let window_for_restore = window.clone();

            let dialog = BackupDialog::new(
                window.upcast_ref::<gtk4::Window>(),
                backups,
                move |backup_path| {
                    controller_for_restore.restore_backup(backup_path)?;
                    refs.refresh(&window_for_restore, &controller_for_restore);
                    Ok(())
                },
                move |backup_path| controller_for_delete.delete_backup(backup_path),
            );
            dialog.show();
        });
    }
}

/// Makes `key` press `button` from anywhere in the window containing `scope_widget`
fn bind_shortcut(button: &Button, key: &str, scope_widget: &impl IsA<gtk4::Widget>) {
    let controller = ShortcutController::new();
    controller.set_scope(ShortcutScope::Global);
    let button = button.clone();
    let action = CallbackAction::new(move |_, _| {
        button.emit_clicked();
        glib::Propagation::Stop
    });
    controller.add_shortcut(Shortcut::new(
        ShortcutTrigger::parse_string(key),
        Some(action),
    ));
    scope_widget.add_controller(controller);
}

/// The components a change has to refresh, cheap to clone into callbacks
#[derive(Clone)]
struct LayoutRefs {
    keybind_list: Rc<KeybindList>,
    details_panel: Rc<DetailsPanel>,
    conflict_panel: Rc<ConflictPanel>,
}

impl From<&MainLayout> for LayoutRefs {
    fn from(layout: &MainLayout) -> Self {
        Self {
            keybind_list: layout.keybind_list.clone(),
            details_panel: layout.details_panel.clone(),
            conflict_panel: layout.conflict_panel.clone(),
        }
    }
}

impl LayoutRefs {
    fn refresh(&self, window: &ApplicationWindow, controller: &Controller) {
        refresh_main_view(
            controller,
            &self.keybind_list,
            &self.details_panel,
            &self.conflict_panel,
        );
        if let Some(app) = window.application() {
            sync_history_actions(&app, controller);
        }
    }
}
