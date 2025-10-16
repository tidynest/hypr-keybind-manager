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

use gtk4::prelude::*;
use gtk4::{Button, ListBox, Orientation, ScrolledWindow, Window};
use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// Dialog for managing configuration file backups.
///
/// Displays a list of timestamped backups sorted newest to oldest, with options to:
/// - **Restore:** Replace current config with selected backup
/// - **Delete:** Remove old backups (not yet implemented)
/// - **Close:** Dismiss dialog
///
/// Backups are shown in human-readable format (e.g., "2025-10-15 14:30:25")
/// instead of the raw filename format.
pub struct BackupDialog {
    window: Window,
    backups: Vec<PathBuf>,
    selected_backup: Rc<Cell<Option<usize>>>,  // Index of selected backup
    list_box: ListBox,
}

impl BackupDialog {
    /// Formats a backup filename for display.
    ///
    /// Converts timestamps from `hyprland.conf.2025-10-15_143025` format
    /// to human-readable `2025-10-15 14:30:25`.
    ///
    /// If the filename doesn't match the expected pattern, returns the
    /// filename as-is for safe fallback.
    ///
    /// # Arguments
    ///
    /// * `backup_path` - Path to the backup file
    ///
    /// # Returns
    ///
    /// Formatted display string (e.g., "2025-10-15 14:30:25")
    fn format_backup_display(backup_path: &Path) -> String {
        // Extract filename from path
        let filename = backup_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown backup");

        // Timestamp parsing and reformatting/-styling
        let parts: Vec<&str> = filename.split('.').collect();                   // parts = ["hyprland", "conf", "2025-10-15_143025"}
        let timestamp = parts.last().unwrap_or(&"");                            // timestamp = "2025-10-15_143025"

        // Start with the filename as fallback
        let mut display_text = filename.to_string();

        let parts_by_underscore: Vec<&str> = timestamp.split('_').collect();
        if parts_by_underscore.len() == 2 {
            let date_part = parts_by_underscore[0];
            let time_part = parts_by_underscore[1];
            if time_part.len() == 6 {
                let hour = &time_part[0..2];
                let minute = &time_part[2..4];
                let second = &time_part[4..6];
                display_text = format!("{} {}:{}:{}", date_part, hour, minute, second);
            }
        }

        display_text
    }

    /// Creates a new backup management dialog.
    ///
    /// # Arguments
    ///
    /// * `parent` - Parent window for modal behaviour
    /// * `backups` - List of backup file paths (typically from `ConfigManager::list_backups()`)
    /// * `on_restore` - Callback invoked when user clicks Restore (receives backup path)
    ///
    /// # Returns
    ///
    /// A new `BackupDialog` instance ready to be shown with `.show()`
    pub fn new<F, G>(parent: &Window, backups: Vec<PathBuf>, on_restore: F, on_delete: G) -> Self
    where
        F: Fn(&Path) -> Result<(), String> + 'static,
        G: Fn(&Path) -> Result<(), String> + 'static,
    {
        let bd_window = Window::builder()
            .title("Backups")
            .modal(true)
            .transient_for(parent)
            .default_width(450)
            .default_height(300)
            .build();

        // Initialise selection state
        let selected_backup = Rc::new(Cell::new(None));

        // Create main vertical box
        let main_vbox = gtk4::Box::new(Orientation::Vertical, 12);
        main_vbox.set_margin_start(12);
        main_vbox.set_margin_end(12);
        main_vbox.set_margin_top(12);
        main_vbox.set_margin_bottom(12);

        // Create scrolled window for the list
        let scrolled_window = ScrolledWindow::builder()
            .vexpand(true)  //Expands vertically to fill space
            .build();

        // Create list box for backups
        let list_box = ListBox::new();

        // Populate list with backups
        for backup_path in &backups {
            let display_text = Self::format_backup_display(backup_path);

            let label = gtk4::Label::new(Some(&display_text));
            label.set_halign(gtk4::Align::Start);
            label.set_margin_start(8);
            label.set_margin_end(8);
            label.set_margin_top(8);
            label.set_margin_bottom(8);

            list_box.append(&label);
        }

        scrolled_window.set_child(Some(&list_box));

        main_vbox.append(&scrolled_window);

        // Create button row
        let button_box = gtk4::Box::new(Orientation::Horizontal, 12);
        button_box.set_halign(gtk4::Align::End);  // Push buttons to the right

        let restore_button = Button::builder()
            .label("Restore")
            .sensitive(false)  // Disabled until something is selected
            .build();
        restore_button.add_css_class("suggested-action");  // Blue/primary colour

        let delete_button = Button::builder()
            .label("Delete")
            .sensitive(false)
            .build();
        delete_button.add_css_class("destructive-action");  // Red colour

        let close_button = Button::builder()
            .label("Close")
            .build();

        button_box.append(&restore_button);
        button_box.append(&delete_button);
        button_box.append(&close_button);

        main_vbox.append(&button_box);

        // ===== SELECTION CALLBACK START =====
        // Wire up selection callback to enable/disable buttons
        let selected_backup_clone = selected_backup.clone();
        let restore_clone = restore_button.clone();
        let delete_clone = delete_button.clone();

        list_box.connect_row_selected(move |_list, row| {
            match row {
                Some(r) => {
                    let row_index = r.index() as usize;
                    selected_backup_clone.set(Some(row_index));
                    restore_clone.set_sensitive(true);
                    delete_clone.set_sensitive(true);
                }
                None => {
                    selected_backup_clone.set(None);
                    restore_clone.set_sensitive(false);
                    delete_clone.set_sensitive(false);
                }
            }
        });
        // ===== END OF SELECTION CALLBACK =====

        // Deselect all rows initially (user must explicitly choose)
        list_box.unselect_all();

        bd_window.set_child(Some(&main_vbox));

        // ===== RESTORE BUTTON CALLBACK =====
        let backups_for_restore = backups.clone();
        let selected_for_restore = selected_backup.clone();
        let window_for_restore = bd_window.clone();

        restore_button.connect_clicked(move |_| {
            // Get selected backup index
            if let Some(index) = selected_for_restore.get() {
                if let Some(backup_path) = backups_for_restore.get(index) {
                    // Call the restore callback
                    match on_restore(backup_path) {
                        Ok(()) => {
                            // Show success dialog
                            let success_dialog = gtk4::AlertDialog::builder()
                                .modal(true)
                                .message("Restore Successful")
                                .detail("Configuration restored successfully from backup.")
                                .buttons(vec!["OK"])
                                .build();

                            success_dialog.show(Some(&window_for_restore));
                        }
                        Err(e) => {
                            // Show error dialog
                            let error_dialog = gtk4::AlertDialog::builder()
                                .modal(true)
                                .message("Restore Failed")
                                .detail(&format!("Failed to restore backup:\n\n{}", e))
                                .buttons(vec!["OK"])
                                .build();

                            error_dialog.show(Some(&window_for_restore));
                        }
                    }
                }
            }
        });

        let window_for_close = bd_window.clone();

        // ===== DELETE BUTTON CALLBACK =====
        let backup_for_delete = backups.clone();
        let selected_for_delete = selected_backup.clone();
        let window_for_delete = bd_window.clone();
        let list_for_delete = list_box.clone();

        delete_button.connect_clicked(move |_| {
            // Get selected backup index
            if let Some(index) = selected_for_delete.get() {
                if let Some(backup_path) = backup_for_delete.get(index) {
                    // Call the delete callback
                    match on_delete(backup_path) {
                        Ok(()) => {
                            // Show success dialog
                            let success_dialog = gtk4::AlertDialog::builder()
                                .modal(true)
                                .message("Delete Successful")
                                .detail("Backup deleted successfully.")
                                .buttons(vec!["OK"])
                                .build();

                            success_dialog.show(Some(&window_for_delete));

                            // Remove the selected item from the list
                            if let Some(row) = list_for_delete.row_at_index(index as i32) {
                                list_for_delete.remove(&row);
                            }
                        }
                        Err(e) => {
                            // Show error dialog
                            let error_dialog = gtk4::AlertDialog::builder()
                                .modal(true)
                                .message("Delete Failed")
                                .detail(&format!("Failed to delete backup:\n\n{}", e))
                                .buttons(vec!["OK"])
                                .build();

                            error_dialog.show(Some(&window_for_delete));
                        }
                    }
                }
            }
        });

        close_button.connect_clicked(move |_| {
            window_for_close.close()
        });

        Self {
            window: bd_window,
            backups,
            selected_backup,
            list_box,
        }
    }

    /// Displays the backup dialog.
    ///
    /// Presents the dialog as a modal window and deselects any previously
    /// selected backup to ensure the user makes an explicit choice.
    pub fn show(&self) {
        self.window.present();
        self.list_box.unselect_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_format_backup_display_valid_timestamp() {
        let path = PathBuf::from("/backups/hyprland.conf.2025-10-15_143025");
        let result = BackupDialog::format_backup_display(&path);
        assert_eq!(result, "2025-10-15 14:30:25");
    }

    #[test]
    fn test_format_backup_display_different_timestamp() {
        let path = PathBuf::from("/backups/hyprland.conf.2024-12-31_235959");
        let result = BackupDialog::format_backup_display(&path);
        assert_eq!(result, "2024-12-31 23:59:59");
    }

    #[test]
    fn test_format_backup_display_invalid_format() {
        // Invalid: wrong number of parts
        let path = PathBuf::from("/backups/hyprland.conf");
        let result = BackupDialog::format_backup_display(&path);
        assert_eq!(result, "hyprland.conf", "Should fall back to filename");
    }

    #[test]
    fn test_format_backup_display_malformed_timestamp() {
        // Valid structure but malformed timestamp (7 digits instead of 6)
        let path = PathBuf::from("/backups/hyprland.conf.2025-10-15_1430255");
        let result = BackupDialog::format_backup_display(&path);
        // Should fall back to original filename since time_part.len() != 6
        assert_eq!(result, "hyprland.conf.2025-10-15_1430255");
    }

    #[test]
    fn test_format_backup_display_no_underscore() {
        // Missing underscore in timestamp
        let path = PathBuf::from("/backups/hyprland.conf.20251015143025");
        let result = BackupDialog::format_backup_display(&path);
        // Should fall back to original filename since split('_') won't give 2 parts
        assert_eq!(result, "hyprland.conf.20251015143025");
    }

    #[test]
    fn test_format_backup_display_unknown_backup() {
        // Path with no filename
        let path = PathBuf::from("/");
        let result = BackupDialog::format_backup_display(&path);
        assert_eq!(result, "Unknown backup");
    }
}
