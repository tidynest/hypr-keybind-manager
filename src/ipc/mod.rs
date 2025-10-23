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

//! Hyprland IPC integration with defence-in-depth security
//!
//! # Safety Modes
//!
//! This module operates in three modes:
//! - **DryRun** (default): Validates only, never sends IPC
//! - **ReadOnly**: Can query Hyprland, cannot modify
//! - **Live**: Full access (requires explicit opt-in)
//!
//! Tests default to DryRun mode for safety.
//!
//! # Example
//! ```
//! use hypr_keybind_manager::ipc::{HyprlandClient, ClientMode};
//! use hypr_keybind_manager::core::{Keybinding, KeyCombo, Modifier, BindType};
//!
//! // Safe: DryRun mode validates but never sends IPC
//! let client = HyprlandClient::new(ClientMode::DryRun);
//!
//! let binding = Keybinding {
//!     key_combo: KeyCombo::new(vec![Modifier::Super], "K"),
//!     bind_type: BindType::Bind,
//!     dispatcher: "exec".to_string(),
//!     args: Some("firefox".to_string()),
//! };
//!
//! // Validates command but doesn't send to Hyprland
//! assert!(client.add_bind(&binding).is_ok());
//! ```

use crate::config::ConfigError;
use crate::core::{Keybinding, Modifier};
use crate::core::validator as injection_validator;

/// IPC client operation mode
///
/// Controls what operations are allowed. Tests default to DryRun.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientMode {
    /// Validation only - NEVER sends to Hyprland (default for tests)
    DryRun,

    /// Can query Hyprland but cannot modify config
    ReadOnly,

    /// Full access - requires explicit opt-in
    /// ONLY use in production or VM testing
    Live,
}

/// Hyprland IPC client with safety validation
///
/// Provides safe communication with Hyprland compositor via IPC socket.
/// All commands are validated before transmission (defence-in-depth).
///
/// # Defence-in-Depth Security
///
/// Before any IPC command is sent, it passes through multiple validation layers:
/// 1. **Injection validation** (Layer 1) - Blocks shell metacharacters
/// 2. **Command building** (Layer 2) - Safe construction without string interpolation
/// 3. **Mode check** (Layer 3) - DryRun/ReadOnly/Live enforcement
/// 4. **IPC transmission** (Layer 4) - Only in Live mode
///
/// # Example
/// ```
/// use hypr_keybind_manager::ipc::{HyprlandClient, ClientMode};
///
/// // Safe for testing - validates but never sends commands
/// let client = HyprlandClient::new(ClientMode::DryRun);
/// ```
pub struct HyprlandClient {
    /// Operation mode (DryRun/ReadOnly/Live)
    mode: ClientMode,
}

impl HyprlandClient {
    /// Creates a new client in the specified mode
    ///
    /// # Safety Modes
    ///
    /// - `DryRun`: Validates commands but never sends IPC (safe for tests)
    /// - `ReadOnly`: Can query but not modify (safe for inspection)
    /// - `Live`: Full access (requires explicit intent)
    ///
    /// # Example (Safe for tests)
    /// ```
    /// use hypr_keybind_manager::ipc::{HyprlandClient, ClientMode};
    ///
    /// let client = HyprlandClient::new(ClientMode::DryRun);
    /// // This client will validate but never actually send commands
    /// ```
    pub fn new(mode: ClientMode) -> Self {
        Self { mode }
    }

    /// Adds a keybinding to Hyprland
    ///
    /// # Defence-in-Depth Process
    ///
    /// 1. Validates binding for injection attempts (Layer 1)
    /// 2. Builds command safely (Layer 2)
    /// 3. Checks operation mode (Layer 3)
    /// 4. Sends to Hyprland only if Live mode (Layer 4)
    ///
    /// # Arguments
    ///
    /// * `binding` - The keybinding to add
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Command validated (DryRun) or sent successfully (Live)
    /// * `Err(ConfigError::ValidationFailed)` - Injection attempt blocked
    /// * `Err(ConfigError::IpcCommandFailed)` - Read-only mode or IPC failure
    ///
    /// # Example
    /// ```
    /// use hypr_keybind_manager::ipc::{HyprlandClient, ClientMode};
    /// use hypr_keybind_manager::core::{Keybinding, KeyCombo, Modifier, BindType};
    ///
    /// let client = HyprlandClient::new(ClientMode::DryRun);
    ///
    /// let binding = Keybinding {
    ///     key_combo: KeyCombo::new(vec![Modifier::Super], "K"),
    ///     bind_type: BindType::Bind,
    ///     dispatcher: "exec".to_string(),
    ///     args: Some("firefox".to_string()),
    /// };
    ///
    /// // Safe: validates but doesn't send in DryRun mode
    /// assert!(client.add_bind(&binding).is_ok());
    /// ```
    pub fn add_bind(&self, binding: &Keybinding) -> Result<(), ConfigError> {
        // Layer 1: Validate BEFORE IPC (defence-in-depth!)
        injection_validator::validate_keybinding(binding)
            .map_err(|e| ConfigError::ValidationFailed(e.to_string()))?;

        // Layer 2: Build command safely (no string interpolation)
        let cmd = self.build_keyword_command("bind", binding);

        // Layer 3: Mode check
        match self.mode {
            ClientMode::DryRun => {
                // Validation passed, but don't send IPC
                Ok(())
            }
            ClientMode::ReadOnly => {
                Err(ConfigError::IpcCommandFailed(
                    "Client in read-only mode - cannot modify bindings".to_string()
                ))
            }
            ClientMode::Live => {
                // Layer 4: Actually send to Hyprland
                self.send_keyword_command("bind", &cmd)
            }
        }
    }

    /// Removes a keybinding from Hyprland
    ///
    /// # Defence-in-Depth Process
    ///
    /// Same validation layers as `add_bind()`, ensuring that even
    /// removal commands are validated for safety.
    ///
    /// # Arguments
    ///
    /// * `binding` - The keybinding to remove
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Command validated (DryRun) or sent successfully (Live)
    /// * `Err(ConfigError)` - Validation failed or IPC error
    ///
    /// # Example
    /// ```
    /// use hypr_keybind_manager::ipc::{HyprlandClient, ClientMode};
    /// use hypr_keybind_manager::core::{Keybinding, KeyCombo, Modifier, BindType};
    ///
    /// let client = HyprlandClient::new(ClientMode::DryRun);
    ///
    /// let binding = Keybinding {
    ///     key_combo: KeyCombo::new(vec![Modifier::Super], "K"),
    ///     bind_type: BindType::Bind,
    ///     dispatcher: "exec".to_string(),
    ///     args: Some("firefox".to_string()),
    /// };
    ///
    /// // Safe: validates but doesn't send in DryRun mode
    /// assert!(client.remove_bind(&binding).is_ok());
    /// ```
    pub fn remove_bind(&self, binding: &Keybinding) -> Result<(), ConfigError> {
        // Layer 1: Validate (even for removal)
        injection_validator::validate_keybinding(binding)
            .map_err(|e| ConfigError::ValidationFailed(e.to_string()))?;

        // Layer 2: Build unbind command
        let cmd = self.build_keyword_command("unbind", binding);

        // Layer 3: Mode check
        match self.mode {
            ClientMode::DryRun => Ok(()),
            ClientMode::ReadOnly => {
                Err(ConfigError::IpcCommandFailed(
                    "Client in read-only mode - cannot modify bindings".to_string()
                ))
            }
            ClientMode::Live => {
                // Layer 4: Send to Hyprland
                self.send_keyword_command("unbind", &cmd)
            }
        }
    }

    /// Reloads Hyprland configuration from file
    ///
    /// This triggers Hyprland to re-read its config file, applying all
    /// changes at once. No validation needed as this just triggers a reload.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Reload command sent (or validated in DryRun)
    /// * `Err(ConfigError)` - Mode restriction or IPC error
    ///
    /// # Example
    /// ```
    /// use hypr_keybind_manager::ipc::{HyprlandClient, ClientMode};
    ///
    /// let client = HyprlandClient::new(ClientMode::DryRun);
    ///
    /// // Safe: just validates in DryRun mode
    /// assert!(client.reload().is_ok());
    /// ```
    pub fn reload(&self) -> Result<(), ConfigError> {
        match self.mode {
            ClientMode::DryRun => Ok(()),
            ClientMode::ReadOnly => {
                Err(ConfigError::IpcCommandFailed(
                    "Client in read-only mode - cannot reload config".to_string()
                ))
            }
            ClientMode::Live => {
                self.send_reload_command()
            }
        }
    }

    /// Builds a keyword command string safely
    ///
    /// This constructs the command using safe concatenation, NOT string
    /// interpolation or format macros that could be vulnerable to injection.
    ///
    /// # Arguments
    ///
    /// * `keyword` - The Hyprland keyword ("bind" or "unbind")
    /// * `binding` - The keybinding to encode
    ///
    /// # Returns
    ///
    /// A safely constructed command string in Hyprland's format:
    /// ```text
    /// MODIFIERS, KEY, dispatcher, args
    /// ```
    ///
    /// # Safety
    ///
    /// This method assumes the binding has already passed validation.
    /// It builds the command by concatenating validated components,
    /// not by interpolating user input into a format string.
    fn build_keyword_command(&self, _keyword: &str, binding: &Keybinding) -> String {
        // Build modifiers string (e.g., "SUPER_SHIFT")
        let modifiers_str = if binding.key_combo.modifiers.is_empty() {
            String::new()
        } else {
            binding.key_combo.modifiers
                .iter()
                .map(|m| match m {
                    Modifier::Super => "SUPER",
                    Modifier::Ctrl => "CTRL",
                    Modifier::Shift => "SHIFT",
                    Modifier::Alt => "ALT",
                })
                .collect::<Vec<_>>()
                .join("_")
        };

        // Build the command parts
        let mut parts = Vec::new();

        // Add modifiers if present
        if !modifiers_str.is_empty() {
            parts.push(modifiers_str);
        }

        // Add key
        parts.push(binding.key_combo.key.clone());

        // Add dispatcher
        parts.push(binding.dispatcher.clone());

        // Add args if present
        if let Some(args) = &binding.args {
            parts.push(args.clone());
        }

        // Join with commas and spaces (Hyprland format)
        parts.join(", ")
    }

    /// Sends a keyword command to Hyprland via IPC
    ///
    /// This is the actual IPC transmission layer. It should ONLY be called
    /// from Live mode after all validation has passed.
    ///
    /// # Arguments
    ///
    /// * `keyword` - The Hyprland keyword ("bind", "unbind", etc.)
    /// * `value` - The command value (already validated and built)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Command sent successfully
    /// * `Err(ConfigError)` - Hyprland not running or command failed
    fn send_keyword_command(&self, keyword: &str, value: &str) -> Result<(), ConfigError> {
        use hyprland::keyword::Keyword;

        // Attempt to send the command
        Keyword::set(keyword, value)
            .map_err(|e| {
                // Check if Hyprland is not running
                if e.to_string().contains("No such file or directory") {
                    ConfigError::HyprlandNotRunning(
                        "Hyprland IPC socket not found - is Hyprland running?".to_string()
                    )
                } else {
                    ConfigError::IpcCommandFailed(
                        format!("Failed to send keyword command: {}", e)
                    )
                }
            })?;

        Ok(())
    }

    /// Sends a reload command to Hyprland
    ///
    /// Uses the exec dispatcher to trigger a config reload via hyprctl.
    /// This is the standard way to reload Hyprland configuration.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Reload command sent successfully
    /// * `Err(ConfigError)` - Hyprland not running or command failed
    fn send_reload_command(&self) -> Result<(), ConfigError> {
        use hyprland::dispatch::{Dispatch, DispatchType};

        // Use exec dispatcher to run hyprctl reload
        Dispatch::call(DispatchType::Exec("hyprctl reload"))
            .map_err(|e| {
                if e.to_string().contains("No such file or directory") {
                    ConfigError::HyprlandNotRunning(
                        "Hyprland IPC socket not found - is Hyprland running?".to_string()
                    )
                } else {
                    ConfigError::IpcCommandFailed(
                        format!("Failed to reload config: {}", e)
                    )
                }
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests;
