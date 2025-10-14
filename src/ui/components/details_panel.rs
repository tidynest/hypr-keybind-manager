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
            .label("🎹 Key Combo:")
            .halign(Align::End)
            .xalign(1.0)
            .build();
        key_header.add_css_class("field-header");

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
            .label("⚡ Dispatcher:")
            .halign(Align::End)
            .xalign(1.0)
            .build();
        dispatcher_header.add_css_class("field-header");

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
            .label("📝 Arguments:")
            .halign(Align::End)
            .xalign(1.0)
            .build();
        args_header.add_css_class("field-header");

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
            .label("🔗 Bind Type:")
            .halign(Align::End)
            .xalign(1.0)
            .build();
        bind_type_header.add_css_class("field-header");

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
            .label("📊 Status:")
            .halign(Align::End)
            .xalign(1.0)
            .build();
        status_header.add_css_class("field-header");

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
                let key_combo_text = format!("{}", b.key_combo);
                self.key_label.set_label(&key_combo_text);
                self.key_label.set_can_target(true);
                self.key_label.set_has_tooltip(true);
                self.key_label.set_tooltip_text(Some(&key_combo_text));  // Tooltip for long combos

                self.dispatcher_label.set_label(&b.dispatcher);
                self.dispatcher_label.set_can_target(true);
                self.dispatcher_label.set_has_tooltip(true);
                self.dispatcher_label.set_tooltip_text(Some(&b.dispatcher));

                let args_text = b.args.as_deref().unwrap_or("(none)");
                self.args_label.set_label(args_text);
                self.args_label.set_can_target(true);
                self.args_label.set_has_tooltip(true);
                self.args_label.set_tooltip_text(Some(args_text));  // Tooltip shows full args

                // Format BindType for display
                let bind_type_str = match b.bind_type {
                    crate::core::types::BindType::Bind => "bind",
                    crate::core::types::BindType::BindE => "binde",
                    crate::core::types::BindType::BindL => "bindl",
                    crate::core::types::BindType::BindM => "bindm",
                    crate::core::types::BindType::BindR => "bindr",
                    crate::core::types::BindType::BindEL => "bindel",
                };
                self.bind_type_label.set_label(bind_type_str);

                // Check for conflicts and show which bindings conflict
                let conflicts = self.controller.get_conflicts();

                // Find conflicts involving this binding
                let mut conflicting_bindings = Vec::new();

                for conflict in conflicts.iter() {
                    // Check if this binding is part of this conflict
                    let is_involved = conflict.conflicting_bindings.iter().any(|cb| {
                        cb.key_combo == b.key_combo && cb.dispatcher == b.dispatcher
                    });

                    if is_involved {
                        // Collect all other bindings in this conflict
                        for cb in conflict.conflicting_bindings.iter() {
                            // Skip the current binding itself
                            if cb.key_combo == b.key_combo && cb.dispatcher == b.dispatcher {
                                continue;
                            }
                            conflicting_bindings.push(cb.clone());
                        }
                    }
                }

                // Format the status message based on conflicts found
                if conflicting_bindings.is_empty() {
                    self.status_label.set_label("✅ No conflicts");
                    self.status_label.set_tooltip_text(Some("This keybinding has no conflicts"));
                } else if conflicting_bindings.len() == 1 {
                    // Single conflic - show the full details
                    let other = &conflicting_bindings[0];
                    let args_preview = if let Some(args) = &other.args {
                        // Truncate long arguments for display
                        if args.len() > 30 {
                            format!("({}", &args[..30])
                        } else {
                            args.clone()
                        }
                    } else {
                        String::new()
                    };

                    let conflict_description = format!(
                        "⚠️ Conflicts with:\n{} → {} {}",
                        other.key_combo,
                        other.dispatcher,
                        args_preview
                    );

                    // Full text in tooltip
                    let full_conflict = format!(
                        "Conflicts with:  {} → {} {}",
                        other.key_combo,
                        other.dispatcher,
                        other.args.as_deref().unwrap_or("")
                    );

                    self.status_label.set_label(&conflict_description);
                    self.status_label.set_tooltip_text(Some(&full_conflict));
                } else {
                    // Multiple conflicts - show first one and count
                    let first_conflict = &conflicting_bindings[0];
                    let args_preview = if let Some(args) = &first_conflict.args {
                        if args.len() > 30 {
                            format!("({}", &args[..30])
                        } else {
                            args.clone()
                        }
                    } else {
                        String::new()
                    };

                    let conflict_description = format!(
                        "⚠️  {}\n   {} {}\n   (and {} more)",
                        first_conflict.key_combo,
                        first_conflict.dispatcher,
                        args_preview,
                        conflicting_bindings.len() - 1
                    );

                    // List all conflicts in tooltip
                    let mut full_conflicts = String::from("Conflicts with:\n");
                    for (i, cb) in conflicting_bindings.iter().enumerate() {
                        full_conflicts.push_str(&format!(
                            "{}. {} → {} {}\n",
                            i + 1,
                            cb.key_combo,
                            cb.dispatcher,
                            cb.args.as_deref().unwrap_or("")
                        ));
                    }

                    self.status_label.set_label(&conflict_description);
                    self.status_label.set_tooltip_text(Some(&full_conflicts));
                }
            }
            None => {
                // Show friendly placeholder when nothing is selected
                self.key_label.set_label("👈 Select a binding");
                self.key_label.set_tooltip_text(None);

                self.dispatcher_label.set_label("");
                self.dispatcher_label.set_tooltip_text(None);

                self.args_label.set_label("");
                self.args_label.set_tooltip_text(None);

                self.bind_type_label.set_label("");

                self.status_label.set_label("");
                self.status_label.set_tooltip_text(None);
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