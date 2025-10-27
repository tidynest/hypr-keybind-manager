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

//! Configuration file management with atomic writes and backup support.
//! This module provides safe, transactional operations for managing Hyprland
//! configuration files. Key features:
//! - **Atomic writes**: Uses temp-file-then-rename to prevent corruption
//! - **Automatic backups**: Every write creates a timestamped backup
//! - **Rollback safety**: Failed transactions leave original config untouched
//! - **Symlink warnings**: Alerts user but allows symlinked configs
//!
//! # Example
//! ```no_run
//! use hypr_keybind_manager::config::{ConfigManager, ConfigTransaction};
//! use std::path::PathBuf;
//!
//! let manager = ConfigManager::new(PathBuf::from("/home/user/.config/hypr/hyprland.conf"))?;
//!
//! // Safe transactional write
//! let tx = ConfigTransaction::begin(&manager)?;
//! tx.commit("bind = SUPER, Q, exec, firefox\n")?;
//!
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod danger;
pub mod error;
pub mod transaction;
pub mod validator;

pub use error::ConfigError;
pub use transaction::ConfigTransaction;

use chrono::Local;
use std::{fs, path::{Path, PathBuf}};

use crate::Keybinding;

/// Manages Hyprland configuration files with safe atomic operations.
/// The ConfigManager provides read-only access and transactional writes
/// with automatic backup creation. All writes go through the transaction
/// API to ensure atomicity and recoverability.
#[allow(dead_code)]
#[derive(Debug)]
pub struct ConfigManager {
    /// Path to the Hyprland configuration file.
    config_path: PathBuf,
    backup_dir: PathBuf,
}

impl ConfigManager {
    /// Creates a new ConfigManager for the given config file.
    ///
    /// This validates that the config file exists and creates the backup
    /// directory if it doesn't exist. If the config is a symlink, a warning
    /// is printed to stderr but the operation continues.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the Hyprland configuration file
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::NotFound` if the config file doesn't exist.
    /// Returns `ConfigError::BackupDirNotWritable` if the backup directory
    /// cannot be created.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hypr_keybind_manager::config::ConfigManager;
    /// use std::path::PathBuf;
    ///
    /// let manager = ConfigManager::new(
    ///     PathBuf::from("/home/user/.config/hypr/hyprland.conf")
    /// )?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(config_path: PathBuf) -> Result<Self, ConfigError> {
        // Validate config file exists
        if !config_path.exists() {
            return Err(ConfigError::NotFound(config_path));
        }

        // Warn if config is a symlink (but allow it per user preference)
        if config_path.read_link().is_ok() {
            eprintln!("⚠ Warning: Config file is a symlink: {}", config_path.display());
            eprintln!("  This is allowed, but be aware of what it points to.");
        }

        // Create backup directory next to config file
        // e.g., ~/.config/hypr/hyprland.conf → ~/.config/hypr/backups/
        let backup_dir = config_path
            .parent()
            .ok_or_else(|| {
                ConfigError::BackupDirNotWritable(
                    PathBuf::from("Config file has no parent directory")
                )
            })?
            .join("backups");

        // Create backup directory if it doesn't exist
        if !backup_dir.exists() {
            fs::create_dir_all(&backup_dir).map_err(|_| {
                ConfigError::BackupDirNotWritable(backup_dir.clone())
            })?;
        }

        // Verify backup directory is writable
        if backup_dir.metadata()?.permissions().readonly() {
            return Err(ConfigError::BackupDirNotWritable(backup_dir));
        }

        Ok(Self {
            config_path,
            backup_dir,
        })
    }

    /// Reads the current configuration file content.
    ///
    /// This is a read-only operation that does not require a transaction
    /// or permission validation.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::Io` if the file cannot be read.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use hypr_keybind_manager::config::ConfigManager;
    /// # let manager = ConfigManager::new("/home/user/.config/hypr/hyprland.conf".into())?;
    /// let content = manager.read_config()?;
    /// println!("Config has {} lines", content.lines().count());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn read_config(&self) -> Result<String, ConfigError> {
        Ok(fs::read_to_string(&self.config_path)?)
    }

    /// Returns a reference to the configuration file path
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    #[allow(dead_code)]
    fn create_timestamped_backup(&self) -> Result<PathBuf, ConfigError> {
        // Read the current config content
        let content = fs::read_to_string(&self.config_path)?;

        // Generate timestamp in YYYY-MM-DD_HHMMSS format
        let timestamp = Local::now().format("%Y-%m-%d_%H%M%S");

        // Build the backup filename
        // Extract the original filename (e.g., "hyprland.conf")
        let original_name = self.config_path
            .file_name()
            .expect("Config path should have a file name")
            .to_str()
            .expect("Filename should be valid UTF-8");

        let backup_filename = format!("{}.{}", original_name, timestamp);
        let backup_path = self.backup_dir.join(&backup_filename);

        // Write the backup file
        fs::write(&backup_path, &content)?;

        // Return the path so caller can verify or log it
        Ok(backup_path)
    }

    /// Lists all backups in the backup directory, sorted newest first.
    ///
    /// Parses timestamps from filenames matching the pattern:
    /// `{basename}.YYYY-MM-DD_HHMMSS`
    ///
    /// Invalid files (wrong format, unparseable timestamps) are silently skipped.
    ///
    /// # Returns
    /// - `Ok(Vec<PathBuf>)` - Backup paths sorted newest to oldest
    /// - `Err(ConfigError)` - If directory cannot be read
    ///
    /// # Examples
    /// ```no_run
    /// # use hypr_keybind_manager::config::ConfigManager;
    /// # use std::path::PathBuf;
    /// # let manager = ConfigManager::new(PathBuf::from("hyprland.conf"))?;
    /// let backups = manager.list_backups();
    /// // backups[0] is the most recent backup
    /// # Ok::<(), hypr_keybind_manager::config::ConfigError>(())
    /// ```
    pub fn list_backups(&self) -> Result<Vec<PathBuf>, ConfigError> {
        // Read the backup directory
        let entries = fs::read_dir(&self.backup_dir)
            .map_err(ConfigError::Io)?;

        // Collect valid backups with their timestamps
        let mut backups: Vec<(PathBuf, chrono::NaiveDateTime)> = Vec::new();

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue, // Skip unreadable entries
            };

            let path = entry.path();

            // Only process files (not directories)
            if !path.is_file() {
                continue;
            }

            // Extract filename
            let filename = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name,
                None => continue, // Skip if filename is invalid UTF-8
            };

            // Parse the filename: expect "basename.ext.YYYY-MM-DD_HHMMSS"
            let parts: Vec<&str> = filename.split('.').collect();
            if parts.len() != 3 {
                continue; // Not a valid backup filename
            }

            // Extract and parse the timestamp (3rd part)
            let timestamp_str = parts[2];
            match chrono::NaiveDateTime::parse_from_str(
                timestamp_str,
                "%Y-%m-%d_%H%M%S"
            ) {
                Ok(timestamp) => {
                    backups.push((path, timestamp));
                }
                Err(_) => continue, // Invalid timestamp format, skip this file
            }
        }

        // Sort by timestamp, newest first (descending order)
        backups.sort_by(|a, b| b.1.cmp(&a.1));

        // Extract just the paths (discard timestamps)
        Ok(backups.into_iter().map(|(path, _)| path).collect())
    }

    /// Deletes old backups, keeping only the N most recent.
    ///
    /// Uses `list_backups()` to discover and sort backups, then deletes
    /// all except the first N entries (most recent).
    ///
    /// # Arguments
    /// - `keep` - Number of most recent backups to preserve
    ///
    /// # Returns
    /// - `Ok(usize)` - Number of backups deleted
    /// - `Err(ConfigError)` - If listing or deletion fails
    ///
    /// # Examples
    /// ```no_run
    /// # use hypr_keybind_manager::config::ConfigManager;
    /// # use std::path::PathBuf;
    /// # let manager = ConfigManager::new(PathBuf::from("hyprland.conf"))?;
    /// // Keep only the 5 most recent backups
    /// let deleted = manager.cleanup_old_backups(5)?;
    /// println!("Deleted {} old backups", deleted);
    /// # Ok::<(), hypr_keybind_manager::config::ConfigError>(())
    /// ```
    pub fn cleanup_old_backups(&self, keep: usize) -> Result<usize, ConfigError> {
        // Get sorted list of backups (newest first)
        let backups = self.list_backups()?;

        // Determine which backups to delete
        let to_delete = if backups.len() > keep {
            &backups[keep..] // Everything after index `keep`
        } else {
            &[] // Nothing to delete (fewer backups than keep limit)
        };

        // Delete the old backups
        let mut deleted_count = 0;
        for backup_path in to_delete {
            fs::remove_file(backup_path)
                .map_err(ConfigError::Io)?;
            deleted_count += 1;
        }

        Ok(deleted_count)
    }

    /// Restores the configuration from a specific backup file.
    ///
    /// This function performs a safe restore operation by:
    /// 1. Creating a safety backup of the current state
    /// 2. Atomically restoring from the specified backup
    ///
    /// If the restore operation fails at any point, the original config
    /// remains unchanged. The safety backup allows reverting a successful
    /// restore if needed.
    ///
    /// # Arguments
    ///
    /// * `backup_path` - Path to the backup file to restore from
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Config successfully restored
    /// * `Err(ConfigError::BackupFailed)` - Backup path invalid or unreadable
    /// * `Err(ConfigError::WriteFailed)` - Atomic write operation failed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hypr_keybind_manager::config::ConfigManager;
    /// use std::path::PathBuf;
    ///
    /// let manager = ConfigManager::new(PathBuf::from("hyprland.conf"))?;
    ///
    /// // List backups to find the one you want
    /// let backups = manager.list_backups()?;
    ///
    /// // Restore from the most recent backup
    /// if let Some(latest) = backups.first() {
    ///     manager.restore_backup(latest)?;
    ///     println!("Config restored successfully!");
    /// }
    /// # Ok::<(), hypr_keybind_manager::config::ConfigError>(())
    /// ```
    ///
    /// # Safety
    ///
    /// This function creates a safety backup before restoring, so you can
    /// always revert the restore operation by restoring from the safety backup.
    pub fn restore_backup(&self, backup_path: &Path) -> Result<(), ConfigError> {
        use std::io::Write;
        use atomic_write_file::AtomicWriteFile;

        // Step 1: Validate backup exists and is readable
        if !backup_path.exists() {
            return Err(ConfigError::BackupFailed(
                format!("Backup file does not exist: {}", backup_path.display())
            ));
        }

        if !backup_path.is_file() {
            return Err(ConfigError::BackupFailed(
                format!("Backup path is not a file: {}", backup_path.display())
            ));
        }

        // Step 2: Read content from the specified backup FIRST
        // (Do this before creating safety backup to ensure backup is readable)
        let backup_content = fs::read_to_string(backup_path)
            .map_err(|e| ConfigError::BackupFailed(
                format!("Failed to read backup file: {}", e)
            ))?;

        // Step 3: Create safety backup of CURRENT state
        // This allows undoing the restore if needed
        let _safety_backup = self.create_timestamped_backup()?;

        // Step 4: Atomically write backup content to config file
        let mut file = AtomicWriteFile::options()
            .open(&self.config_path)
            .map_err(|e| ConfigError::WriteFailed(
                format!("Failed to open config for restore: {}", e)
            ))?;

        file.write_all(backup_content.as_bytes())
            .map_err(|e| ConfigError::WriteFailed(
                format!("Failed to write restored content: {}", e)
            ))?;

        file.commit()
            .map_err(|e| ConfigError::WriteFailed(
                format!("Failed to commit restore: {}", e)
            ))?;

        Ok(())
    }

    /// Writes keybindings back to the configuration file
    ///
    /// Creates an automatic backup via the transaction system before writing.
    /// Preserves comments, blank lines, and non-keybinding configuration.
    ///
    /// # Arguments
    /// * `bindings` - The complete list of keybindings to write
    ///
    /// # Errors
    /// Returns `ConfigError` if:
    /// - File cannot be read
    /// - Backup creation fails
    /// - File cannot be written
    ///
    /// # Example
    /// ```no_run
    /// # use hypr_keybind_manager::config::ConfigManager;
    /// # use hypr_keybind_manager::core::Keybinding;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut manager = ConfigManager::new("~/.config/hypr/hyprland.conf".into())?;
    /// let bindings = vec![/* your modified bindings */];
    ///
    /// manager.write_bindings(&bindings)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_bindings(&mut self, bindings: &[Keybinding]) -> Result<(), ConfigError> {
        // Read current config to preserve non-keybinding content
        let original_content = self.read_config()?;

        // Rebuild config with updated keybindings
        let new_content = self.rebuild_config(&original_content, bindings)?;

        // Write atomically via transaction (creates backup automatically)
        let transaction = ConfigTransaction::begin(self)?;
        transaction.commit(&new_content)?;

        Ok(())
    }

    /// Exports keybindings to a specified file path
    ///
    /// Creates a new config file containing only keybinding (no preservation of other content)
    pub fn export_to(&self, export_path: &Path, bindings: &[Keybinding]) -> Result<(), ConfigError> {
        let mut content = String::from("# Exported Hyprland Keybindings\n\n");

        for binding in bindings {
            content.push_str(&self.format_binding(binding));
            content.push('\n');
        }

        fs::write(export_path, content)?;

        Ok(())
    }

    /// Rebuilds config file, replacing keybinding lines whilst preserving everything else
    ///
    /// This is the "smart" part - we identify the keybinding section and replace only that,
    /// keeping comments, blank lines, and other settings intact.
    ///
    /// # Strategy
    /// 1. Scan through original line by line
    /// 2. When we hit the first keybinding line, mark that position
    /// 3. Skip all subsequent keybinding lines
    /// 4. At the end of the keybinding section, insert our new bindings
    /// 5. Continue with the rest of the file
    ///
    /// # Arguments
    /// * `original` - Original config file content
    /// * `bindings` - New keybindings to write
    ///
    /// # Returns
    /// The rebuilt config as a string
    fn rebuild_config(&self, original: &str, bindings: &[Keybinding]) -> Result<String, ConfigError> {
        let mut result = String::new();
        let mut in_keybinding_section = false;
        let mut keybindings_written = false;

        for line in original.lines() {
            let trimmed = line.trim();

            // Check if this is a keybinding line
            let is_keybinding = trimmed.starts_with("bind")
                && !trimmed.starts_with("#")
                && (trimmed.starts_with("bind =")
                    || trimmed.starts_with("binde =")
                    || trimmed.starts_with("bindl =")
                    || trimmed.starts_with("bindm =")
                    || trimmed.starts_with("bindr =")
                    || trimmed.starts_with("bindel ="));

            if is_keybinding {
                // Keybinding section has been reached
                if !in_keybinding_section {
                    in_keybinding_section = true;
                }

                // Skip this line - new bindings will be written at the end of the section
                continue;
            }

            // If we're in keybinding section but hit a non-keybinding line, write our bindings now
            if in_keybinding_section && !keybindings_written {
                for binding in bindings {
                    result.push_str(&self.format_binding(binding));
                    result.push('\n');
                }
                keybindings_written = true;
                in_keybinding_section = false;
            }

            // Preserve all non-keybinding lines
            result.push_str(line);
            result.push('\n');
        }

        // If we never found a keybinding section, or we're still in it at EOF, write bindings now
        if !keybindings_written {
            result.push_str("\n# Keybindings\n");
            for binding in bindings {
                result.push_str(&self.format_binding(binding));
                result.push('\n');
            }
        }

        Ok(result)
    }

    /// Formats a keybinding into a config file line
    ///
    /// Example output: `bind = SUPER, K, exec, firefox`
    ///
    /// # Arguments
    /// * `binding` - The keybinding to format
    ///
    /// # Returns
    /// A formatted config line (without trailing newline)
    fn format_binding(&self, binding: &Keybinding) -> String {
        use crate::core::types::Modifier::*;
        // Build a modifier string
        let modifiers_str = if binding.key_combo.modifiers.is_empty() {
            String::new()
        } else {
            binding.key_combo.modifiers
                .iter()
                .map(|m| match m {
                    Super => "SUPER",
                    Ctrl => "CTRL",
                    Shift => "SHIFT",
                    Alt => "ALT",
                })
                .collect::<Vec<_>>()
                .join("_")
        };

        // Build the parts that will be comma-separated
        let mut parts = Vec::new();

        // Add modifiers and key
        if !modifiers_str.is_empty() {
            parts.push(modifiers_str);
        } else {
            // No modifiers - just key
            parts.push(String::new());
        }

        // Add key
        parts.push(binding.key_combo.key.clone());

        // Add dispatcher
        parts.push(binding.dispatcher.clone());

        // Add args if present
        if let Some(args) = &binding.args {
            parts.push(args.clone());
        }

        // Format: bind_type = comma,separated,parts
        // Example: bind = SUPER, K, exec, firefox
        format!("{} = {}", binding.bind_type, parts.join(", "))
    }
}

#[cfg(test)]
mod tests;