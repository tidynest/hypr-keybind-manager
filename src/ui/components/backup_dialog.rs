use gtk4::prelude::*;
use gtk4::{Button, ListBox, Orientation, ScrolledWindow, Window};
use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

pub struct BackupDialog {
    window: Window,
    backups: Vec<PathBuf>,
    selected_backup: Rc<Cell<Option<usize>>>,  // Index of selected backup
    list_box: ListBox,
}

impl BackupDialog {
    pub fn new(parent: &Window, backups: Vec<PathBuf>) -> Self {
        let bd_window = Window::builder()
            .title("Backups")
            .modal(true)
            .transient_for(parent)
            .default_width(450)
            .default_height(300)
            .build();

        // After window.build();

        // Initialise selection state
        let selected_backup = Rc::new(Cell::new(None));

        // TODO: Build UI layers here

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

            let parts_by_underscore: Vec<&str> = timestamp.split('_').collect();    // Replace the underscore with a space and add colons
            if parts_by_underscore.len() == 2 {
                let date_part = parts_by_underscore[0];  // "2025-10-15"
                let time_part = parts_by_underscore[1];  // "143023"
                // Slice time_part
                if time_part.len() == 6 {
                    let hour   = &time_part[0..2];  // "14"
                    let minute = &time_part[2..4];  // "30"
                    let second = &time_part[4..6];  // "25"

                    // Combine them
                    display_text = format!("{} {}:{}:{}", date_part, hour, minute, second);
                    // Result "2025-10-15 14:30:25"
                }
            }

            // Create a label for this backup (use display_text instead of filename
            let label = gtk4::Label::new(Some(&display_text));
            label.set_halign(gtk4::Align::Start);  // Left-align text
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

        let window_for_close = bd_window.clone();
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

    pub fn show(&self) {
        self.window.present();
        self.list_box.unselect_all();
    }
}
