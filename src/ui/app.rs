// Copyright 2025 Eric Jingryd (tidynest@proton.me)
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
//!   ├─ Creates Controller when the app activates
//!   ├─ Asks for a config file if the given one does not exist
//!   ├─ Builds main window
//!   └─ Connects components to Controller
//! ```

use gtk4::{
    AlertDialog, Application, ApplicationWindow, CssProvider, FileDialog, Window, gdk,
    gio::Cancellable, prelude::*,
};
use std::{path::PathBuf, rc::Rc};

use crate::{
    config::ConfigError,
    ui::{Controller, actions, builders, file_watcher::FileWatcher},
};

/// GTK4 Application for keybinding management
pub struct App {
    /// GTK4 Application instance
    app: Application,
    /// Config file to open once the application activates
    config_path: PathBuf,
}

impl App {
    /// Creates a new App with the given config file path
    ///
    /// The file is opened when the application activates, so a missing
    /// file leads to a file chooser instead of a failure here.
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
        let app = Application::builder()
            .application_id("com.tidynest.hypr-keybind-manager")
            .build();

        Ok(Self { app, config_path })
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
        let config_path = self.config_path.clone();
        self.app.connect_activate(move |app| {
            Self::open_config(app, config_path.clone());
        });

        self.app.run_with_args::<&str>(&[]);
    }

    /// Creates the Controller for `config_path` and shows the main window
    ///
    /// A missing file opens a chooser; any other error is shown and the
    /// application quits.
    fn open_config(app: &Application, config_path: PathBuf) {
        match Controller::new(config_path) {
            Ok(controller) => Self::build_ui(app, Rc::new(controller)),
            Err(ConfigError::NotFound(missing)) => Self::prompt_for_config(app, missing),
            Err(e) => Self::fail_and_quit(app, "Cannot open config file", &e.to_string()),
        }
    }

    /// Shows `detail` in a dialog and quits once it is dismissed
    fn fail_and_quit(app: &Application, message: &str, detail: &str) {
        let hold = app.hold();
        let app = app.clone();
        AlertDialog::builder()
            .modal(true)
            .message(message)
            .detail(detail)
            .buttons(vec!["Quit"])
            .build()
            .choose(None::<&Window>, None::<&Cancellable>, move |_| {
                drop(hold);
                app.quit();
            });
    }

    /// Explains that `missing` does not exist and offers a file chooser
    fn prompt_for_config(app: &Application, missing: PathBuf) {
        // No window exists yet, so keep the application alive until one does
        let hold = app.hold();
        let app = app.clone();

        let dialog = AlertDialog::builder()
            .modal(true)
            .message("Config file not found")
            .detail(format!(
                "{} does not exist.\n\nChoose the Hyprland config file to manage, or start the \
                 program with -c <path>.",
                missing.display()
            ))
            .buttons(vec!["Choose file…", "Quit"])
            .cancel_button(1)
            .default_button(0)
            .build();

        dialog.choose(None::<&Window>, None::<&Cancellable>, move |response| {
            if response != Ok(0) {
                drop(hold);
                app.quit();
                return;
            }
            let file_dialog = FileDialog::builder()
                .title("Choose Hyprland config file")
                .modal(true)
                .build();
            let app_for_pick = app.clone();
            file_dialog.open(None::<&Window>, None::<&Cancellable>, move |result| {
                match result.ok().and_then(|file| file.path()) {
                    Some(path) => Self::open_config(&app_for_pick, path),
                    None => app_for_pick.quit(),
                }
                drop(hold);
            });
        });
    }

    /// Loads custom CSS styling for the application
    ///
    /// Applies the CSS from `style.css` to the default display
    /// at APPLICATION priority level.
    fn load_css() {
        let provider = CssProvider::new();
        let css = include_str!("style.css");
        provider.load_from_string(css);

        if let Some(display) = gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }

    /// Builds the main window UI
    ///
    /// This is called when the application activates. It creates
    /// the window and all components.
    fn build_ui(app: &Application, controller: Rc<Controller>) {
        if let Err(e) = controller.load_keybindings() {
            Self::fail_and_quit(app, "Cannot read keybindings", &e.to_string());
            return;
        }

        let file_watcher = FileWatcher::new(controller.config_path())
            .map_err(|e| eprintln!("⚠️  File watcher setup failed: {}", e))
            .ok()
            .map(Rc::new);

        actions::setup_quit_action(app);
        Self::load_css();

        let header = builders::build_header_bar();

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Hyprland Keybinding Manager")
            .default_width(1000)
            .default_height(800)
            .titlebar(&header.header_bar)
            .build();

        actions::setup_export_action(app, &window, controller.clone());

        let layout = builders::build_main_layout(controller.clone());
        Self::setup_paned_constraints(&window, &layout.paned);
        window.set_child(Some(&layout.main_vbox));

        layout.conflict_panel.connect_resolve_button(
            window.upcast_ref(),
            layout.conflict_panel.clone(),
            layout.keybind_list.clone(),
        );

        actions::setup_import_action(app, &window, controller.clone(), &layout);
        actions::setup_history_actions(app, &window, controller.clone(), &layout);
        actions::setup_apply_action(app, &window, controller.clone(), &layout);

        builders::wire_up_handlers(&window, controller.clone(), &layout, &header);

        layout
            .keybind_list
            .update_with_bindings(controller.get_current_view());
        actions::sync_history_actions(app, &controller);
        layout.conflict_panel.refresh();

        if let Some(file_watcher) = file_watcher {
            let app = app.clone();
            let controller = controller.clone();
            let keybind_list = layout.keybind_list.clone();
            let details_panel = layout.details_panel.clone();
            let conflict_panel = layout.conflict_panel.clone();
            let status_banner = layout.status_banner.clone();

            glib::timeout_add_local(std::time::Duration::from_secs(1), move || {
                if file_watcher.check_for_changes() {
                    match controller.load_keybindings() {
                        Ok(_) => {
                            controller.clear_history();
                            actions::refresh_main_view(
                                &controller,
                                &keybind_list,
                                &details_panel,
                                &conflict_panel,
                            );
                            actions::sync_history_actions(&app, &controller);
                            status_banner.show(
                                "The config file changed on disk and was reloaded. Undo history was cleared.",
                            );
                        }
                        Err(e) => status_banner.show(&format!(
                            "The config file changed on disk but could not be reloaded: {e}"
                        )),
                    }
                }
                glib::ControlFlow::Continue
            });
        }

        window.present();
    }

    fn setup_paned_constraints(window: &ApplicationWindow, paned: &gtk4::Paned) {
        let window_for_tick = window.clone();
        let paned_for_tick = paned.clone();

        paned.add_tick_callback(move |_, _| {
            let width = window_for_tick.width();
            if width > 0 {
                let current_position = paned_for_tick.position();
                let clamped = builders::layout::clamp_paned_position(width, current_position);
                if clamped != current_position {
                    paned_for_tick.set_position(clamped);
                }
            }

            glib::ControlFlow::Continue
        });

        let paned_for_startup = paned.clone();
        let window_for_startup = window.clone();
        glib::idle_add_local_once(move || {
            let width = window_for_startup
                .width()
                .max(window_for_startup.default_width());
            let startup_position = builders::layout::clamp_paned_position(width, width);
            paned_for_startup.set_position(startup_position);
        });
    }
}
