//! GTK Action setup for the application
//!
//! This module contains all GTK action definitions (quit, export, import)
//! and their setup functions

use gtk4::{gio, prelude::*, Application, ApplicationWindow, FileDialog};
use std::rc::Rc;
use crate::ui::Controller;

/// Sets up the quit action
///
/// Creates a GTK action that quits the application when triggered.
pub fn setup_quit_action(app: &Application) {
    let quit_action = gio::SimpleAction::new("quit", None);
    let app_for_quit = app.clone();

    quit_action.connect_activate(move |_, _| {
        app_for_quit.quit();
    });

    app.add_action(&quit_action);
}

/// Sets up the export action
///
/// Creates a GTK action that opens a file save dialog and exports
/// keybindings to the selected file.
pub fn setup_export_action(
    app: &Application,
    window: &ApplicationWindow,
    controller: Rc<Controller>
) {
    let export_action = gio::SimpleAction::new("export", None);
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

        file_dialog.save(Some(&window_clone), None::<&gio::Cancellable>, move |result| {
            match result {
                Ok(file) => {
                    let path = file.path().unwrap();
                    eprintln!("💾 Exporting to: {:?}", path);

                    match controller_clone.export_to(&path) {
                        Ok(()) => eprintln!("✅ Export successful!"),
                        Err(e) => eprintln!("❌ Export failed: {}", e),
                    }
                }
                Err(_) => eprintln!("🚫 Export cancelled"),
            }
        });
    });

    app.add_action(&export_action);
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
    let import_action = gio::SimpleAction::new("import", None);
    let controller_for_import = controller.clone();
    let window_for_import = window.clone();
    let keybind_list_for_import = keybind_list.clone();
    let details_panel_for_import = details_panel.clone();
    let conflict_panel_for_import = conflict_panel.clone();

    import_action.connect_activate(move |_, _| {
        eprintln!("📥 Import clicked");

        let file_dialog = FileDialog::builder()
            .title("Import Keybindings")
            .build();

        let controller_clone = controller_for_import.clone();
        let keybind_list_clone = keybind_list_for_import.clone();
        let details_panel_clone = details_panel_for_import.clone();
        let conflict_panel_clone = conflict_panel_for_import.clone();
        let window_clone = window_for_import.clone();

        file_dialog.open(Some(&window_clone), None::<&gio::Cancellable>, move |result| {
            match result {
                Ok(file) => {
                    let path = file.path().unwrap();
                    eprintln!("📥 Importing from: {:?}", path);

                    match controller_clone.import_from(&path) {
                        Ok(()) => {
                            eprintln!("✅ Import successful!");
                            let updated_bindings = controller_clone.get_keybindings();
                            keybind_list_clone.update_with_bindings(updated_bindings);
                            details_panel_clone.update_binding(None);
                            conflict_panel_clone.refresh();
                        }
                        Err(e) => eprintln!("❌ Import failed: {}", e),
                    }
                }
                Err(_) => eprintln!("🚫 Import cancelled"),
            }
        });
    });

    app.add_action(&import_action);
}