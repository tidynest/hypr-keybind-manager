// Copyright 2025 Eric Jingryd (tidynest@proton.me)
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

use crate::core::types::Keybinding;
use thiserror::Error;

/// Validation errors
#[derive(Debug, Error, PartialEq)]
pub enum ValidationError {
    /// Dispatcher name not in whitelist
    #[error("Invalid dispatcher '{0}': not in whitelist")]
    InvalidDispatcher(String),

    /// Key name contains invalid characters
    #[error("Invalid key name '{0}'")]
    InvalidKey(String),

    /// Dangerous shell metacharacters detected in arguments
    #[error("Dangerous shell metacharacters detected in arguments: '{0}'")]
    ShellMetacharacters(String),

    /// Argument exceeds maximum length
    #[error("Argument too long: {0} characters (max 1000)")]
    ArgumentTooLong(usize),
}

/// Hyprland dispatcher whitelist
///
/// Source: https://wiki.hyprland.org/Configuring/Dispatchers/
/// Last verified: October 2025
const ALLOWED_DISPATCHERS: &[&str] = &[
    "exec",
    "execr",
    "killactive",
    "closewindow",
    "workspace",
    "movetoworkspace",
    "togglefloating",
    "fullscreen",
    "pseudo",
    "pin",
    "movefocus",
    "movewindow",
    "swapwindow",
    "centerwindow",
    "resizeactive",
    "moveactive",
    "cyclenext",
    "focuswindow",
    "focusmonitor",
    "splitratio",
    "toggleopaque",
    "movecursortocorner",
    "workspaceopt",
    "exit",
    "forcerendererreload",
    "movecurrentworkspacetomonitor",
    "focusurgentor",
    "togglespecialworkspace",
    "togglegroup",
    "changegroupactive",
    "moveintogroup",
    "moveoutofgroup",
    "lockgroups",
    "lockactivegroup",
    "movegroupwindow",
    "pass",
    "sendshortcut",
    "layoutmsg",
    "dpms",
    "submap",
    "global",
];

/// Validates dispatcher name against whitelist
pub fn validate_dispatcher(name: &str) -> Result<(), ValidationError> {
    if ALLOWED_DISPATCHERS.contains(&name.to_lowercase().as_str()) {
        Ok(())
    } else {
        Err(ValidationError::InvalidDispatcher(name.to_string()))
    }
}

/// Checks for shell metacharacters that enable command injection
///
/// Detects: ; | & $ ` ( ) { } [ ] < > \ " ' and newlines
pub fn check_shell_metacharacters(input: &str) -> Result<(), ValidationError> {
    const DANGEROUS_CHARS: &[char] = &[
        ';', '|', '&', '$', '`', '(', ')', '{', '}', '[', ']', '<', '>', '\\', '"', '\'', '\n',
        '\r',
    ];

    for ch in DANGEROUS_CHARS {
        if input.contains(*ch) {
            return Err(ValidationError::ShellMetacharacters(input.to_string()));
        }
    }

    Ok(())
}

/// Validates key name format
///
/// Accepts alphanumeric, underscores, colons, hyphens (for mouse buttons),
/// and common special keys (Return, Escape, Space, Tab, arrow keys, etc.)
pub fn validate_key(key: &str) -> Result<(), ValidationError> {
    let valid = key
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == ':' || c == '-')
        || matches!(
            key,
            "Return"
                | "Escape"
                | "Space"
                | "Tab"
                | "BackSpace"
                | "Delete"
                | "Insert"
                | "Home"
                | "End"
                | "Prior"
                | "Next"
                | "Left"
                | "Right"
                | "Up"
                | "Down"
        );

    if valid {
        Ok(())
    } else {
        Err(ValidationError::InvalidKey(key.to_string()))
    }
}

/// Validates complete keybinding before execution
///
/// Performs all validation checks:
/// - Dispatcher whitelist
/// - Key name format
/// - Argument length limit (1000 chars)
/// - Shell metacharacter detection
pub fn validate_keybinding(binding: &Keybinding) -> Result<(), ValidationError> {
    // Validate dispatcher against whitelist
    validate_dispatcher(&binding.dispatcher)?;

    // Validate key name
    validate_key(&binding.key_combo.key)?;

    // Validate arguments if present
    if let Some(args) = &binding.args {
        // Check length limit (prevent memory exhaustion)
        if args.len() > 1000 {
            return Err(ValidationError::ArgumentTooLong(args.len()));
        }

        // Check for shell injection attempts
        check_shell_metacharacters(args)?;
    }

    Ok(())
}
