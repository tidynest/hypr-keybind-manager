//! Event handler setup
//!
//! Wires up all event handlers for the main UI:
//! - Row selection
//! - Keyboard navigation
//! - Delete/Edit/Add buttons
//! - Backup manager

use gtk4::{prelude::*, gio, gdk, ApplicationWindow, Button, EventControllerKey};
use std::rc::Rc;
use crate::core::types::{BindType, KeyCombo, Keybinding};
use crate::ui::components::{BackupDialog, ConflictPanel, DetailsPanel, EditDialog, KeybindList};
use crate::ui::Controller;

/// Wires up all event handlers for the main UI
///
/// Sets up:
/// - Row selection in keybind list
/// - Keyboard navigation (Up/Down/Enter)
/// - Delete button click handler
/// - Edit button click handler
/// - Add button click handler
/// - Backup button click handler
pub fn wire_up_handlers(
    window:         &ApplicationWindow,
    controller:     Rc<Controller>,
    keybind_list:   Rc<KeybindList>,
    details_panel:  Rc<DetailsPanel>,
    conflict_panel: Rc<ConflictPanel>,
    add_button:     &Button,
    backup_button:  &Button,
) {
    // ============================================================================
    // Row selection handler
    // ============================================================================
    let details_panel_clone = details_panel.clone();
    let keybind_list_clone = keybind_list.clone();

    keybind_list.list_box().connect_row_selected(move |_list_box,
                                                       row| {
        match row {
            Some(r) => {
                let index = r.index() as usize;
                if let Some(binding) =
                    keybind_list_clone.get_binding_at_index(index) {
                    eprintln!("👆 Selected: {}", binding.key_combo);

                    details_panel_clone.update_binding(Some(&binding));
                }
            }
            None => {
                eprintln!("👆 Selection cleared");
                details_panel_clone.update_binding(None);
            }
        }
    });

    // ============================================================================
    // Keyboard navigation
    // ============================================================================
    let key_controller = EventControllerKey::new();
    let list_box_for_keys = keybind_list.list_box().clone();

    key_controller.connect_key_pressed(move |_controller, key, _code, _modifier| {
        match key {
            gdk::Key::Up => {
                if let Some(selected_row) =
                    list_box_for_keys.selected_row() {
                    let current_index = selected_row.index();
                    if current_index > 0 {
                        if let Some(previous_row) =
                            list_box_for_keys.row_at_index(current_index - 1) {
                            list_box_for_keys.select_row(Some(&previous_row));
                        }
                    }
                }
                glib::Propagation::Stop
            }
            gdk::Key::Down => {
                if let Some(selected_row) =
                    list_box_for_keys.selected_row() {
                    let current_index = selected_row.index();
                    if let Some(next_row) =
                        list_box_for_keys.row_at_index(current_index + 1) {
                        list_box_for_keys.select_row(Some(&next_row));
                    }
                } else if let Some(first_row) =
                        list_box_for_keys.row_at_index(0) {
                        list_box_for_keys.select_row(Some(&first_row));
                }
                glib::Propagation::Stop
            }
            gdk::Key::Return | gdk::Key::KP_Enter => {
                if let Some(selected_row) =
                    list_box_for_keys.selected_row() {
                    list_box_for_keys.select_row(Some(&selected_row));
                }
                glib::Propagation::Stop
            }
            _ => glib::Propagation::Proceed
        }
    });

    keybind_list.list_box().add_controller(key_controller);
    keybind_list.list_box().set_can_focus(true);
    keybind_list.list_box().grab_focus();

    // ============================================================================
    // Delete button handler
    // ============================================================================
    let window_for_delete = window.clone();
    let controller_for_delete = controller.clone();
    let keybind_list_for_delete = keybind_list.clone();
    let details_panel_for_delete = details_panel.clone();
    let conflict_panel_for_delete = conflict_panel.clone();

    details_panel.connect_delete(move |binding| {
        eprintln!("🗑️  Delete button clicked for: {}", binding.key_combo);

        let controller_clone = controller_for_delete.clone();
        let keybind_list_clone = keybind_list_for_delete.clone();
        let details_panel_clone = details_panel_for_delete.clone();
        let conflict_panel_clone = conflict_panel_for_delete.clone();
        let binding_clone = binding.clone();
        let window_clone = window_for_delete.clone();

        let dialog = gtk4::AlertDialog::builder()
            .modal(true)
            .message("Delete Keybinding?")
            .detail(format!(
                "Are you sure you want to delete:\n\n{} → {} {}",
                binding.key_combo,
                binding.dispatcher,
                binding.args.as_deref().unwrap_or("(no args)")
            ))
            .buttons(vec!["Cancel", "Delete"])
            .cancel_button(0)
            .default_button(0)
            .build();

        let window_for_inner = window_clone.clone();

        dialog.choose(
            Some(&window_clone),
            None::<&gio::Cancellable>,
            move |response| {
                match response {
                    Ok(1) => {
                        match
                        controller_clone.delete_keybinding(&binding_clone) {
                            Ok(()) => {
                                let updated = controller_clone.get_keybindings();
                                keybind_list_clone.update_with_bindings(updated);
                                details_panel_clone.update_binding(None);
                                conflict_panel_clone.refresh();
                                eprintln!("✅ Keybinding deleted successfully");
                            }
                            Err(e) => {
                                eprintln!("❌ Failed to delete: {}", e);

                                let error_dialog =
                                    gtk4::AlertDialog::builder()
                                        .modal(true)
                                        .message("Delete Failed")
                                        .detail(format!("Failed to delete keybinding:\n{}", e))
                                        .buttons(vec!["OK"])
                                        .build();
                                error_dialog.show(Some(&window_for_inner));
                            }
                        }
                    }
                    Ok(0) => {
                        eprintln!("🚫 Delete cancelled");
                    }
                    Ok(_other) => {
                        eprintln!("? Unexpected button index");
                    }
                    Err(_e) => {
                        eprintln!("❌ Delete dialog error");
                    }
                }
            }
        );
    });

    // ============================================================================
    // Edit button handler
    // ============================================================================
    let window_for_edit = window.clone();
    let controller_for_edit = controller.clone();
    let keybind_list_for_edit = keybind_list.clone();
    let details_panel_for_edit = details_panel.clone();
    let conflict_panel_for_edit = conflict_panel.clone();

    details_panel.connect_edit(move |binding| {
        eprintln!("✏️  Edit button clicked for: {}", binding.key_combo);

        let controller_clone = controller_for_edit.clone();
        let keybind_list_clone = keybind_list_for_edit.clone();
        let details_panel_clone = details_panel_for_edit.clone();
        let conflict_panel_clone = conflict_panel_for_edit.clone();
        let binding_clone = binding.clone();
        let window_clone = window_for_edit.clone();
        let edit_dialog = EditDialog::new(&window_clone, &binding_clone);

        if let Some(new_binding) = edit_dialog.show_and_wait() {
            match controller_clone.update_keybinding(&binding_clone, new_binding) {
                Ok(()) => {
                    details_panel_clone.update_binding(None);
                    let updated_bindings = controller_clone.get_keybindings();
                    keybind_list_clone.update_with_bindings(updated_bindings);
                    conflict_panel_clone.refresh();
                    eprintln!("✅ Keybinding updated successfully");
                }
                Err(e) => {
                    eprintln!("❌ Failed to update: {}", e);

                    let error_dialog = gtk4::AlertDialog::builder()
                        .modal(true)
                        .message("Edit Failed")
                        .detail(format!("Failed to update keybinding:\n\n{}", e))
                        .buttons(vec!["OK"])
                        .build();
                    error_dialog.show(Some(&window_clone));
                }
            }
        } else {
            eprintln!("🚫 Edit cancelled");
        }
    });

    // ============================================================================
    // Add button handler
    // ============================================================================
    let window_for_add = window.clone();
    let controller_for_add = controller.clone();
    let keybind_list_for_add = keybind_list.clone();
    let details_panel_for_add = details_panel.clone();
    let conflict_panel_for_add = conflict_panel.clone();

    add_button.connect_clicked(move |_| {
        eprintln!("➕ Add button clicked");

        let controller_clone = controller_for_add.clone();
        let keybind_list_clone = keybind_list_for_add.clone();
        let details_panel_clone = details_panel_for_add.clone();
        let conflict_panel_clone = conflict_panel_for_add.clone();
        let window_clone = window_for_add.clone();

        let empty_binding = Keybinding {
            bind_type: BindType::Bind,
            key_combo: KeyCombo {
                modifiers: vec![],
                key: String::new(),
            },
            dispatcher: String::new(),
            args: None,
        };

        let edit_dialog = EditDialog::new(&window_clone,
                                          &empty_binding);

        if let Some(new_binding) = edit_dialog.show_and_wait() {
            match controller_clone.add_keybinding(new_binding) {
                Ok(()) => {
                    details_panel_clone.update_binding(None);
                    let updated_bindings = controller_clone.get_keybindings();
                    keybind_list_clone.update_with_bindings(updated_bindings);
                    conflict_panel_clone.refresh();
                    eprintln!("✅ Keybinding added successfully");
                }
                Err(e) => {
                    eprintln!("❌ Failed to add: {}", e);
                    let error_dialog = gtk4::AlertDialog::builder()
                        .modal(true)
                        .message("Add Failed")
                        .detail(format!("Failed to add keybinding:\n\n{}", e))
                        .buttons(vec!["OK"])
                        .build();
                    error_dialog.show(Some(&window_clone));
                }
            }
        } else {
            eprintln!("🚫 Add cancelled");
        }
    });

    // ============================================================================
    // Backup button handler
    // ============================================================================
    let window_for_backup = window.clone();
    let controller_for_backup = controller.clone();
    let keybind_list_for_backup = keybind_list.clone();
    let details_panel_for_backup = details_panel.clone();
    let conflict_panel_for_backup = conflict_panel.clone();

    backup_button.connect_clicked(move |_| {
        eprintln!("📦 Backup manager opened");

        let backups = match controller_for_backup.list_backups() {
            Ok(b) => b,
            Err(e) => {
                eprintln!("❌ Failed to list backups: {}", e);
                return;
            }
        };

        let controller_clone = controller_for_backup.clone();
        let keybind_list_clone = keybind_list_for_backup.clone();
        let details_panel_clone = details_panel_for_backup.clone();
        let conflict_panel_clone =
            conflict_panel_for_backup.clone();

        let controller_for_delete = controller_for_backup.clone();

        let dialog = BackupDialog::new(
            window_for_backup.upcast_ref::<gtk4::Window>(),
            backups,
            move |backup_path| {
                match controller_clone.restore_backup(backup_path) {
                    Ok(()) => {
                        let updated_bindings =
                            controller_clone.get_keybindings();

                        keybind_list_clone.update_with_bindings(updated_bindings);
                        details_panel_clone.update_binding(None);
                        conflict_panel_clone.refresh();
                        Ok(())
                    }
                    Err(e) => Err(e)
                }
            },
            move |backup_path| {
                controller_for_delete.delete_backup(backup_path)
            }
        );
        dialog.show();
    });

}