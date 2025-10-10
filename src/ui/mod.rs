//! src/ui/mod.rs
//!
//! GTK4 layer-shell user interface
//!
//! This module implements the popup overlay window using GTK4 and
//! gtk4-layer-shell for proper Wayland layer-shell positioning.
//!
//! # Architecture
//! - Layer-shell window configuration (overlay layer, keyboard focus)
//! - Keybinding list view with filtering/search
//! - Conflict highlighting
//! - Real-time updates via file watching
//!
//! # Memory Safety
//! All GTK signal handlers use weak references (#[weak]) to prevent
//! reference cycles and memory leaks. Panics are caught at FFI boundaries
//! to prevent undefined behavior.
//!
//! # Implementation Note
//! GUI implementation coming in future step.

// GUI will be implemented after parser is complete