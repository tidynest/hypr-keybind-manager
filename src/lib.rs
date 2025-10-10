//! Hyprland Keybinding Manager
//!
//! A secure, high-performance keybinding manager for Hyprland with
//! real-time conflict detection and a GTK4 layer-shell GUI.
//!
//! # Architecture
//! - `core`: Business logic (types, parser, conflict detection, validation)
//! - `config`: File operations (reading, writing, atomic updates)
//! - `ipc`: Hyprland IPC communication
//! - `ui`: GTK4 layer-shell GUI
//!
//! # Security
//! This crate is designed with security-first principles:
//! - Whitelist-based input validation
//! - Atomic file operations
//! - No arbitrary code execution
//! - Memory-safe by default (no unsafe blocks)

pub mod config;
pub mod core;
pub mod ipc;
pub mod ui;

// Re-export commonly used types for convenience
pub use core::{Keybinding, KeyCombo, Modifier, BindType};
// TODO: Add ConflictDetector when implemented