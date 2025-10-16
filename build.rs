// Copyright 2025 bakri (tidynest@proton.me)
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

//! Build script for Hyprland Keybinding Manager
//!
//! Handles compilation-time dependency checks and library linking.
//!
//! # Dependencies
//!
//! This script probes for the `gtk4-layer-shell-0` library, which provides
//! Wayland layer-shell support for GTK4 applications. This library is required
//! for creating properly layered popup windows in Wayland compositors.
//!
//! # System Requirements
//!
//! - **Arch Linux:** `sudo pacman -S gtk4-layer-shell`
//! - **Other distros:** Install `gtk4-layer-shell` development package
//!
//! # Panics
//!
//! Exits with code 1 if `gtk4-layer-shell-0` is not found on the system.

/// Main build script entry point.
///
/// Probes for the gtk4-layer-shell library using pkg-config and configures
/// cargo to link against it. If the library is not found, prints installation
/// instructions and exits with an error.
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
