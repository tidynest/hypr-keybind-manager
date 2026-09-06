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
use std::{collections::HashMap, fmt};

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

/// Bind flags as documented by Hyprland, in the order they are rendered.
///
/// A bind line is `bind` followed by any of these letters, so `bindel`
/// repeats while held and works on the lock screen.
pub const BIND_FLAGS: [(char, &str); 12] = [
    ('e', "repeat while held"),
    ('l', "works on the lock screen"),
    ('r', "triggers on release"),
    ('c', "triggers on release without movement"),
    ('n', "non-consuming, the key still reaches the window"),
    ('m', "mouse binding"),
    ('t', "transparent, cannot be shadowed by other binds"),
    ('i', "ignores modifiers"),
    ('s', "separate, any combination of the given modifiers"),
    ('d', "has a description"),
    ('o', "long press"),
    ('p', "bypasses app requests to inhibit keybinds"),
];

/// Type of keybinding: `bind` plus any set of Hyprland flag letters.
///
/// The six common variants are available as constants (`BindType::Bind`,
/// `BindType::BindEL`, ...). Any other combination such as `bindd` or `bindnt`
/// is represented too, so a config using them still loads.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct BindType {
    flags: u16,
}

#[allow(non_upper_case_globals)] // enum-style names, kept so call sites read as before
impl BindType {
    /// Standard keybinding
    pub const Bind: Self = Self { flags: 0 };
    /// Repeat on hold
    pub const BindE: Self = Self {
        flags: Self::bit('e'),
    };
    /// Works on locked screen
    pub const BindL: Self = Self {
        flags: Self::bit('l'),
    };
    /// Mouse binding
    pub const BindM: Self = Self {
        flags: Self::bit('m'),
    };
    /// Trigger on release
    pub const BindR: Self = Self {
        flags: Self::bit('r'),
    };
    /// Repeat on hold + locked screen
    pub const BindEL: Self = Self {
        flags: Self::bit('e') | Self::bit('l'),
    };

    const fn bit(flag: char) -> u16 {
        let mut i = 0;
        while i < BIND_FLAGS.len() {
            if BIND_FLAGS[i].0 == flag {
                return 1 << i;
            }
            i += 1;
        }
        0
    }

    /// Builds a bind type from the letters after `bind`, e.g. `"el"`.
    ///
    /// Returns the offending character if one is not a known flag.
    pub fn from_flags(flags: &str) -> Result<Self, char> {
        let mut bits = 0;
        for c in flags.chars() {
            let bit = Self::bit(c);
            if bit == 0 {
                return Err(c);
            }
            bits |= bit;
        }
        Ok(Self { flags: bits })
    }

    /// Whether the given flag letter is set
    pub fn has(&self, flag: char) -> bool {
        self.flags & Self::bit(flag) != 0
    }

    /// Whether this is a `bindd` style binding carrying a description
    pub fn has_description(&self) -> bool {
        self.has('d')
    }

    /// Returns a copy with the description flag set or cleared
    pub fn with_description(self, enabled: bool) -> Self {
        let bit = Self::bit('d');
        Self {
            flags: if enabled {
                self.flags | bit
            } else {
                self.flags & !bit
            },
        }
    }

    /// The flag letters in canonical order, e.g. `"el"`
    pub fn flags(&self) -> String {
        BIND_FLAGS
            .iter()
            .filter(|(c, _)| self.has(*c))
            .map(|(c, _)| *c)
            .collect()
    }
}

impl fmt::Display for BindType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bind{}", self.flags())
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
/// ```
#[derive(Clone, Debug, Default, Deserialize, Eq, Serialize)]
pub struct KeyCombo {
    /// Modifier keys (SUPER, CTRL, SHIFT, ALT)
    /// Stored in a Vec to allow multiple modifiers
    pub modifiers: Vec<Modifier>,

    /// Base key name as written in the config (e.g., "K", "Return", "XF86AudioMute").
    /// Compared and hashed case-insensitively, since Hyprland resolves keysyms that way.
    pub key: String,
}

impl KeyCombo {
    /// Create a new KeyCombo with normalized data
    ///
    /// Normalisation includes:
    /// - Sorting modifiers for consistent hashing
    /// - Removing duplicate modifiers
    /// - Trimming the key name (case is kept for display)
    pub fn new(mut modifiers: Vec<Modifier>, key: &str) -> Self {
        // Sort modifiers for consistent hashing, SUPER first as configs are usually written
        modifiers.sort_by_key(|m| *m as u8);

        // Remove duplicates
        modifiers.dedup();

        Self {
            modifiers,
            key: key.trim().to_string(),
        }
    }
}

impl PartialEq for KeyCombo {
    fn eq(&self, other: &Self) -> bool {
        self.modifiers == other.modifiers && self.key.eq_ignore_ascii_case(&other.key)
    }
}

impl std::hash::Hash for KeyCombo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.modifiers.hash(state);
        self.key.to_ascii_uppercase().hash(state);
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
                .map(|m| m.to_string())
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
///     description: None,
///     submap: None,
/// };
/// // Represents: bind = SUPER, K, exec, firefox
/// ```
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Keybinding {
    /// The key combination that triggers this binding
    pub key_combo: KeyCombo,

    /// Type of binding (bind, binde, bindl, etc.)
    pub bind_type: BindType,

    /// Hyprland dispatcher name (e.g., "exec", "killactive", "workspace")
    pub dispatcher: String,

    /// Optional arguments for the dispatcher
    /// Examples:
    /// - exec: Some("firefox")
    /// - workspace: Some("3")
    /// - killactive: None
    pub args: Option<String>,

    /// Human-readable description, the extra field of a `bindd` line
    #[serde(default)]
    pub description: Option<String>,

    /// Submap this binding lives in, `None` for the global map
    #[serde(default)]
    pub submap: Option<String>,
}

impl Keybinding {
    /// Renders the binding as a config line, e.g. `bind = SUPER SHIFT, K, exec, firefox`.
    ///
    /// `variables` maps names to values as found in the config. When the
    /// modifiers or the arguments equal a variable's value the `$name` form is
    /// written instead, so `$mainMod` survives an edit.
    pub fn to_config_line(&self, variables: &HashMap<String, String>) -> String {
        let mut modifiers = self
            .key_combo
            .modifiers
            .iter()
            .map(|m| m.to_string())
            .collect::<Vec<_>>();
        if let Some(alias) = variable_alias(&modifiers.join(" "), variables) {
            modifiers = vec![alias];
        } else if let Some(first) = modifiers.first_mut() {
            if let Some(alias) = variable_alias(first, variables) {
                *first = alias;
            }
        }

        let mut parts = vec![modifiers.join(" "), self.key_combo.key.clone()];
        if let Some(description) = &self.description {
            parts.push(description.clone());
        }
        parts.push(self.dispatcher.clone());
        if let Some(args) = &self.args {
            parts.push(variable_alias(args, variables).unwrap_or_else(|| args.clone()));
        }

        let bind_type = self.bind_type.with_description(self.description.is_some());
        format!("{} = {}", bind_type, parts.join(", "))
    }
}

/// Returns `$name` for the variable whose value equals `value`, if any.
///
/// Ties are broken by name so the output is deterministic.
fn variable_alias(value: &str, variables: &HashMap<String, String>) -> Option<String> {
    variables
        .iter()
        .filter(|(_, v)| v.eq_ignore_ascii_case(value.trim()))
        .map(|(name, _)| name)
        .min()
        .map(|name| format!("${name}"))
}

impl fmt::Display for Keybinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_config_line(&HashMap::new()))
    }
}
