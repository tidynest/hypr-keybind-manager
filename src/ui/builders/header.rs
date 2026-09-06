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

//! Header bar builder
//!
//! Creates the application header bar: undo/redo, Add, Backups, Apply to
//! Hyprland and the application menu. Buttons use symbolic icons from the
//! icon theme so they match whatever GTK theme is active.

use gtk4::{
    Box as GtkBox, Button, HeaderBar, Image, Label, MenuButton, Orientation, gio::Menu, prelude::*,
};

/// The header bar and the buttons other modules wire up
pub struct HeaderWidgets {
    pub header_bar: HeaderBar,
    pub add_button: Button,
    pub backup_button: Button,
}

/// A button showing a symbolic icon followed by a text label
pub fn icon_button(icon: &str, label: &str) -> Button {
    let content = GtkBox::new(Orientation::Horizontal, 6);
    content.append(&Image::from_icon_name(icon));
    content.append(&Label::new(Some(label)));
    Button::builder().child(&content).build()
}

/// Builds the application header bar
pub fn build_header_bar() -> HeaderWidgets {
    let header_bar = HeaderBar::new();

    let menu = Menu::new();
    menu.append(Some("Export..."), Some("app.export"));
    menu.append(Some("Import..."), Some("app.import"));
    menu.append(Some("Quit"), Some("app.quit"));

    let menu_button = MenuButton::new();
    menu_button.set_icon_name("open-menu-symbolic");
    menu_button.set_menu_model(Some(&menu));
    menu_button.set_tooltip_text(Some("Open the application menu"));

    let undo_button = Button::builder()
        .icon_name("edit-undo-symbolic")
        .action_name("app.undo")
        .tooltip_text("Undo the last change (Ctrl+Z)")
        .focus_on_click(false)
        .build();

    let redo_button = Button::builder()
        .icon_name("edit-redo-symbolic")
        .action_name("app.redo")
        .tooltip_text("Redo the last undone change (Ctrl+Shift+Z)")
        .focus_on_click(false)
        .build();

    let add_button = icon_button("list-add-symbolic", "Add");
    add_button.set_tooltip_text(Some("Create a new keybinding (Ctrl+N)"));

    let backup_button = icon_button("document-open-recent-symbolic", "Backups");
    backup_button.set_tooltip_text(Some("Browse, restore, or delete automatic backups"));

    let apply_button = icon_button("view-refresh-symbolic", "Apply to Hyprland");
    apply_button.set_action_name(Some("app.apply-to-hyprland"));
    apply_button.set_tooltip_text(Some("Reload Hyprland with the saved config (Ctrl+R)"));
    apply_button.set_focus_on_click(false);
    apply_button.add_css_class("suggested-action");

    header_bar.pack_start(&undo_button);
    header_bar.pack_start(&redo_button);
    header_bar.pack_start(&add_button);
    header_bar.pack_end(&menu_button);
    header_bar.pack_end(&apply_button);
    header_bar.pack_end(&backup_button);

    HeaderWidgets {
        header_bar,
        add_button,
        backup_button,
    }
}
