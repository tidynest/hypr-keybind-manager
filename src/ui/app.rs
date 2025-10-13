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
use gtk4::{Application, ApplicationWindow};
use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;
use super::components::{ConflictPanel, KeybindList, SearchBar};

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
        // Load keybindings from config
        match controller.load_keybindings() {
            Ok(count) => println!("✓ Loaded {} keybindings", count),
            Err(e) => {
                eprintln!("✗ Failed to load keybindings: {}", e);
                return;
            }
        }

        // Create main window
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Hyprland Keybinding Manager")
            .default_width(800)
            .default_height(600)
            .build();

        // Main vertical container
        let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        window.set_child(Some(&vbox));

        // Conflict warning panel (at the top)
        let conflict_panel = ConflictPanel::new(controller.clone());
        vbox.append(conflict_panel.widget());

        // Keybinding list (needs RefCell for SearchBar to update it)
        let keybind_list = Rc::new(RefCell::new(
            KeybindList::new(controller.clone())
        ));

        // Search bar (connects to list for filtering)
        let search_bar = SearchBar::new(
            keybind_list.clone(),
            controller.clone(),
        );
        vbox.append(search_bar.widget());

        // Add the scrollable list
        vbox.append(keybind_list.borrow().widget());

        // Load initial data into list
        keybind_list.borrow().refresh();

        // Update conflict panel
        conflict_panel.refresh();

        // Show the window
        window.present();
    }
}