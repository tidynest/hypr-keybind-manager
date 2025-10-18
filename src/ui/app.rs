// Copyright 2025 bakri (tidynest@proton.me)
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

//! GTK4 Application wrapper
//!
//! This module sets up the GTK4 application lifecycle and creates
//! the main window. It uses the Controller to load and display data.
//!
//! # Architecture
//!
//! ```text
//! App (GTK4 Application)
//!   ├─ Creates Controller
//!   ├─ Builds main window
//!   └─ Connects components to Controller
//! ```

use gtk4::prelude::*;
use gtk4::{gdk, Application, ApplicationWindow, Button, CssProvider, Orientation, Paned};
use std::path::PathBuf;
use std::rc::Rc;
use crate::ui::components::{BackupDialog, ConflictPanel, DetailsPanel, EditDialog, KeybindList, SearchBar};

use crate::ui::Controller;

/// GTK4 Application for keybinding management
pub struct App {
    /// GTK4 Application instance
    app: Application,
    /// MVC Controller
    controller: Rc<Controller>,
}

impl App {
    /// Creates a new App with the given config file path
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to Hyprland configuration file
    ///
    /// # Returns
    ///
    /// * `Ok(App)` - Successfully initialised
    /// * `Err(String)` - Failed to create Controller or App
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hypr_keybind_manager::ui::App;
    /// use std::path::PathBuf;
    ///
    /// let app = App::new(
    ///     PathBuf::from("~/.config/hypr/hyprland.conf")
    /// )?;
    /// # Ok::<(), String>(())
    /// ```
    pub fn new(config_path: PathBuf) -> Result<Self, String> {
        // Create GTK4 Application
        let app = Application::builder()
            .application_id("com.tidynest.hypr-keybind-manager")
            .build();

        // Create Controller
        let controller = Controller::new(config_path)
            .map_err(|e| format!("Failed to create controller: {}", e))?;

        let controller = Rc::new(controller);

        Ok(Self { app, controller })
    }

    /// Runs the GTK4 application
    ///
    /// This starts the GTK4 main loop. Call this after creating the App.
    /// The function blocks until the application exits.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use hypr_keybind_manager::ui::App;
    /// # use std::path::PathBuf;
    /// # let app = App::new(PathBuf::from("hyprland.conf"))?;
    /// app.run();  // Blocks until window closes
    /// # Ok::<(), String>(())
    /// ```
    pub fn run(self) {
        let controller = self.controller.clone();

        // Connect activate signal (called when app starts)
        self.app.connect_activate(move |app| {
            Self::build_ui(app, controller.clone());
        });

        // Run the application (blocks until exit)
        self.app.run_with_args::<&str>(&[]);
    }

    /// Loads custom CSS styling for the application
    ///
    /// Applies the CSS from `style.css` to the default display
    /// at APPLICATION priority level.
    fn load_css() {
        let provider = CssProvider::new();
        let css = include_str!("style.css");
        provider.load_from_string(css);

        // Apply CSS to the default display
        gtk4::style_context_add_provider_for_display(
            &gdk::Display::default().expect("Could not connect to a display"),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    /// Builds the main window UI
    ///
    /// This is called when the application activates. It creates
    /// the window and all components.
    fn build_ui(app: &Application, controller: Rc<Controller>) {
        // Load keybindings
        if let Err(e) = controller.load_keybindings() {
            eprintln!("Failed to load keybindings: {}", e);
            return;
        }

        // Load custom CSS styling
        Self::load_css();

        // Create application window
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Hyprland Keybinding Manager")
            .default_width(1000)
            .default_height(800)
            .build();

        // Create main vertical box
        let main_vbox = gtk4::Box::new(Orientation::Vertical, 0);

        // Create conflict panel at top
        let conflict_panel = Rc::new(ConflictPanel::new(controller.clone()));
        main_vbox.append(conflict_panel.widget());

        // Use PANED for fixed right panel
        let paned = Paned::new(Orientation::Horizontal);

        // LEFT SIDE: Search + List (resizable)
        let left_vbox = gtk4::Box::new(Orientation::Vertical, 10);
        left_vbox.set_margin_start(10);
        left_vbox.set_margin_end(10);
        left_vbox.set_margin_bottom(10);

        // Create SINGLE keybind list instance
        let keybind_list = Rc::new(KeybindList::new(controller.clone()));

        // Create search bar
        let search_bar = SearchBar::new();
        left_vbox.append(search_bar.widget());

        let add_keybinding_button = Button::builder()
            .label("➕ Add Keybinding")
            .build();
        add_keybinding_button.add_css_class("suggested-action");
        left_vbox.append(&add_keybinding_button);

        let backup_button = Button::builder()
            .label("📦 Manage Backups")
            .build();
        left_vbox.append(&backup_button);

        // Add keybind list to left side
        left_vbox.append(keybind_list.widget());

        // Wire up search functionality manually
        let keybind_list_for_search = keybind_list.clone();
        let controller_for_search = controller.clone();

        search_bar.widget().connect_search_changed(move |entry| {
            let query = entry.text().to_string();
            eprintln!("🔍 Search: '{}'", query);
            let filtered = controller_for_search.filter_keybindings(&query);
            keybind_list_for_search.update_with_bindings(filtered);
        });

        // RIGHT SIDE: Details Panel (FIXED 280px)
        let details_panel = Rc::new(DetailsPanel::new(controller.clone()));

        // KEY: Configure Paned to keep right side fixed at 280px
        paned.set_start_child(Some(&left_vbox));
        paned.set_resize_start_child(true);   // Left side resizes with window
        paned.set_shrink_start_child(true);   // Left side can shrink

        paned.set_end_child(Some(details_panel.widget()));
        paned.set_resize_end_child(false);    // Right side DOES NOT resize!
        paned.set_shrink_end_child(false);    // Right side CANNOT shrink!

        // Set divider position (window width - panel width)
        paned.set_position(720);  // 1000px default width - 280px panel = 720px

        // Adjust paned position when window size changes
        let paned_clone = paned.clone();
        window.connect_default_width_notify(move |window| {
            let width = window.default_width();
            paned_clone.set_position(width - 280);
        });

        // Add paned to main
        main_vbox.append(&paned);

        // Set window content
        window.set_child(Some(&main_vbox));

        // Connect row selection signal
        let details_panel_clone = details_panel.clone();
        let keybind_list_clone = keybind_list.clone();

        keybind_list.list_box().connect_row_selected(move |_list_box, row| {
            match row {
                Some(r) => {
                    let index = r.index() as usize;
                    if let Some(binding) = keybind_list_clone.get_binding_at_index(index) {
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

        // Add keyboard navigation
        use gtk4::{EventControllerKey, gdk};

        let key_controller    = EventControllerKey::new();
        let list_box_for_keys = keybind_list.list_box().clone();

        key_controller.connect_key_pressed(move |_controller, key, _code, _modifier| {
            match key {
                gdk::Key::Up => {
                    // Move selection up
                    if let Some(selected_row) = list_box_for_keys.selected_row() {
                        let current_index = selected_row.index();
                        if current_index > 0 {
                            if let Some(previous_row) = list_box_for_keys.row_at_index(current_index - 1) {
                                list_box_for_keys.select_row(Some(&previous_row));
                            }
                        }
                    }
                    glib::Propagation::Stop
                }
                gdk::Key::Down => {
                    // Move selection down
                    if let Some(selected_row) = list_box_for_keys.selected_row() {
                        let current_index = selected_row.index();
                        if let Some(next_row) = list_box_for_keys.row_at_index(current_index + 1) {
                            list_box_for_keys.select_row(Some(&next_row));
                        }
                    } else {
                        // If nothing selected, select first row
                        if let Some(first_row) = list_box_for_keys.row_at_index(0) {
                            list_box_for_keys.select_row(Some(&first_row));
                        }
                    }
                    glib::Propagation::Stop
                }
                gdk::Key::Return | gdk::Key::KP_Enter => {
                    // Enter key - already handled by row selection, just ensure it's visible
                    if let Some(selected_row) = list_box_for_keys.selected_row() {
                        list_box_for_keys.select_row(Some(&selected_row));
                    }
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed
            }
        });

        keybind_list.list_box().add_controller(key_controller);

        // Make list box focusable for keyboard navigation
        keybind_list.list_box().set_can_focus(true);
        keybind_list.list_box().grab_focus();

        // Initial display
        let all_bindings = controller.get_keybindings();
        keybind_list.update_with_bindings(all_bindings);

        // Update conflict panel
        conflict_panel.refresh();

        // ============================================================================
        // Wire up delete button
        // ============================================================================
        let window_for_delete = window.clone();
        let controller_for_delete = controller.clone();
        let keybind_list_for_delete = keybind_list.clone();
        let details_panel_for_delete = details_panel.clone();
        let conflict_panel_for_delete = conflict_panel.clone();

        details_panel.connect_delete(move |binding| {
            eprintln!("🗑️  Delete button clicked for: {}", binding.key_combo);

            // Clone everything needed for the dialog
            let controller_clone = controller_for_delete.clone();
            let keybind_list_clone = keybind_list_for_delete.clone();
            let details_panel_clone = details_panel_for_delete.clone();
            let conflict_panel_clone = conflict_panel_for_delete.clone();
            let binding_clone = binding.clone();
            let window_clone = window_for_delete.clone();

            // Create confirmation dialog using modern AlertDialog
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

            // Clone window AGAIN for the inner closure (this fixes the borrow error!)
            let window_for_inner = window_clone.clone();

            dialog.choose(
                Some(&window_clone),  // Borrows window_clone here
                None::<&gtk4::gio::Cancellable>,
                move |response| {  // Moves window_for_inner here (different variable!)
                    match response {
                        Ok(1) => {  // 1 = Delete button (second button)
                            match controller_clone.delete_keybinding(&binding_clone) {
                                Ok(()) => {
                                    // Refresh UI
                                    let updated = controller_clone.get_keybindings();
                                    keybind_list_clone.update_with_bindings(updated);
                                    details_panel_clone.update_binding(None);
                                    conflict_panel_clone.refresh();
                                    eprintln!("✅ Keybinding deleted successfully");
                                }
                                Err(e) => {
                                    eprintln!("❌ Failed to delete: {}", e);

                                    // Show error dialog (use window_for_inner)
                                    let error_dialog = gtk4::AlertDialog::builder()
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
                            // User clicked Cancel
                            eprintln!("🚫 Delete cancelled");
                        }
                        Ok(_other) => {
                            // Unexpected button index
                            eprintln!("? Unexpected button index");
                        }
                        Err(_e) => {
                            // Dialog error
                            eprintln!("❌ Delete dialog error"); // ADD THIS
                        }
                    }
                }
            );
        });
        // ============================================================================
        // End of delete button wiring
        // ============================================================================

        // ============================================================================
        // Wire up edit button
        // ============================================================================
        let window_for_edit = window.clone();
        let controller_for_edit = controller.clone();
        let keybind_list_for_edit = keybind_list.clone();
        let details_panel_for_edit = details_panel.clone();
        let conflict_panel_for_edit = conflict_panel.clone();

        details_panel.connect_edit(move |binding| {
            eprintln!("✏️  Edit button clicked for: {}", binding.key_combo);

            // Clone everything for the nested closures
            let controller_clone = controller_for_edit.clone();
            let keybind_list_clone = keybind_list_for_edit.clone();
            let details_panel_clone = details_panel_for_edit.clone();
            let conflict_panel_clone = conflict_panel_for_edit.clone();
            let binding_clone = binding.clone();
            let window_clone = window_for_edit.clone();

            // Show the edit dialog
            let edit_dialog = EditDialog::new(&window_clone, &binding_clone);

            // Get the result (blocks until user clicks Save or Cancel)
            if let Some(new_binding) = edit_dialog.show_and_wait() {
                // Try to update the keybinding
                match controller_clone.update_keybinding(&binding_clone, new_binding) {
                    Ok(()) => {
                        // Clear the details panel (user needs to reselect)
                        details_panel_clone.update_binding(None);

                        // Refresh the keybinding list
                        let updated_bindings = controller_clone.get_keybindings();
                        keybind_list_clone.update_with_bindings(updated_bindings);

                        // Refresh conflicts
                        conflict_panel_clone.refresh();

                        eprintln!("✅ Keybinding updated successfully");
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to update: {}", e);

                        // Show error dialog
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
        // End of edit button wiring
        // ============================================================================

        // ============================================================================
        // Wire up add button
        // ============================================================================
        let window_for_add = window.clone();
        let controller_for_add = controller.clone();
        let keybind_list_for_add = keybind_list.clone();
        let details_panel_for_add = details_panel.clone();
        let conflict_panel_for_add = conflict_panel.clone();

        add_keybinding_button.connect_clicked(move |_| {
            eprintln!("➕ Add button clicked");

            // Clone everything for the nested closures
            let controller_clone = controller_for_add.clone();
            let keybind_list_clone = keybind_list_for_add.clone();
            let details_panel_clone = details_panel_for_add.clone();
            let conflict_panel_clone = conflict_panel_for_add.clone();
            let window_clone = window_for_add.clone();

            // Create an empty keybinding for the dialog
            use crate::core::types::{BindType, KeyCombo, Keybinding};
            let empty_binding = Keybinding {
                bind_type: BindType::Bind,  // Default to standard bind
                key_combo: KeyCombo {
                    modifiers: vec![],
                    key: String::new(),
                },
                dispatcher: String::new(),
                args: None,
            };

            // Show the edit dialog (reused for adding!)
            let edit_dialog = EditDialog::new(&window_clone, &empty_binding);

            // Get the result (blocks until user clicks Save or Cancel)
            if let Some(new_binding) = edit_dialog.show_and_wait() {
                // Try to add the keybinding
                match controller_clone.add_keybinding(new_binding) {
                    Ok(()) => {
                        // Clear the details panel (user needs to select the new binding)
                        details_panel_clone.update_binding(None);

                        // Refresh the keybinding list
                        let updated_bindings = controller_clone.get_keybindings();
                        keybind_list_clone.update_with_bindings(updated_bindings);

                        // Refresh conflicts
                        conflict_panel_clone.refresh();

                        eprintln!("✅ Keybinding added successfully");
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to add: {}", e);

                        // Show error dialog
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
        // End of add button wiring
        // ============================================================================

        // ============================================================================
        // Wire up backup button
        // ============================================================================
        let window_for_backup = window.clone();
        let controller_for_backup = controller.clone();
        let keybind_list_for_backup = keybind_list.clone();
        let details_panel_for_backup = details_panel.clone();
        let conflict_panel_for_backup = conflict_panel.clone();

        backup_button.connect_clicked(move |_| {
            eprintln!("📦 Backup manager opened");

            // Get list of backups from ConfigManager
            let backups = match controller_for_backup.list_backups() {
                Ok(b)  => b,
                Err(e) => {
                    eprintln!("❌ Failed to list backups: {}", e);
                    return;
                }
            };

            // Clone for the restore callback
            let controller_clone = controller_for_backup.clone();
            let keybind_list_clone = keybind_list_for_backup.clone();
            let details_panel_clone = details_panel_for_backup.clone();
            let conflict_panel_clone = conflict_panel_for_backup.clone();

            // Clone again for the delete callback
            let controller_for_delete = controller_for_backup.clone();

            // Create and show dialog with restore callback
            let dialog = BackupDialog::new(
                window_for_backup.upcast_ref::<gtk4::Window>(),
                backups,
                move |backup_path| {
                    // Call controller to restore backup
                    match controller_clone.restore_backup(backup_path) {
                        Ok(()) => {
                            // Refresh UI after restore
                            let updated_bindings = controller_clone.get_keybindings();
                            keybind_list_clone.update_with_bindings(updated_bindings);

                            // Clear details panel
                            details_panel_clone.update_binding(None);

                            // Refresh conflicts
                            conflict_panel_clone.refresh();

                            Ok(())
                        }
                        Err(e) => {
                            Err(e)
                        }
                    }
                },
                // Add the on_delete callback
                move |backup_path| {
                    controller_for_delete.delete_backup(backup_path)
                }
            );
            dialog.show();
        });

        // Show window
        window.present();
    }
}
