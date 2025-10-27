use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Orientation, ScrolledWindow, Window};
use std::rc::Rc;
use crate::ui::Controller;
use crate::ui::components::{ConflictPanel, KeybindList};

pub struct ConflictResolutionDialog {
    window: Window,
}

impl ConflictResolutionDialog {
    pub fn new(
        parent: &Window,
        controller: Rc<Controller>,
        conflict_panel: Rc<ConflictPanel>,
        keybind_list: Rc<KeybindList>
    ) -> Self {
        let window = Window::builder()
            .title("Resolve Conflicts")
            .modal(true)
            .transient_for(parent)
            .default_width(500)
            .default_height(400)
            .build();

        // Main container
        let main_box = GtkBox::new(Orientation::Vertical, 12);
        main_box.set_margin_top(12);
        main_box.set_margin_bottom(12);
        main_box.set_margin_start(12);
        main_box.set_margin_end(12);

        // Scrolled window for conflict list
        let scrolled = ScrolledWindow::builder()
            .vexpand(true)
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .build();

        // Container for all conflicts
        let conflict_box = GtkBox::new(Orientation::Vertical, 16);
        conflict_box.set_margin_start(6);
        conflict_box.set_margin_end(6);

        // Get conflicts from controller
        let conflicts = controller.get_conflicts();

        for conflict in conflicts.iter() {
            // Group container for this conflict
            let group_box = GtkBox::new(Orientation::Vertical, 8);
            group_box.set_margin_start(20);

            // Header showing conflicted key combo
            let header = gtk4::Label::new(Some(&format!("⚠️ Conflict: {}", conflict.key_combo)));
            header.set_halign(gtk4::Align::Start);
            header.add_css_class("conflict-header");
            group_box.append(&header);

            // List each conflicting binding
            for binding in conflict.conflicting_bindings.iter() {
                let binding_row = GtkBox::new(Orientation::Horizontal, 8);
                binding_row.set_margin_start(20);

                // Binding description
                let description = if let Some(args) = &binding.args {
                    format!("{} {}", binding.dispatcher, args)
                } else {
                    binding.dispatcher.clone()
                };

                let label = gtk4::Label::new(Some(&description));
                label.set_halign(gtk4::Align::Start);
                label.set_hexpand(true);
                binding_row.append(&label);

                // Delete button
                let delete_button = Button::with_label("Delete");
                delete_button.add_css_class("destructive-action");
                binding_row.append(&delete_button);

                // Wire up delete handler
                let binding_clone = binding.clone();
                let controller_clone = controller.clone();
                let window_clone = window.clone();
                let conflict_panel_clone = conflict_panel.clone();
                let keybind_list_clone = keybind_list.clone();
                delete_button.connect_clicked(move |_| {
                    eprintln!("🗑️ Deleting keybinding: {}", binding_clone);
                    if let Err(e) = controller_clone.delete_keybinding(&binding_clone) {
                        eprintln!("❌ Error deleting keybinding: {}", e);
                    } else {
                        eprintln!("✅ Keybinding deleted successfully");
                        // Refresh UI
                        let all_bindings = controller_clone.get_keybindings();
                        keybind_list_clone.update_with_bindings(all_bindings);
                        conflict_panel_clone.refresh();
                        window_clone.close();
                        }
                    });

                group_box.append(&binding_row);
            }

            conflict_box.append(&group_box);
        }

        scrolled.set_child(Some(&conflict_box));
        main_box.append(&scrolled);

        // Close button
        let close_button = Button::with_label("Close");
        let window_clone = window.clone();
        close_button.connect_clicked(move |_| {
            window_clone.close();
        });

        main_box.append(&close_button);

        window.set_child(Some(&main_box));

        Self { window }
    }

    pub fn show(&self) {
        self.window.present();
    }
}