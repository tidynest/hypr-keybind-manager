//! src/core/mod.rs
//!
//! Core business logic module
//!
//! This module contains the fundamental data structures and algorithms
//! for keybinding management, including:
//! - Type definitions for keybindings and key combinations
//! - Conflict detection using HashMap-based O(1) lookup
//! - Input validation with security whitelisting
//! - Configuration parsing
//!
//! All business logic is isolated from UI and I/O concerns to enable
//! comprehensive unit testing without requiring a display server.

pub mod conflict;
pub use conflict::{ConflictDetector, Conflict};
pub mod parser;
pub mod types;
pub mod validator;

// Re-export commonly used types
pub use types::*;

// TODO: Implement validator functions
// pub use validator::*;