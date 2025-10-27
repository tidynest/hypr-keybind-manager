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
//! Creates the application header bar with menu

use gtk4::{gio, HeaderBar, MenuButton};

/// Builds the application header bar with File menu
///
/// Creates a HeaderBar containing a menu button with:
/// - Export... (app.export action)
/// - Import... (app.import action)
/// - Quit (app.quit action)
///
/// # Returns
///
/// The configured HeaderBar widget
pub fn build_header_bar() -> HeaderBar {
    let header_bar = HeaderBar::new();

    // Menu options
    let menu = gio::Menu::new();
    menu.append(Some("Export..."), Some("app.export"));
    menu.append(Some("Import..."), Some("app.import"));
    menu.append(Some("Quit..."),   Some("app.quit"));

    // Menu button
    let menu_button = MenuButton::new();
    menu_button.set_icon_name("open-menu-symbolic");
    menu_button.set_menu_model(Some(&menu));
    header_bar.pack_end(&menu_button);

    header_bar
}