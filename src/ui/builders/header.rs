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