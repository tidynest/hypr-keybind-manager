//! build.rs
//!
//! Build script for hypr-keybind-manager
//!
//! This script handles linking to the gtk4-layer-shell C library,
//! which is required for creating Wayland layer-shell popup windows.
//! The library must be installed on the system (via pacman on Arch).
fn main() {
    // Link to gtk4-layer-shell C library
    match pkg_config::probe_library("gtk4-layer-shell-0") {
        Ok(_) => println!("cargo:rerun-if-changed=build.rs"),
        Err(e) => {
            eprintln!("Error: gtk4-layer-shell not found!");
            eprintln!("Install with: sudo pacman -S gtk4-layer-shell");
            eprintln!("Details: {}", e);
            std::process::exit(1);
        }
    }
}