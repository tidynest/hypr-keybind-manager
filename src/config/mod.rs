//! src/config/mod.rs
//!
//! Configuration file management module
//!
//! This module provides secure, atomic operations for reading and writing
//! Hyprland configuration files. It implements atomic writes to prevent
//! corruption from crashes or concurrent access, and includes backup/rollback
//! functionality for safe configuration modifications.
//!
//! # Security Features
//! - Atomic file writes (write-to-temp-then-rename pattern)
//! - Automatic backups before modifications
//! - Permission validation (0o600 for user-only access)
//! - Symlink attack prevention
//! - Path traversal protection

// This module will be implemented in future steps