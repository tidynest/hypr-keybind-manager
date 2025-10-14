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
use gtk4::{gdk, Application, ApplicationWindow, CssProvider, Orientation, Paned};
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

    // Load custom CSS styling for the application
    fn load_css() {
        let provider = CssProvider::new();
        provider.load_from_string(
            "
            /* Alternating row colours for keybind list */
            .even-row {
                background-color: alpha(@theme_bg_color, 0.5);
            }

            .odd-row {
                background-color: transparent;
            }

            /* Selected row highlighting */
            row:selected {
                background-color: @theme_selected_bg_color;
            }

            /* Hover effect for better interaction feedback */
            row:hover {
                background-color: alpha(@theme_bg_color, 0.3);
            }

            /* Slightly increase spacing in details panel */
            frame {
                padding: 8px;
            }

            /* Bold labels for field headers */
            .field-header {
                font-weight: bold;
                font-size: 0.95em;
            }

            /* Value labels with slightly larger spacing */
            .field-value {
                padding-left: 8px;
            }

            /* Panel title styling */
            frame > label {
                font-weight: bold;
                font-size: 1.05em;
            }

            /* Search bar styling */
            searchentry {
                border-radius: 8px;
            }

            /* Warning banner styling (replaces deprecated InfoBar) */
            .warning-banner {
                background: linear-gradient(to bottom, #fcd34d, #fbbf24);
                border: 1px solid #f59e0b;
                border-left: 4px solid #d97706;
                border-radius: 8px;
                padding: 4px;
                box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
            }

            .warning-banner label {
                color: #78350f;
                font-weight: 600;
                text-shadow: 0 1px 1px rgba(255, 255, 255, 0.3);
            }
            "
        );

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
            .default_height(600)
            .build();

        // Create main vertical box
        let main_vbox = gtk4::Box::new(Orientation::Vertical, 0);

        // Create conflict panel at top
        let conflict_panel = ConflictPanel::new(controller.clone());
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
                    gtk4::glib::Propagation::Stop
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
                    gtk4::glib::Propagation::Stop
                }
                gdk::Key::Return | gdk::Key::KP_Enter => {
                    // Enter key - already handled by row selection, just ensure it's visible
                    if let Some(selected_row) = list_box_for_keys.selected_row() {
                        list_box_for_keys.select_row(Some(&selected_row));
                    }
                    gtk4::glib::Propagation::Stop
                }
                _ => gtk4::glib::Propagation::Proceed
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

        // Show window
        window.present();
    }
}