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
use gtk4::{Application, ApplicationWindow, Orientation, Paned};
use std::path::PathBuf;
use std::rc::Rc;
use crate::ui::components::{ConflictPanel, DetailsPanel, KeybindList, SearchBar};

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

        // Create application window
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Hyprland Keybinding Manager")
            .default_width(1000)
            .default_height(600)
            .build();

        // Create main vertical box
        let main_vbox = gtk4::Box::new(Orientation::Vertical, 0);

        // Create conflict panel at top
        let conflict_panel = ConflictPanel::new(controller.clone());
        main_vbox.append(conflict_panel.widget());

        // USE PANED INSTEAD OF BOX FOR FIXED RIGHT PANEL
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

        // Add keybind list to left side
        left_vbox.append(keybind_list.widget());

        // Wire up search functionality manually
        let keybind_list_for_search = keybind_list.clone();
        let controller_for_search = controller.clone();

        search_bar.widget().connect_search_changed(move |entry| {
            let query = entry.text().to_string();
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
        // This will be adjusted when window is realized
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
                        details_panel_clone.update_binding(Some(&binding));
                    }
                }
                None => {
                    details_panel_clone.update_binding(None);
                }
            }
        });

        // Initial display
        let all_bindings = controller.get_keybindings();
        keybind_list.update_with_bindings(all_bindings);

        // Update conflict panel
        conflict_panel.refresh();

        // Show window
        window.present();
    }
}