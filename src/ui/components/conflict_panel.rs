//! Conflict warning panel component
//!
//! Displays a warning banner at the top of the window when keybinding conflicts
//! are detected. The panel smoothly animates in/out based on conflict state.
//!
//! # Features
//!
//! - Yellow warning banner using GTK4's GtkBox widget
//! - Displays count of detected conflicts
//! - Automatically shows/hides based on conflict state
//! - Smooth reveal/hide animations
//!
//! # Layout
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │ ⚠️  Warning: 2 keybinding conflicts detected        │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```rust,no_run
//! use hypr_keybind_manager::ui::{Controller, components::ConflictPanel};
//! use std::rc::Rc;
//! use std::path::PathBuf;
//!
//! let controller = Rc::new(
//!     Controller::new(PathBuf::from("~/.config/hypr/hyprland.conf"))
//!         .expect("Failed to create controller")
//! );
//!
//! let panel = ConflictPanel::new(controller.clone());
//!
//! // Initially hidden (no conflicts loaded yet)
//! // After loading keybindings:
//! panel.refresh();  // Shows banner if conflicts exist
//! ```

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, Revealer};
use std::rc::Rc;

use crate::ui::Controller;

/// Warning panel that displays when keybinding conflicts are detected
///
/// This component uses GTK4's GtkBox widget to show a dismissible warning
/// banner. It queries the Controller for conflicts and updates its visibility
/// and message accordingly.
pub struct ConflictPanel {
    /// Root widget (Revealer for smooth show/hide animation)
    widget: Revealer,
    /// Label displaying the conflict message and count
    message_label: Label,
    /// Controller for accessing conflict data
    controller: Rc<Controller>,
}

impl ConflictPanel {
    /// Creates a new conflict warning panel
    ///
    /// The panel is initially hidden (revealed = false). Call `refresh()` after
    /// loading keybindings to update the panel based on actual conflict state.
    ///
    /// # Arguments
    ///
    /// * `controller` - Shared controller for accessing conflict data
    ///
    /// # Returns
    ///
    /// A new ConflictPanel instance ready to be added to a window
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use hypr_keybind_manager::ui::{Controller, components::ConflictPanel};
    /// # use std::rc::Rc;
    /// # use std::path::PathBuf;
    /// let controller = Rc::new(Controller::new(PathBuf::from("test.conf")).unwrap());
    /// let panel = ConflictPanel::new(controller);
    ///
    /// // Add to window
    /// // vbox.append(panel.widget());
    /// ```
    pub fn new(controller: Rc<Controller>) -> Self {
        // Create revealer for smooth animations
        let revealer = Revealer::builder()
            .transition_type(gtk4::RevealerTransitionType::SlideDown)
            .transition_duration(300)
            .reveal_child(false)
            .build();

        // Create warning box with styling
        let warning_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(10)
            .margin_start(10)
            .margin_end(10)
            .margin_top(5)
            .margin_bottom(5)
            .hexpand(true)
            .build();

        warning_box.add_css_class("warning-banner");

        // Create the message label
        let message_label = Label::builder()
            .label("No conflicts detected")
            .xalign(0.0)
            .margin_start(10)
            .margin_end(10)
            .margin_top(5)
            .margin_bottom(5)
            .build();

        warning_box.append(&message_label);
        revealer.set_child(Some(&warning_box));

        Self {
            widget: revealer,
            message_label,
            controller,
        }
    }

    /// Updates the panel based on current conflict state
    ///
    /// Queries the Controller for conflicts and:
    /// - Shows the panel if conflicts exist (with count)
    /// - Hides the panel if no conflicts exist
    ///
    /// The panel smoothly animates in/out using GTK4's reveal animation.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use hypr_keybind_manager::ui::{Controller, components::ConflictPanel};
    /// # use std::rc::Rc;
    /// # use std::path::PathBuf;
    /// # let controller = Rc::new(Controller::new(PathBuf::from("test.conf")).unwrap());
    /// # let panel = ConflictPanel::new(controller.clone());
    /// // After loading keybindings
    /// panel.refresh();  // Updates based on current conflicts
    ///
    /// // After resolving conflicts
    /// panel.refresh();  // Panel will hide automatically
    /// ```
    pub fn refresh(&self) {
        let conflicts = self.controller.get_conflicts();

        if conflicts.is_empty() {
            // No conflicts - hide the panel
            self.widget.set_reveal_child(false);
            self.message_label.set_label("No conflicts detected");
        } else {
            // Conflicts exist - show the panel with count
            self.widget.set_reveal_child(true);

            let count = conflicts.len();
            let message = if count == 1 {
                "⚠️  Warning: 1 keybinding conflict detected".to_string()
            } else {
                format!("⚠️  Warning: {} keybinding conflicts detected", count)
            };

            self.message_label.set_label(&message);
        }
    }

    /// Returns the root widget for adding to a container
    ///
    /// # Returns
    ///
    /// Reference to the GtkBox widget that can be appended to a Box or other container
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use hypr_keybind_manager::ui::{Controller, components::ConflictPanel};
    /// # use std::rc::Rc;
    /// # use std::path::PathBuf;
    /// # use gtk4::prelude::*;
    /// # let controller = Rc::new(Controller::new(PathBuf::from("test.conf")).unwrap());
    /// # let panel = ConflictPanel::new(controller);
    /// # let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    /// vbox.append(panel.widget());
    /// ```
    pub fn widget(&self) -> &Revealer {
        &self.widget
    }

    /// Returns the current conflict count displayed
    ///
    /// Useful for checking panel state without querying the Controller directly.
    ///
    /// # Returns
    ///
    /// Number of conflicts currently detected, or 0 if panel is hidden
    pub fn conflict_count(&self) -> usize {
        self.controller.get_conflicts().len()
    }
}