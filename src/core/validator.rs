//! src/core/validator.rs
//!
//! Security-focused input validation
//!
//! This module implements whitelist-based validation to prevent:
//! - Shell command injection via keybinding arguments
//! - Invalid dispatcher names that could exploit Hyprland IPC
//! - Malformed key names that could cause parser issues
//!
//! # Security Philosophy
//! We use WHITELIST validation (allow known-good) rather than BLACKLIST
//! (block known-bad) because blacklists can be bypassed. Only explicitly
//! allowed dispatchers, keys, and argument formats are accepted.