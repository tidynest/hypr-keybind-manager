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

//! Configuration error types

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during configuration management.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Configuration file does not exist.
    #[error("Config file not found: {0}")]
    NotFound(PathBuf),
    /// Backup directory cannot be created or written to.
    #[error("Backup directory not writable: {0}")]
    BackupDirNotWritable(PathBuf),
    /// Configuration file has incorrect permissions (should be 0o600).
    #[error("Invalid permissions on config: expected 0o600, found {0:o}")]
    InvalidPermissions(u32),
    /// Attempted to commit a transaction twice.
    #[error("Transaction already committed")]
    AlreadyCommitted,
    /// Failed to create backup file.
    #[error("Failed to create backup: {0}")]
    BackupFailed(String),
    /// Atomic write operation failed.
    #[error("Atomic write failed: {0}")]
    WriteFailed(String),
    /// Configuration file does not exist.
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    /// Dangerous command detected
    #[error("Dangerous command detected: {0}")]
    DangerousCommand(String),
    /// Generic I/O error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Hyprland IPC connection failed
    #[error("Failed to connect to Hyprland IPC: {0}")]
    IpcConnectionFailed(String),
    /// Hyprland is not running or socket not found
    #[error("Hyprland not running (socket not found: {0})")]
    HyprlandNotRunning(String),
    /// IPC command was sent but Hyprland returned an error
    #[error("Hyprland command failed: {0}")]
    IpcCommandFailed(String),
    /// Failed to write file to path
    #[error("Failed to write to path: {0}")]
    WriteError(PathBuf),
}
