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

//! Layout builder
//!
//! Creates the main application layout structure.

use crate::ui::{
    Controller,
    components::{ConflictPanel, DetailsPanel, KeybindList, SearchBar},
};
use gtk4::{Box as GtkBox, Button, Orientation, Paned, prelude::*};
use std::rc::Rc;

pub const DEFAULT_WINDOW_WIDTH: i32 = 1000;
pub const IDEAL_RIGHT_PANEL_WIDTH: i32 = 280;
pub const MIN_LEFT_PANEL_WIDTH: i32 = 520;

/// Builds the main application layout
///
/// Creates a vertical box containing:
/// - Conflict panel at top
/// - Paned layout with:
///   - Left: Search bar, buttons, keybinding list
///   - Right: Details panel (fixed 280px width)
///
/// # Returns
///
/// Tuple of (main_vbox, keybind_list, details_panel, conflict_panel, add_button, backup_button)
pub fn build_main_layout(
    controller: Rc<Controller>,
) -> (
    GtkBox,
    Paned,
    Rc<KeybindList>,
    Rc<DetailsPanel>,
    Rc<ConflictPanel>,
    Button,
    Button,
) {
    // Create main vertical box
    let main_vbox = GtkBox::new(Orientation::Vertical, 0);

    // Create conflict panel at top
    let conflict_panel = Rc::new(ConflictPanel::new(controller.clone()));
    main_vbox.append(conflict_panel.widget());

    let paned = Paned::new(Orientation::Horizontal);
    paned.set_wide_handle(true);

    let left_vbox = GtkBox::new(Orientation::Vertical, 10);
    left_vbox.set_margin_start(10);
    left_vbox.set_margin_end(10);
    left_vbox.set_margin_top(10);
    left_vbox.set_margin_bottom(10);
    left_vbox.set_size_request(MIN_LEFT_PANEL_WIDTH, -1);

    // Create SINGLE keybind list instance
    let keybind_list = Rc::new(KeybindList::new(controller.clone()));

    // Create search bar
    let search_bar = SearchBar::new();
    left_vbox.append(search_bar.widget());

    let add_keybinding_button = Button::builder().label("➕ Add Keybinding").build();
    add_keybinding_button.add_css_class("suggested-action");
    add_keybinding_button.set_tooltip_text(Some("Create a new keybinding"));
    add_keybinding_button.set_can_focus(true);
    left_vbox.append(&add_keybinding_button);

    let backup_button = Button::builder().label("📦 Manage Backups").build();
    backup_button.set_tooltip_text(Some("Browse, restore, or delete automatic backups"));
    backup_button.set_can_focus(true);
    left_vbox.append(&backup_button);

    // Add keybind list to left side
    left_vbox.append(keybind_list.widget());

    // Wire up search functionality manually
    let keybind_list_for_search = keybind_list.clone();
    let controller_for_search = controller.clone();

    search_bar.widget().connect_search_changed(move |entry| {
        let query = entry.text().to_string();
        eprintln!("🔍 Search: '{}'", query);

        // Store the query in Controller (single source of truth)
        controller_for_search.set_search_query(query.clone());

        // Update the view with filtered results
        let filtered = controller_for_search.get_current_view();
        keybind_list_for_search.update_with_bindings(filtered);
    });

    let details_panel = Rc::new(DetailsPanel::new(controller.clone()));

    paned.set_start_child(Some(&left_vbox));
    paned.set_resize_start_child(true);
    paned.set_shrink_start_child(false);

    paned.set_end_child(Some(details_panel.widget()));
    paned.set_resize_end_child(false);
    paned.set_shrink_end_child(false);

    paned.set_position(clamp_paned_position(
        DEFAULT_WINDOW_WIDTH,
        DEFAULT_WINDOW_WIDTH,
    ));

    main_vbox.append(&paned);

    (
        main_vbox,
        paned,
        keybind_list,
        details_panel,
        conflict_panel,
        add_keybinding_button,
        backup_button,
    )
}

pub fn clamp_paned_position(window_width: i32, requested_position: i32) -> i32 {
    let effective_width = window_width.max(MIN_LEFT_PANEL_WIDTH + 120);
    let max_right_width = (effective_width / 3).max(180);
    let startup_right_width = IDEAL_RIGHT_PANEL_WIDTH.min(max_right_width);
    let requested_right_width = (effective_width - requested_position)
        .max(startup_right_width)
        .min(max_right_width);

    let max_position = effective_width - startup_right_width;
    let min_position = MIN_LEFT_PANEL_WIDTH.min(max_position);

    (effective_width - requested_right_width).clamp(min_position, max_position)
}
