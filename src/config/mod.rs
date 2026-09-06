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

pub use {error::ConfigError, transaction::ConfigTransaction};

use atomic_write_file::AtomicWriteFile;
use chrono::Local;
#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use crate::core::{
    parser::{LineKind, parse_config_tree, scan_lines},
    types::Keybinding,
};

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
            eprintln!(
                "⚠ Warning: Config file is a symlink: {}",
                config_path.display()
            );
            eprintln!("  This is allowed, but be aware of what it points to.");
        }

        for warning in Self::permission_warnings(&config_path) {
            eprintln!("⚠ Warning: {warning}");
        }

        // Create backup directory next to config file
        // e.g., ~/.config/hypr/hyprland.conf → ~/.config/hypr/backups/
        let backup_dir = config_path
            .parent()
            .ok_or_else(|| {
                ConfigError::BackupDirNotWritable(PathBuf::from(
                    "Config file has no parent directory",
                ))
            })?
            .join("backups");

        // Create backup directory if it doesn't exist
        if !backup_dir.exists() {
            fs::create_dir_all(&backup_dir)
                .map_err(|_| ConfigError::BackupDirNotWritable(backup_dir.clone()))?;
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

    fn permission_warnings(config_path: &Path) -> Vec<String> {
        #[cfg(unix)]
        {
            let metadata = match fs::metadata(config_path) {
                Ok(metadata) => metadata,
                Err(_) => return Vec::new(),
            };

            Self::permission_warnings_for_metadata(config_path, &metadata, current_uid())
        }

        #[cfg(not(unix))]
        {
            let _ = config_path;
            Vec::new()
        }
    }

    #[cfg(unix)]
    fn permission_warnings_for_metadata(
        config_path: &Path,
        metadata: &fs::Metadata,
        expected_uid: Option<u32>,
    ) -> Vec<String> {
        let mut warnings = Vec::new();
        let mode = metadata.permissions().mode();

        if mode & 0o004 != 0 {
            warnings.push(format!(
                "Config file is world-readable: {} (mode {:o})",
                config_path.display(),
                mode & 0o777
            ));
        }

        if mode & 0o002 != 0 {
            warnings.push(format!(
                "Config file is world-writable: {} (mode {:o})",
                config_path.display(),
                mode & 0o777
            ));
        }

        if let Some(expected_uid) = expected_uid {
            let actual_uid = metadata.uid();
            if actual_uid != expected_uid {
                warnings.push(format!(
                    "Config file is owned by uid {} instead of current uid {}: {}",
                    actual_uid,
                    expected_uid,
                    config_path.display()
                ));
            }
        }

        warnings
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
        let original_name = self
            .config_path
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
        let entries = fs::read_dir(&self.backup_dir).map_err(ConfigError::Io)?;

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
            match chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d_%H%M%S") {
                Ok(timestamp) => {
                    backups.push((path, timestamp));
                }
                Err(_) => continue, // Invalid timestamp format, skip this file
            }
        }

        // Sort by timestamp, newest first (descending order)
        backups.sort_by_key(|(_, timestamp)| std::cmp::Reverse(*timestamp));

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
            fs::remove_file(backup_path).map_err(ConfigError::Io)?;
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
        // Step 1: Validate backup exists and is readable
        if !backup_path.exists() {
            return Err(ConfigError::BackupFailed(format!(
                "Backup file does not exist: {}",
                backup_path.display()
            )));
        }

        if !backup_path.is_file() {
            return Err(ConfigError::BackupFailed(format!(
                "Backup path is not a file: {}",
                backup_path.display()
            )));
        }

        // Step 2: Read content from the specified backup FIRST
        // (Do this before creating safety backup to ensure backup is readable)
        let backup_content = fs::read_to_string(backup_path)
            .map_err(|e| ConfigError::BackupFailed(format!("Failed to read backup file: {}", e)))?;

        // Step 3: Create safety backup of CURRENT state
        // This allows undoing the restore if needed
        let _safety_backup = self.create_timestamped_backup()?;

        // Step 4: Atomically write backup content to config file
        let mut file = AtomicWriteFile::options()
            .open(&self.config_path)
            .map_err(|e| {
                ConfigError::WriteFailed(format!("Failed to open config for restore: {}", e))
            })?;

        file.write_all(backup_content.as_bytes()).map_err(|e| {
            ConfigError::WriteFailed(format!("Failed to write restored content: {}", e))
        })?;

        file.commit()
            .map_err(|e| ConfigError::WriteFailed(format!("Failed to commit restore: {}", e)))?;

        Ok(())
    }

    /// Writes keybindings to the config file, changing only the lines that changed
    ///
    /// The current file (and every file it sources) is scanned line by line.
    /// A binding that is no longer in `bindings` has its line replaced by a
    /// new binding in the same submap when one is waiting, or removed. New
    /// bindings that found no line to take over are appended after the last
    /// bind line of their submap in the main file. Comments, variables,
    /// settings and the order of untouched bindings stay as they were.
    ///
    /// # Errors
    /// Returns `ConfigError` if a file cannot be read or parsed, a backup
    /// cannot be created, or a file cannot be written.
    ///
    /// # Example
    /// ```no_run
    /// # use hypr_keybind_manager::{config::ConfigManager, core::Keybinding};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut manager = ConfigManager::new("~/.config/hypr/hyprland.conf".into())?;
    /// let bindings = vec![/* your modified bindings */];
    ///
    /// manager.write_bindings(&bindings)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_bindings(&mut self, bindings: &[Keybinding]) -> Result<(), ConfigError> {
        let content = self.read_config()?;
        let tree = parse_config_tree(&content, &self.config_path)
            .map_err(|e| ConfigError::ValidationFailed(e.to_string()))?;

        // ponytail: O(n·m) multiset diff, bindings lists are a few hundred entries at most
        let mut removed = tree.bindings.clone();
        let mut added = Vec::new();
        for binding in bindings {
            match removed.iter().position(|r| r == binding) {
                Some(i) => {
                    removed.remove(i);
                }
                None => added.push(binding.clone()),
            }
        }
        if removed.is_empty() && added.is_empty() {
            return Ok(());
        }

        // Main file first so its removed lines are consumed before sourced
        // files are looked at; whatever is still new lands in the main file.
        let mut main = rewrite_file(&content, &tree.variables, &mut removed, &mut added);
        for file in tree.files.iter().skip(1) {
            let original = fs::read_to_string(file)?;
            let rewritten = rewrite_file(&original, &tree.variables, &mut removed, &mut added);
            if rewritten.changed {
                let manager = ConfigManager::new(file.clone())?;
                ConfigTransaction::begin(&manager)?.commit(&rewritten.render(&original))?;
            }
        }
        if !added.is_empty() {
            main.append(added, &tree.variables);
        }
        if main.changed {
            ConfigTransaction::begin(self)?.commit(&main.render(&content))?;
        }

        Ok(())
    }

    /// Exports keybindings to a specified file path
    ///
    /// Creates a new config file containing only keybindings (no preservation of other content).
    /// Bindings are grouped under `submap = name` sections where needed.
    pub fn export_to(
        &self,
        export_path: &Path,
        bindings: &[Keybinding],
    ) -> Result<(), ConfigError> {
        let mut content = String::from("# Exported Hyprland Keybindings\n\n");
        let mut current_submap: Option<&str> = None;

        for binding in bindings {
            if binding.submap.as_deref() != current_submap {
                current_submap = binding.submap.as_deref();
                content.push_str(&format!("submap = {}\n", current_submap.unwrap_or("reset")));
            }
            content.push_str(&binding.to_string());
            content.push('\n');
        }
        if current_submap.is_some() {
            content.push_str("submap = reset\n");
        }

        fs::write(export_path, content)?;

        Ok(())
    }
}

/// One file's lines after the in-place pass, plus where each submap's last bind line is
struct Rewritten {
    lines: Vec<String>,
    /// Index in `lines` of the last bind line per submap, `None` = global map
    last_bind_line: HashMap<Option<String>, usize>,
    changed: bool,
}

impl Rewritten {
    /// Adds bindings that found no line to take over
    ///
    /// Each goes after the last bind line of its submap. Submaps not present
    /// in the file get a fresh `submap = name` ... `submap = reset` block at
    /// the end, under a `# Keybindings` header.
    fn append(&mut self, added: Vec<Keybinding>, variables: &HashMap<String, String>) {
        let mut inserts: HashMap<usize, Vec<String>> = HashMap::new();
        let mut tail: Vec<String> = Vec::new();
        let mut tail_submap: Option<String> = None;

        for binding in added {
            let rendered = binding.to_config_line(variables);
            match self.last_bind_line.get(&binding.submap) {
                Some(&index) => inserts.entry(index).or_default().push(rendered),
                None => {
                    if tail.is_empty() {
                        tail.push(String::new());
                        tail.push("# Keybindings".to_string());
                    }
                    if binding.submap != tail_submap {
                        tail.push(format!(
                            "submap = {}",
                            binding.submap.as_deref().unwrap_or("reset")
                        ));
                        tail_submap = binding.submap.clone();
                    }
                    tail.push(rendered);
                }
            }
        }
        if tail_submap.is_some() {
            tail.push("submap = reset".to_string());
        }

        let old = std::mem::take(&mut self.lines);
        for (index, line) in old.into_iter().enumerate() {
            self.lines.push(line);
            if let Some(extra) = inserts.remove(&index) {
                self.lines.extend(extra);
            }
        }
        self.lines.extend(tail);
        self.changed = true;
    }

    /// Joins the lines back, keeping a trailing newline when the original had one
    fn render(&self, original: &str) -> String {
        let mut result = self.lines.join("\n");
        if original.ends_with('\n') || original.is_empty() {
            result.push('\n');
        }
        result
    }
}

/// Rewrites one file's bind lines in place
///
/// Removed bindings are consumed from `removed` as their lines are met. Each
/// such line is replaced by the first waiting entry of `added` in the same
/// submap, so an edit stays where it was and a new binding may take the slot
/// of a deleted one. Untouched lines are copied as they are.
fn rewrite_file(
    original: &str,
    variables: &HashMap<String, String>,
    removed: &mut Vec<Keybinding>,
    added: &mut Vec<Keybinding>,
) -> Rewritten {
    let mut vars = variables.clone();
    let scanned = scan_lines(original, &mut vars);
    let mut out = Rewritten {
        lines: Vec::with_capacity(scanned.len()),
        last_bind_line: HashMap::new(),
        changed: false,
    };

    for (line, (_, kind)) in original.lines().zip(scanned) {
        let Ok(LineKind::Binding(binding)) = kind else {
            out.lines.push(line.to_string());
            continue;
        };
        if let Some(i) = removed.iter().position(|r| *r == binding) {
            removed.remove(i);
            out.changed = true;
            let Some(j) = added.iter().position(|a| a.submap == binding.submap) else {
                continue;
            };
            let replacement = added.remove(j);
            let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
            out.lines
                .push(format!("{indent}{}", replacement.to_config_line(&vars)));
        } else {
            out.lines.push(line.to_string());
        }
        out.last_bind_line
            .insert(binding.submap.clone(), out.lines.len() - 1);
    }

    out
}

#[cfg(unix)]
fn current_uid() -> Option<u32> {
    fs::metadata("/proc/self")
        .ok()
        .map(|metadata| metadata.uid())
}

#[cfg(test)]
mod tests;
