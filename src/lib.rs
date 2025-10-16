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

//! Hyprland Keybinding Manager
//!
//! A secure, high-performance keybinding manager for Hyprland with
//! real-time conflict detection and a GTK4 GUI.
//!
//! # Features
//!
//! - **Conflict Detection:** Real-time detection of duplicate keybindings
//! - **CRUD Operations:** Create, read, update, and delete keybindings
//! - **Automatic Backups:** Timestamped backups before every config change
//! - **GTK4 Interface:** Modern, responsive graphical interface
//! - **Three-Layer Security:** Injection prevention, danger detection, validation
//! - **Atomic Operations:** Safe file writes with rollback on failure
//!
//! # Architecture
//!
//! - **`core`:** Business logic (types, parser, conflict detection, validation)
//! - **`config`:** File operations (reading, writing, atomic updates, backups)
//! - **`ipc`:** Hyprland IPC communication (future)
//! - **`ui`:** GTK4 GUI components (MVC pattern)
//!
//! # Security
//!
//! This crate is designed with security-first principles:
//!
//! - **Layer 1:** Input validation (core/validator.rs)
//! - **Layer 2:** Dangerous command detection (config/danger.rs)
//! - **Layer 3:** Config validation (config/validator.rs)
//! - **Atomic file operations:** No partial writes
//! - **No arbitrary code execution:** Whitelist-based validation
//! - **Memory-safe:** 100% safe Rust (no unsafe blocks)
//!
//! # Examples
//!
//! ## Parsing a config file
//!
//! ```no_run
//! use hypr_keybind_manager::core::parser::parse_config_file;
//! use std::path::Path;
//!
//! let content = std::fs::read_to_string("/tmp/hyprland.conf")?;
//! let bindings = parse_config_file(&content, Path::new("/tmp"))?;
//! println!("Found {} keybindings", bindings.len());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Detecting conflicts
//!
//! ```no_run
//! use hypr_keybind_manager::core::ConflictDetector;
//! # use hypr_keybind_manager::core::parser::parse_config_file;
//! # use std::path::Path;
//! # let content = std::fs::read_to_string("/tmp/hyprland.conf")?;
//! # let bindings = parse_config_file(&content, Path::new("/tmp"))?;
//!
//! let mut detector = ConflictDetector::new();
//! for binding in bindings {
//!     detector.add_binding(binding);
//! }
//!
//! let conflicts = detector.find_conflicts();
//! if conflicts.is_empty() {
//!     println!("No conflicts!");
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Using the GUI
//!
//! ```no_run
//! use hypr_keybind_manager::ui::App;
//! use std::path::PathBuf;
//!
//! let app = App::new(PathBuf::from("~/.config/hypr/hyprland.conf"))?;
//! app.run(); // Blocks until window closes
//! # Ok::<(), String>(())
//! ```

pub mod config;
pub mod core;
pub mod ipc;
pub mod ui;

// Re-export commonly used types for convenience
pub use core::{BindType, Keybinding, KeyCombo, Modifier};
