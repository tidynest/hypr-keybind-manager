//! src/ipc/mod.rs
//!
//! Hyprland IPC communication module
//!
//! This module provides safe communication with the Hyprland compositor
//! via its UNIX socket IPC interface. It wraps the hyprland-rs crate
//! with additional security validation.
//!
//! # Security Features
//! - Parameterized command construction (no string interpolation)
//! - Dispatcher whitelist enforcement
//! - Argument validation before transmission
//! - Socket permission verification
//! - Request timeouts to prevent DoS
//!
//! # Implementation Note
//! IPC client implementation coming in future step.

// IPC client will be implemented later
