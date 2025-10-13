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
        self.app.run();
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

        // TODO: Add components here
        // For now, just show empty window to verify GTK4 works

        // Display conflict count in window title (temporary)
        let conflict_count = controller.conflict_count();
        if conflict_count > 0 {
            window.set_title(Some(&format!(
                "Hyprland Keybinding Manager - {} conflict{}",
                conflict_count,
                if conflict_count == 1 { "" } else { "s" }
            )));
        }

        // Show the window
        window.present();
    }
}