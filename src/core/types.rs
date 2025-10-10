//! src/core/types.rs
//!
//! Core type definitions for keybinding management
//!
//! This module defines the fundamental types used throughout the application:
//! - `Modifier`: Keyboard modifier keys (SUPER, CTRL, SHIFT, ALT)
//! - `KeyCombo`: A combination of modifiers and a key
//! - `BindType`: Different types of Hyprland bindings (bind, bindl, binde, etc.)
//! - `Keybinding`: Complete keybinding with action and metadata
//!
//! All types implement serialization for config persistence and are designed
//! with security in mind (validation, normalization, consistent hashing).

use serde::{Deserialize, Serialize};
use std::fmt;

/// Keyboard modifier keys
///
/// Represents the four standard modifier keys used in keybindings.
/// These are mapped from various Hyprland names (e.g., MOD4, WIN → Super).
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Modifier {
    /// Super/Windows/Command key (MOD4)
    Super,
    /// Control key
    Ctrl,
    /// Shift key
    Shift,
    /// Alt key (MOD1)
    Alt,
}

impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Modifier::Super => write!(f, "SUPER"),
            Modifier::Ctrl => write!(f, "CTRL"),
            Modifier::Shift => write!(f, "SHIFT"),
            Modifier::Alt => write!(f, "ALT"),
        }
    }
}

/// Type of keybinding
///
/// Hyprland supports six different binding types with different behaviours:
/// - `Bind`: Standard binding
/// - `BindE`: Repeat while key is held (e for "repeat")
/// - `BindL`: Works on locked screen (l for "locked")
/// - `BindM`: Mouse binding (m for "mouse")
/// - `BindR`: Trigger on key release (r for "release")
/// - `BindEL`: Combination of BindE and BindL
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum BindType {
    /// Standard keybinding
    Bind,
    /// Repeat on hold
    BindE,
    /// Works on locked screen
    BindL,
    /// Mouse binding
    BindM,
    /// Trigger on release
    BindR,
    /// Repeat on hold + locked screen
    BindEL,
}

impl fmt::Display for BindType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BindType::Bind => write!(f, "bind"),
            BindType::BindE => write!(f, "binde"),
            BindType::BindL => write!(f, "bindl"),
            BindType::BindM => write!(f, "bindm"),
            BindType::BindR => write!(f, "bindr"),
            BindType::BindEL => write!(f, "bindel"),
        }
    }
}

/// A combination of modifier keys and a base key
///
/// Represents a complete key combination like SUPER+SHIFT+K.
/// Implements Hash and Eq for use as HashMap keys in conflict detection.
///
/// # Hash Implementation
/// The hash is based on a **sorted** list of modifiers plus the key name.
/// This ensures that different orderings of the same modifiers produce
/// the same hash (e.g., SUPER+SHIFT and SHIFT+SUPER are identical).
///
/// # Example
/// ```ignore
/// let combo = KeyCombo {
///     modifiers: vec![Modifier::Super, Modifier::Shift],
///     key: "K".to_string(),
/// };
/// ``
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct KeyCombo {
    /// Modifier keys (SUPER, CTRL, SHIFT, ALT)
    /// Stored in a Vec to allow multiple modifiers
    pub modifiers: Vec<Modifier>,

    /// Base key name (e.g., "K", "Return", "F1")
    /// Always stored in uppercase for consistent hashing
    pub key: String,
}

impl KeyCombo {
    /// Create a new KeyCombo with normalized data
    ///
    /// Normalisation includes:
    /// - Sorting modifiers for consistent hashing
    /// - Converting key to uppercase
    /// - Removing duplicate modifiers
    pub fn new(mut modifiers: Vec<Modifier>, key: &str) -> Self {
        // Sort modifiers for consistent hashing
        modifiers.sort_by_key(|m| format!("{:?}", m));

        // Remove duplicates
        modifiers.dedup();

        Self {
            modifiers,
            key: key.to_uppercase(),
        }
    }
}

impl fmt::Display for KeyCombo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.modifiers.is_empty() {
            write!(f, "{}", self.key)
        } else {
            let mods = self
                .modifiers
                .iter()
                .map(|m| format!("{}", m))
                .collect::<Vec<_>>()
                .join("+");
            write!(f, "{}+{}", mods, self.key)
        }
    }
}

/// A complete keybinding definition
///
/// Represents a full Hyprland keybinding with all its components:
/// - The key combination that triggers it
/// - The type of binding (bind, binde, etc.)
/// - The dispatcher (action) to execute
/// - Optional arguments for the dispatcher
///
/// # Example
/// ```ignore
/// let binding = Keybinding {
///     key_combo: KeyCombo::new(vec![Modifier::Super], "K"),
///     bind_type: BindType::Bind,
///     dispatcher: "exec".to_string(),
///     args: Some("firefox".to_string()),
/// };
/// // Represents: bind = SUPER, K, exec, firefox
/// ```
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Keybinding {
    /// The key combination that triggers this binding
    pub key_combo: KeyCombo,

    /// Type of bidning (bind, binde, bindl, etc.)
    pub bind_type: BindType,

    /// Hyprland dispatcher name (e.g., "exec", "killactive", "workspace")
    pub dispatcher: String,

    /// Optional arguments for the dispatcher
    /// Examples:
    /// - exec: Some("firefox")
    /// - workspace: Some("3")
    /// - killactive: None
    pub args: Option<String>,
}

impl fmt::Display for Keybinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}, {}", self.bind_type, self.key_combo, self.dispatcher)?;

        if let Some(args) = &self.args {
            write!(f, ", {}", args)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modifier_display() {
        assert_eq!(format!("{}", Modifier::Super), "SUPER");
        assert_eq!(format!("{}", Modifier::Ctrl), "CTRL");
    }

    #[test]
    fn test_bind_type_display() {
        assert_eq!(format!("{}", BindType::Bind), "bind");
        assert_eq!(format!("{}", BindType::BindEL), "bindel");
    }

    #[test]
    fn test_key_combo_normalization() {
        // Test that order doesn't matter
        let combo1 = KeyCombo::new(vec![Modifier::Super, Modifier::Shift], "k");
        let combo2 = KeyCombo::new(vec![Modifier::Shift, Modifier::Super], "K");

        assert_eq!(combo1, combo2); // Should be equal after normalization
    }

    #[test]
    fn test_key_combo_display() {
        let combo = KeyCombo::new(vec![Modifier::Super, Modifier::Shift], "K");
        let display = format!("{}", combo);

        // Should show modifiers in sorted order
        assert!(display.contains("SUPER"));
        assert!(display.contains("SHIFT"));
        assert!(display.contains("K"));
    }

    #[test]
    fn test_keybinding_display() {
        let binding = Keybinding {
            key_combo: KeyCombo::new(vec![Modifier::Super], "K"),
            bind_type: BindType::Bind,
            dispatcher: "exec".to_string(),
            args: Some("firefox".to_string()),
        };

        let display = format!("{}", binding);
        assert!(display.contains("bind"));
        assert!(display.contains("SUPER"));
        assert!(display.contains("K"));
        assert!(display.contains("exec"));
        assert!(display.contains("firefox"));
    }

    #[test]
    fn test_keybinding_no_args() {
        let binding = Keybinding {
            key_combo: KeyCombo::new(vec![Modifier::Super], "Q"),
            bind_type: BindType::Bind,
            dispatcher: "killactive".to_string(),
            args: None,
        };

        let display = format!("{}", binding);
        assert!(display.contains("killactive"));
        assert!(!display.ends_with(",")); // No trailing comma when no args
    }
}
