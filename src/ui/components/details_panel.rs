//! Details panel component for displaying selected keybinding information.
//!
//! This component shows comprehensive details about a selected keybinding,
//! including its key combination, dispatcher, arguments, bind type, and
//! conflict status.

use gtk4::pango::WrapMode::WordChar;
use gtk4::prelude::*;
use gtk4::{Frame, Grid, Label, Align};
use std::rc::Rc;

use crate::core::types::Keybinding;
use crate::ui::Controller;

/// A panel that displays detailed information about a selected keybinding.
///
/// The panel shows:
/// - Key combination (e.g., "SUPER+K")
/// - Dispatcher (e.g., "exec")
/// - Arguments (e.g., "firefox")
/// - Bind type (e.g., "bind")
/// - Conflict status (whether this binding conflicts with others)
///
/// The panel width is enforced by the parent Paned widget in app.rs
pub struct DetailsPanel {
    /// Root widget (Frame)
    widget: Frame,
    /// Label displaying the key combination
    key_label: Label,
    /// Label displaying the dispatcher
    dispatcher_label: Label,
    /// Label displaying the arguments
    args_label: Label,
    /// Label displaying the bind type
    bind_type_label: Label,
    /// Label displaying conflict status
    status_label: Label,
    /// Controller for accessing conflict information
    controller: Rc<Controller>,
}

impl DetailsPanel {
    /// Create a new details panel.
    ///
    /// # Arguments
    ///
    /// * `controller` - Reference to the Controller for accessing keybinding data
    ///
    /// # Returns
    ///
    /// A new `DetailsPanel` instance
    pub fn new(controller: Rc<Controller>) -> Self {
        // Create the FRAME (content container)
        let frame = Frame::builder()
            .label("Selected Keybinding")
            .margin_start(10)
            .margin_end(10)
            .margin_top(10)
            .margin_bottom(10)
            .width_request(280)  // Request 280px width
            .build();

        // Create grid for two-column layout (label / value)
        let grid = Grid::builder()
            .row_spacing(10)
            .column_spacing(15)
            .margin_start(15)
            .margin_end(15)
            .margin_top(15)
            .margin_bottom(15)
            .build();

        // Row 0: Key Combo
        let key_header = Label::builder()
            .label("Key Combo:")
            .halign(Align::End)
            .xalign(1.0)
            .build();

        let key_label = Label::builder()
            .label("Select a keybinding...")
            .halign(Align::Start)
            .xalign(0.0)
            .wrap(true)
            .wrap_mode(WordChar)
            .max_width_chars(20)
            .build();

        grid.attach(&key_header, 0, 0, 1, 1);
        grid.attach(&key_label, 1, 0, 1, 1);

        // Row 1: Dispatcher
        let dispatcher_header = Label::builder()
            .label("Dispatcher:")
            .halign(Align::End)
            .xalign(1.0)
            .build();

        let dispatcher_label = Label::builder()
            .label("")
            .halign(Align::Start)
            .xalign(0.0)
            .wrap(true)
            .wrap_mode(WordChar)
            .max_width_chars(20)
            .build();

        grid.attach(&dispatcher_header, 0, 1, 1, 1);
        grid.attach(&dispatcher_label, 1, 1, 1, 1);

        // Row 2: Arguments
        let args_header = Label::builder()
            .label("Arguments:")
            .halign(Align::End)
            .xalign(1.0)
            .build();

        let args_label = Label::builder()
            .label("")
            .halign(Align::Start)
            .xalign(0.0)
            .wrap(true)
            .wrap_mode(WordChar)
            .max_width_chars(20)
            .build();

        grid.attach(&args_header, 0, 2, 1, 1);
        grid.attach(&args_label, 1, 2, 1, 1);

        // Row 3: Bind Type
        let bind_type_header = Label::builder()
            .label("Bind Type:")
            .halign(Align::End)
            .xalign(1.0)
            .build();

        let bind_type_label = Label::builder()
            .label("")
            .halign(Align::Start)
            .xalign(0.0)
            .wrap(true)
            .wrap_mode(WordChar)
            .max_width_chars(20)
            .build();

        grid.attach(&bind_type_header, 0, 3, 1, 1);
        grid.attach(&bind_type_label, 1, 3, 1, 1);

        // Row 4: Status
        let status_header = Label::builder()
            .label("Status:")
            .halign(Align::End)
            .xalign(1.0)
            .build();

        let status_label = Label::builder()
            .label("")
            .halign(Align::Start)
            .xalign(0.0)
            .wrap(true)
            .wrap_mode(WordChar)
            .max_width_chars(20)
            .build();

        grid.attach(&status_header, 0, 4, 1, 1);
        grid.attach(&status_label, 1, 4, 1, 1);

        // Add grid to frame
        frame.set_child(Some(&grid));

        Self {
            widget: frame,
            key_label,
            dispatcher_label,
            args_label,
            bind_type_label,
            status_label,
            controller,
        }
    }


    /// Update the panel to display information about a specific keybinding.
    ///
    /// If `None` is passed, the panel shows a "Select a keybinding..." message.
    ///
    /// # Arguments
    ///
    /// * `binding` - The keybinding to display, or `None` to clear
    pub fn update_binding(&self, binding: Option<&Keybinding>) {
        match binding {
            Some(b) => {
                // Display binding information
                self.key_label.set_label(&format!("{}", b.key_combo));
                self.dispatcher_label.set_label(&b.dispatcher);
                self.args_label.set_label(b.args.as_deref().unwrap_or("(none)"));

                // Format BindType for display
                let bind_type_str = match b.bind_type {
                    crate::core::types::BindType::Bind   => "bind",
                    crate::core::types::BindType::BindE  => "binde",
                    crate::core::types::BindType::BindL  => "bindl",
                    crate::core::types::BindType::BindM  => "bindm",
                    crate::core::types::BindType::BindR  => "bindr",
                    crate::core::types::BindType::BindEL => "bindel",
                };
                self.bind_type_label.set_label(bind_type_str);

                // Check for conflicts
                let conflicts = self.controller.get_conflicts();
                let has_conflict = conflicts.iter().any(|conflict| {
                    conflict.conflicting_bindings.iter().any(|cb| {
                        cb.key_combo == b.key_combo && cb.dispatcher == b.dispatcher
                    })
                });

                if has_conflict {
                    self.status_label.set_label("⚠️  Conflicts with another binding");
                } else {
                    self.status_label.set_label("✅ No conflicts");
                }
            }
            None => {
                self.key_label.set_label("Select a keybinding...");
                self.dispatcher_label.set_label("");
                self.args_label.set_label("");
                self.bind_type_label.set_label("");
                self.status_label.set_label("");
            }
        }
    }

    /// Get the root widget for adding to a container.
    ///
    /// # Returns
    ///
    /// Reference to the root `Frame` widget
    pub fn widget(&self) -> &Frame {
        &self.widget
    }
}