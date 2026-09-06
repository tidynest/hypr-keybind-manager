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

//! Hyprland IPC client
//!
//! Talks to the running compositor over its control socket,
//! `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket.sock`, the
//! same channel `hyprctl` uses. A request is the command text; Hyprland
//! answers `ok` or an error message and closes the connection. Only the
//! standard library is involved.
//!
//! # Modes
//!
//! `ClientMode::DryRun` validates and builds commands without sending them,
//! `ClientMode::ReadOnly` refuses anything that would change state, and
//! `ClientMode::Live` sends to the socket. Every binding is run through the
//! injection validator before a command is built.
//!
//! # Example
//! ```no_run
//! use hypr_keybind_manager::ipc::{HyprlandClient, ClientMode};
//!
//! // Safe: DryRun mode validates but never sends IPC
//! let client = HyprlandClient::new(ClientMode::DryRun);
//! client.reload()?;
//! # Ok::<(), hypr_keybind_manager::config::ConfigError>(())
//! ```

use std::{
    env,
    io::{Read, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
    time::Duration,
};

use crate::config::ConfigError;
use crate::core::{Keybinding, validator as injection_validator};

/// How long to wait for the compositor to answer
const TIMEOUT: Duration = Duration::from_secs(5);

/// What the client is allowed to do
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientMode {
    /// Validate and build commands, send nothing
    DryRun,
    /// Send nothing that changes state
    ReadOnly,
    /// Send to the running Hyprland
    Live,
}

/// Client for Hyprland's control socket
pub struct HyprlandClient {
    mode: ClientMode,
}

impl HyprlandClient {
    pub fn new(mode: ClientMode) -> Self {
        Self { mode }
    }

    /// Adds a binding at runtime with `keyword bind<flags> ...`
    ///
    /// The binding is validated first, so an injection attempt never reaches
    /// the socket. Runtime binds do not survive a reload; write the config too.
    pub fn add_bind(&self, binding: &Keybinding) -> Result<(), ConfigError> {
        injection_validator::validate_keybinding(binding)
            .map_err(|e| ConfigError::ValidationFailed(e.to_string()))?;
        let value = self.build_keyword_command("bind", binding);
        self.modify(|| self.keyword(&binding.bind_type.to_string(), &value))
    }

    /// Removes a binding at runtime with `keyword unbind ...`
    pub fn remove_bind(&self, binding: &Keybinding) -> Result<(), ConfigError> {
        injection_validator::validate_keybinding(binding)
            .map_err(|e| ConfigError::ValidationFailed(e.to_string()))?;
        let value = self.build_keyword_command("unbind", binding);
        self.modify(|| self.keyword("unbind", &value))
    }

    /// Asks Hyprland to reload its config, like `hyprctl reload`
    pub fn reload(&self) -> Result<(), ConfigError> {
        self.modify(|| self.send("reload"))
    }

    /// Applies the mode gate to a state-changing action
    fn modify(&self, action: impl FnOnce() -> Result<(), ConfigError>) -> Result<(), ConfigError> {
        match self.mode {
            ClientMode::DryRun => Ok(()),
            ClientMode::ReadOnly => Err(ConfigError::IpcCommandFailed(
                "Client in read-only mode - cannot modify Hyprland".to_string(),
            )),
            ClientMode::Live => action(),
        }
    }

    /// The value part of a keyword command: everything after `bind... = `
    fn build_keyword_command(&self, _keyword: &str, binding: &Keybinding) -> String {
        let line = binding.to_string();
        line.split_once(" = ")
            .map(|(_, value)| value.to_string())
            .unwrap_or(line)
    }

    fn keyword(&self, keyword: &str, value: &str) -> Result<(), ConfigError> {
        self.send(&format!("keyword {keyword} {value}"))
    }

    /// Sends one command and expects `ok` back
    fn send(&self, command: &str) -> Result<(), ConfigError> {
        let reply = request(command)?;
        if reply.trim() == "ok" {
            Ok(())
        } else {
            Err(ConfigError::IpcCommandFailed(format!(
                "Hyprland answered: {}",
                reply.trim()
            )))
        }
    }
}

/// Sends one request to the running Hyprland instance and returns its reply
///
/// Fails with `HyprlandNotRunning` when the instance signature is not in the
/// environment or the socket cannot be connected.
pub fn request(command: &str) -> Result<String, ConfigError> {
    let path = socket_path()?;
    let mut stream = UnixStream::connect(&path).map_err(|e| {
        ConfigError::HyprlandNotRunning(format!("cannot connect to {}: {e}", path.display()))
    })?;
    stream
        .set_read_timeout(Some(TIMEOUT))
        .and_then(|_| stream.set_write_timeout(Some(TIMEOUT)))
        .map_err(|e| ConfigError::IpcCommandFailed(format!("socket setup failed: {e}")))?;

    stream
        .write_all(command.as_bytes())
        .map_err(|e| ConfigError::IpcCommandFailed(format!("failed to send command: {e}")))?;

    let mut reply = String::new();
    stream
        .read_to_string(&mut reply)
        .map_err(|e| ConfigError::IpcCommandFailed(format!("failed to read reply: {e}")))?;
    Ok(reply)
}

/// `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket.sock`
fn socket_path() -> Result<PathBuf, ConfigError> {
    let signature = env::var_os("HYPRLAND_INSTANCE_SIGNATURE").ok_or_else(|| {
        ConfigError::HyprlandNotRunning(
            "HYPRLAND_INSTANCE_SIGNATURE is not set; is Hyprland running?".to_string(),
        )
    })?;
    let runtime_dir = env::var_os("XDG_RUNTIME_DIR")
        .ok_or_else(|| ConfigError::HyprlandNotRunning("XDG_RUNTIME_DIR is not set".to_string()))?;
    Ok(PathBuf::from(runtime_dir)
        .join("hypr")
        .join(signature)
        .join(".socket.sock"))
}

#[cfg(test)]
mod tests;
