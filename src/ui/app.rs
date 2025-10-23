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
use gtk4::{gdk, Application, ApplicationWindow, CssProvider, Paned};
use std::path::PathBuf;
use std::rc::Rc;
use crate::ui::actions;
use crate::ui::builders;
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

        // Setup quit action
        actions::setup_quit_action(app);

        Self::load_css();

        // Create header bar with menu
        let header_bar = builders::build_header_bar();

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Hyprland Keybinding Manager")
            .default_width(1000)
            .default_height(800)
            .titlebar(&header_bar)
            .build();

        // Setup export action
        actions::setup_export_action(app, &window, controller.clone());

        // Build main layout
        let (
            main_vbox,
            keybind_list,
            details_panel,
            conflict_panel,
            add_keybinding_button,
            backup_button,
        ) = builders::build_main_layout(controller.clone());

        // Adjust paned position when window size changes
        let paned = main_vbox.last_child().unwrap().downcast::<Paned>().unwrap();
        let paned_clone = paned.clone();
        window.connect_default_width_notify(move |window| {
            let width = window.default_width();
            paned_clone.set_position(width - 280);
        });

        // Set window content
        window.set_child(Some(&main_vbox));

        // Setup import action (needs widgets to refresh UI after import)
        actions::setup_import_action(
            app,
            &window,
            controller.clone(),
            keybind_list.clone(),
            details_panel.clone(),
            conflict_panel.clone(),
        );

        // Wire up all event handlers
        builders::wire_up_handlers(
            &window,
            controller.clone(),
            keybind_list.clone(),
            details_panel.clone(),
            conflict_panel.clone(),
            &add_keybinding_button,
            &backup_button,
        );

        // Initial display
        let all_bindings = controller.get_keybindings();
        keybind_list.update_with_bindings(all_bindings);

        // Update conflict panel
        conflict_panel.refresh();

        // Show window
        window.present();
    }
}
