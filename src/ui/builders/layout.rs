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
    components::{ConflictPanel, DetailsPanel, KeybindList, SearchBar, StatusBanner},
};
use gtk4::{
    Box as GtkBox, CallbackAction, Orientation, Paned, Shortcut, ShortcutController, ShortcutScope,
    ShortcutTrigger, prelude::*,
};
use std::rc::Rc;

pub const DEFAULT_WINDOW_WIDTH: i32 = 1000;
pub const IDEAL_RIGHT_PANEL_WIDTH: i32 = 300;
pub const MIN_LEFT_PANEL_WIDTH: i32 = 520;

/// The main window content and the components other modules talk to
pub struct MainLayout {
    pub main_vbox: GtkBox,
    pub paned: Paned,
    pub keybind_list: Rc<KeybindList>,
    pub details_panel: Rc<DetailsPanel>,
    pub conflict_panel: Rc<ConflictPanel>,
    pub status_banner: Rc<StatusBanner>,
}

/// Builds the main application layout
///
/// A vertical box holding the conflict banner, the status banner and a
/// paned area with the search bar and list on the left and the details
/// panel on the right.
pub fn build_main_layout(controller: Rc<Controller>) -> MainLayout {
    let main_vbox = GtkBox::new(Orientation::Vertical, 0);

    let conflict_panel = Rc::new(ConflictPanel::new(controller.clone()));
    main_vbox.append(conflict_panel.widget());

    let status_banner = Rc::new(StatusBanner::new());
    main_vbox.append(status_banner.widget());

    let paned = Paned::new(Orientation::Horizontal);
    paned.set_wide_handle(true);

    let left_vbox = GtkBox::new(Orientation::Vertical, 10);
    left_vbox.set_margin_start(10);
    left_vbox.set_margin_end(10);
    left_vbox.set_margin_top(10);
    left_vbox.set_margin_bottom(10);
    left_vbox.set_size_request(MIN_LEFT_PANEL_WIDTH, -1);

    let keybind_list = Rc::new(KeybindList::new(controller.clone()));

    // Search bar, focusable from anywhere with Ctrl+F
    let search_bar = SearchBar::new();
    left_vbox.append(search_bar.widget());

    let search_entry = search_bar.widget().clone();
    let focus_search = CallbackAction::new(move |_, _| {
        search_entry.grab_focus();
        glib::Propagation::Stop
    });
    let shortcuts = ShortcutController::new();
    shortcuts.set_scope(ShortcutScope::Global);
    shortcuts.add_shortcut(Shortcut::new(
        ShortcutTrigger::parse_string("<Control>f"),
        Some(focus_search),
    ));
    left_vbox.add_controller(shortcuts);

    left_vbox.append(keybind_list.widget());

    let keybind_list_for_search = keybind_list.clone();
    let controller_for_search = controller.clone();
    search_bar.widget().connect_search_changed(move |entry| {
        let query = entry.text().to_string();
        controller_for_search.set_search_query(query);
        keybind_list_for_search.update_with_bindings(controller_for_search.get_current_view());
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

    MainLayout {
        main_vbox,
        paned,
        keybind_list,
        details_panel,
        conflict_panel,
        status_banner,
    }
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
