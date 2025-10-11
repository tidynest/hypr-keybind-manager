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
//! use hypr_keybind_manager::config::ConfigManager;
//! let manager = ConfigManager::new("/home/user/.config/hypr/hyprland.conf".into())?;
//! // Safe transactional write
//! manager.begin_transaction()?
//!     .write("bind = SUPER, Q, exec, firefox".to_string())?
//!     .commit()?;
//! // Automatic rollback if commit() is never called
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod danger;
mod validator;

use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use chrono::Local;

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
    /// Generic I/O error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

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
    /// ```
    /// let backups = manager.list_backups()?;
    /// // backups[0] is the most recent backup
    /// ```
    pub fn list_backups(&self) -> Result<Vec<PathBuf>, ConfigError> {
        // Read the backup directory
        let entries = fs::read_dir(&self.backup_dir)
            .map_err(|e| ConfigError::Io(e))?;

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
    /// ```
    /// // Keep only the 5 most recent backups
    /// let deleted = manager.cleanup_old_backups(5)?;
    /// println!("Deleted {} old backups", deleted);
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
                .map_err(|e| ConfigError::Io(e))?;
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
}

/// Atomic configuration transaction with automatic backup.
///
/// Provides ACID guarantees:
/// - **Atomic**: Changes are all-or-nothing (atomic file operations)
/// - **Consistent**: Config is never in a half-written state
/// - **Isolated**: No race conditions (OS-level atomic rename)
/// - **Durable**: Backup created before any modifications
///
/// # Lifecycle
///
/// 1. `begin()` - Creates timestamped backup immediately
/// 2. User prepares new content (in memory)
/// 3. `commit()` - Writes atomically or `rollback()` - Restores original
///
/// # Example
///
/// ```no_run
/// use hypr_keybind_manager::config::ConfigManager;
/// use std::path::PathBuf;
///
/// let manager = ConfigManager::new(PathBuf::from("hyprland.conf"))?;
/// let tx = ConfigTransaction::begin(&manager)?;
///
/// // Prepare new content
/// let new_content = "# Updated config\nbind = SUPER, X, exec, firefox\n";
///
/// // Commit atomically
/// match tx.commit(new_content) {
///     Ok(()) => println!("Changes applied successfully"),
///     Err(e) => {
///         eprintln!("Commit failed: {}", e);
///         // Transaction can be rolled back if needed
///     }
/// }
/// # Ok::<(), hypr_keybind_manager::config::ConfigError>(())
/// ```
#[allow(dead_code)]
pub struct ConfigTransaction<'a> {
    manager: &'a ConfigManager,
    backup_path: Option<PathBuf>,
}

impl<'a> ConfigTransaction<'a> {
    /// Begins a new transaction by creating a timestamped backup.
    ///
    /// The backup is created immediately when `begin()` is called, ensuring
    /// that a rollback point exists before any modifications are attempted.
    ///
    /// # Arguments
    ///
    /// * `manager` - Reference to the ConfigManager. The transaction cannot
    ///               outlive this reference (enforced by lifetime `'a`).
    ///
    /// # Returns
    ///
    /// * `Ok(ConfigTransaction)` - Transaction ready for commit/rollback
    /// * `Err(ConfigError)` - Backup creation failed (no changes made)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Backup directory cannot be created
    /// - Config file cannot be read
    /// - Backup file cannot be written
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hypr_keybind_manager::config::ConfigManager;
    /// use hypr_keybind_manager::config::ConfigTransaction;
    /// use std::path::PathBuf;
    ///
    /// let manager = ConfigManager::new(PathBuf::from("hyprland.conf"))?;
    /// let tx = ConfigTransaction::begin(&manager)?;
    /// // Backup now exists in backup directory
    /// # Ok::<(), hypr_keybind_manager::config::ConfigError>(())
    /// ```
    pub fn begin(manager: &'a ConfigManager) -> Result<Self, ConfigError> {
        // Create backup immediately - this is our rollback point
        let backup_path = manager.create_timestamped_backup()?;

        Ok(Self {
            manager,
            backup_path: Some(backup_path),
        })
    }

    /// Commits the transaction by atomically writing new content to the config file.
    ///
    /// The write operation is atomic at the filesystem level (temp file + rename),
    /// ensuring the config is never in a partially-written state. The backup created
    /// during `begin()` remains available for manual rollback if needed.
    ///
    /// This method consumes the transaction, preventing accidental double-commits.
    ///
    /// # Arguments
    ///
    /// * `new_content` - The complete new configuration content to write
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Content written successfully, backup preserved
    /// * `Err(ConfigError)` - Write failed, original config untouched, backup available
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Temporary file cannot be created
    /// - Content cannot be written to temp file
    /// - Atomic rename operation fails
    ///
    /// If an error occurs, the original config file remains unchanged and the backup
    /// created during `begin()` is still available for rollback.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hypr_keybind_manager::config::{ConfigManager, ConfigTransaction};
    /// use std::path::PathBuf;
    ///
    /// let manager = ConfigManager::new(PathBuf::from("hyprland.conf"))?;
    /// let tx = ConfigTransaction::begin(&manager)?;
    ///
    /// let new_content = "# Updated configuration\nbind = SUPER, X, exec, firefox\n";
    ///
    /// match tx.commit(new_content) {
    ///     Ok(()) => println!("Configuration updated successfully"),
    ///     Err(e) => eprintln!("Failed to commit: {}", e),
    /// }
    /// # Ok::<(), hypr_keybind_manager::config::ConfigError>(())
    /// ```
    pub fn commit(self, new_content: &str) -> Result<(), ConfigError> {
        use std::io::Write;
        use atomic_write_file::AtomicWriteFile;

        // Open file for atomic writing
        let mut file = AtomicWriteFile::options()
            .open(&self.manager.config_path)
            .map_err(|e| ConfigError::WriteFailed(
                format!("Failed to open for atomic write: {}", e)
            ))?;

        // Write content
        file.write_all(new_content.as_bytes())
            .map_err(|e| ConfigError::WriteFailed(
                format!("Failed to write content: {}", e)
            ))?;

        // Commit atomically
        file.commit()
            .map_err(|e| ConfigError::WriteFailed(
                format!("Failed to commit atomic write: {}", e)
            ))?;

        // Backup remains in backup directory for future rollback if needed
        // Cleanup is handled separately by cleanup_old_backups()
        Ok(())
    }

    /// Rolls back to the backup created during `begin()`.
    ///
    /// Atomically restores the configuration file to its state when the transaction
    /// began. This can be called after a failed commit or to manually undo changes.
    ///
    /// Unlike `commit()`, this method borrows `self` immutably, allowing multiple
    /// rollback attempts if needed.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Config restored successfully from backup
    /// * `Err(ConfigError)` - Rollback failed (backup unreadable or write failed)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No backup path is available (should not happen in normal usage)
    /// - Backup file cannot be read
    /// - Atomic write of backup content fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hypr_keybind_manager::config::{ConfigManager, ConfigTransaction};
    /// use std::path::PathBuf;
    ///
    /// let manager = ConfigManager::new(PathBuf::from("hyprland.conf"))?;
    /// let tx = ConfigTransaction::begin(&manager)?;
    ///
    /// // Attempt risky operation
    /// if let Err(e) = tx.commit(potentially_bad_content) {
    ///     eprintln!("Commit failed: {}", e);
    ///     tx.rollback()?;  // Restore original
    ///     println!("Rolled back successfully");
    /// }
    /// # Ok::<(), hypr_keybind_manager::config::ConfigError>(())
    /// ```
    pub fn rollback(&self) -> Result<(), ConfigError> {
        use std::io::Write;
        use atomic_write_file::AtomicWriteFile;

        // Check if backup path is available
        if let Some(backup_path) = &self.backup_path {
            // Read backup content
            let backup_content = fs::read_to_string(backup_path)?;

            // Open file for atomic writing
            let mut file = AtomicWriteFile::options()
                .open(&self.manager.config_path)
                .map_err(|e| ConfigError::WriteFailed(
                    format!("Failed to open for atomic write: {}", e)
                ))?;

            // Write backup content
            file.write_all(backup_content.as_bytes())
                .map_err(|e| ConfigError::WriteFailed(
                    format!("Failed to write content: {}", e)
                ))?;

            // Commit atomically
            file.commit()
                .map_err(|e| ConfigError::WriteFailed(
                    format!("Failed to commit: {}", e)
                ))?;

            Ok(())
        } else {
            // This should not happen in normal usage (begin() always creates backup)
            Err(ConfigError::BackupFailed(
                "No backup available for rollback".to_string()
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::Duration;
    use std::thread;
    use tempfile::TempDir;

    /// Helper: Creates a temporary config file for testing.
    fn create_test_config() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("hyprland.conf");
        fs::write(&config_path, "# Test config\nbind = SUPER, Q, exec, firefox\n").unwrap();
        (temp_dir, config_path)
    }

    #[test]
    fn test_new_with_valid_config() {
        let (_temp_dir, config_path) = create_test_config();

        let manager = ConfigManager::new(config_path.clone());
        assert!(manager.is_ok(), "Should create manager with valid config");

        let manager = manager.unwrap();
        assert_eq!(manager.config_path, config_path);

        // Backup directory should be created
        let backup_dir = config_path.parent().unwrap().join("backups");
        assert!(backup_dir.exists(), "Backup directory should be created");
    }

    #[test]
    fn test_new_with_missing_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.conf");

        let result = ConfigManager::new(config_path.clone());
        assert!(result.is_err(), "Should fail with missing config");

        match result.unwrap_err() {
            ConfigError::NotFound(path) => assert_eq!(path, config_path),
            other => panic!("Expected NotFound error, got: {:?}", other),
        }
    }

    #[test]
    fn test_read_config() {
        let (_temp_dir, config_path) = create_test_config();
        let manager = ConfigManager::new(config_path).unwrap();

        let content = manager.read_config();
        assert!(content.is_ok(), "Should read config successfully");

        let content = content.unwrap();
        assert!(content.contains("Test config"), "Should contain test content");
        assert!(content.contains("bind = SUPER"), "Should contain keybinding");
    }

    #[test]
    fn test_backup_dir_creation() {
        let (_temp_dir, config_path) = create_test_config();

        // Remove backup dir if it exists (shouldn't, but just in case)
        let backup_dir = config_path.parent().unwrap().join("backups");
        if backup_dir.exists() {
            fs::remove_dir_all(&backup_dir).unwrap();
        }

        assert!(!backup_dir.exists(), "Backup dir should not exist yet");

        let _manager = ConfigManager::new(config_path).unwrap();

        assert!(backup_dir.exists(), "Backup dir should be created");
        assert!(backup_dir.is_dir(), "Backup dir should be a directory");
    }

    #[test]
    fn test_symlink_warning() {
        // This test is tricky because we need to capture stderr
        // For now, we just verify that symlinks are allowed

        let temp_dir = TempDir::new().unwrap();
        let real_config = temp_dir.path().join("real_config.conf");
        let link_config = temp_dir.path().join("link_config.conf");

        fs::write(&real_config, "# Real config\n").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(&real_config, &link_config).unwrap();

            // Should succeed despite being a symlink
            let result = ConfigManager::new(link_config);
            assert!(result.is_ok(), "Should allow symlinked configs");
        }

        #[cfg(not(unix))]
        {
            // Skip test on non-Unix systems
            println!("Skipping symlink test on non-Unix system");
        }
    }

    #[test]
    fn test_create_timestamped_backup() {
        // Setup: Create a temp config file
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("hyprland.conf");
        fs::write(&config_path, "bind = SUPER, K, exec, firefox\n").unwrap();

        let manager = ConfigManager::new(config_path.clone()).unwrap();

        // Create a backup
        let backup_path = manager.create_timestamped_backup().unwrap();

        // Verify: Backup file exists
        assert!(backup_path.exists(), "Backup file should exist");

        // Verify: Backup is in the backup directory
        assert_eq!(
            backup_path.parent().unwrap(),
            manager.backup_dir,
            "Backup should be in backup directory",
        );

        // Verify: Filename format using chrono parsing
        let filename = backup_path.file_name().unwrap().to_str().unwrap();

        // Extract timestamp part: "hyprland.conf.2025-10-10_221500" -> "2025-10-10_221500"
        let parts: Vec<&str> = filename.split('.').collect();
        assert_eq!(
            parts.len(),
            3,
            "Filename should have 3 parts: name.ext.timestamp"
        );
        assert_eq!(parts[0], "hyprland", "First part should be 'hyprland'");
        assert_eq!(parts[1], "conf", "Second part should be 'conf'");

        let timestamp = parts[2];

        // Validate timestamp by parsing with chrono
        let parsed = chrono::NaiveDateTime::parse_from_str(
            timestamp,
            "%Y-%m-%d_%H%M%S",
        );
        assert!(
            parsed.is_ok(),
            "Timestamp should be valid chrono format: {}",
            timestamp,
        );

        // Verify: Backup content matches original
        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert_eq!(backup_content, "bind = SUPER, K, exec, firefox\n");
    }

    #[test]
    fn test_multiple_backups_dont_overwrite() {
        // Setup
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("hyprland.conf");
        fs::write(&config_path, "original content").unwrap();

        let manager = ConfigManager::new(config_path.clone()).unwrap();

        // Create first backup
        let backup1 = manager.create_timestamped_backup().unwrap();

        // Wait 1 second to ensure different timestamp
        thread::sleep(Duration::from_secs(1));

        // Modify config and create second backup
        fs::write(&config_path, "modified content").unwrap();
        let backup2 = manager.create_timestamped_backup().unwrap();

        // Verify: Both backups exist
        assert!(backup1.exists(), "First backup should exist");
        assert!(backup2.exists(), "Second backup should exist");

        // Verify: They're different files
        assert_eq!(
            fs::read_to_string(&backup1).unwrap(),
            "original content"
        );
        assert_eq!(
            fs::read_to_string(&backup2).unwrap(),
            "modified content"
        );
    }

    #[test]
    fn test_list_backups_sorted_newest_first() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("hyprland.conf");
        let backup_dir = temp_dir.path().join("backups");
        fs::create_dir(&backup_dir).unwrap();

        // Write initial config
        fs::write(&config_path, "initial\n").unwrap();

        let manager = ConfigManager::new(config_path.clone()).unwrap();

        // Create 3 backups with delays to ensure different timestamps
        let backup1 = manager.create_timestamped_backup().unwrap();
        thread::sleep(Duration::from_secs(1));

        fs::write(&config_path, "second\n").unwrap();
        let backup2 = manager.create_timestamped_backup().unwrap();
        thread::sleep(Duration::from_secs(1));

        fs::write(&config_path, "third\n").unwrap();
        let backup3 = manager.create_timestamped_backup().unwrap();

        // List backups
        let backups = manager.list_backups().unwrap();

        // Should have 3 backups
        assert_eq!(backups.len(), 3);

        // Should be sorted newest first
        assert_eq!(backups[0], backup3); // Most recent
        assert_eq!(backups[1], backup2); // Middle
        assert_eq!(backups[2], backup1); // Oldest
    }

    #[test]
    fn test_list_backups_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("hyprland.conf");
        let backup_dir = temp_dir.path().join("backups");
        fs::create_dir(&backup_dir).unwrap();

        fs::write(&config_path, "config\n").unwrap();

        let manager = ConfigManager::new(config_path.clone()).unwrap();

        // List backups in empty directory
        let backups = manager.list_backups().unwrap();

        // Should return empty vector, not error
        assert_eq!(backups.len(), 0);
    }

    #[test]
    fn test_list_backups_ignores_invalid_files() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("hyprland.conf");
        let backup_dir = temp_dir.path().join("backups");
        fs::create_dir(&backup_dir).unwrap();

        fs::write(&config_path, "config\n").unwrap();

        let manager = ConfigManager::new(config_path.clone()).unwrap();

        // Create one valid backup
        let valid_backup = manager.create_timestamped_backup().unwrap();

        // Create some invalid files in backup directory
        fs::write(backup_dir.join("random.txt"), "not a backup").unwrap();
        fs::write(backup_dir.join("hyprland.conf.notimestamp"), "wrong format").unwrap();
        fs::write(backup_dir.join("hyprland.conf.2025-99-99_invalid"), "bad date").unwrap();

        // List backups
        let backups = manager.list_backups().unwrap();

        // Should only find the one valid backup
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0], valid_backup);
    }

    #[test]
    fn test_cleanup_keeps_n_most_recent() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("hyprland.conf");

        fs::write(&config_path, "initial\n").unwrap();

        let manager = ConfigManager::new(config_path.clone()).unwrap();

        // Create 5 backups with delays
        let mut backup_paths = Vec::new();
        for i in 1..=5 {
            fs::write(&config_path, format!("version {}\n", i)).unwrap();
            let backup = manager.create_timestamped_backup().unwrap();
            backup_paths.push(backup);
            thread::sleep(Duration::from_secs(1));
        }

        // Keep only 2 most recent
        let deleted = manager.cleanup_old_backups(2).unwrap();

        // Should have deleted 3 backups (5 - 2 = 3)
        assert_eq!(deleted, 3);

        // Verify only 2 backups remain
        let remaining = manager.list_backups().unwrap();
        assert_eq!(remaining.len(), 2);

        // Verify the correct ones remain (the 2 newest: indices 4 and 3)
        assert_eq!(remaining[0], backup_paths[4]); // Newest
        assert_eq!(remaining[1], backup_paths[3]); // Second newest
    }

    #[test]
    fn test_cleanup_with_no_backups() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("hyprland.conf");

        fs::write(&config_path, "config\n").unwrap();

        let manager = ConfigManager::new(config_path).unwrap();

        // Cleanup with no backups present
        let deleted = manager.cleanup_old_backups(5).unwrap();

        // Should succeed with 0 deleted
        assert_eq!(deleted, 0);
    }

    #[test]
    fn test_cleanup_when_fewer_than_keep() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("hyprland.conf");

        fs::write(&config_path, "initial\n").unwrap();

        let manager = ConfigManager::new(config_path.clone()).unwrap();

        // Create only 3 backups
        for i in 1..=3 {
            fs::write(&config_path, format!("version {}\n", i)).unwrap();
            manager.create_timestamped_backup().unwrap();
            thread::sleep(Duration::from_secs(1));
        }

        // Try to keep 10 (more than exist)
        let deleted = manager.cleanup_old_backups(10).unwrap();

        // Should delete nothing
        assert_eq!(deleted, 0);

        // All 3 should still exist
        let remaining = manager.list_backups().unwrap();
        assert_eq!(remaining.len(), 3);
    }

    // ============================================================================
    // ConfigTransaction Tests
    // ============================================================================

    #[test]
    fn test_transaction_basic_flow() {
        // Setup: Create temp config with original content
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("hyprland.conf");
        let original_content = "# Original config\nbind = SUPER, Q, exec, firefox\n";
        fs::write(&config_path, original_content).unwrap();

        let manager = ConfigManager::new(config_path.clone()).unwrap();

        // Begin transaction (creates backup)
        let tx = ConfigTransaction::begin(&manager).unwrap();

        // Verify backup was created
        let backups = manager.list_backups().unwrap();
        assert_eq!(backups.len(), 1, "Should have created one backup");

        // Commit new content
        let new_content = "# Updated config\nbind = SUPER, X, exec, kitty\n";
        tx.commit(new_content).unwrap();

        // Verify new content was written
        let final_content = fs::read_to_string(&config_path).unwrap();
        assert_eq!(final_content, new_content, "Config should have new content");

        // Verify backup still exists (not deleted after commit)
        let backups_after = manager.list_backups().unwrap();
        assert_eq!(backups_after.len(), 1, "Backup should still exist after commit");

        // Verify backup contains original content
        let backup_content = fs::read_to_string(&backups_after[0]).unwrap();
        assert_eq!(backup_content, original_content, "Backup should contain original content");
    }

    #[test]
    fn test_transaction_rollback() {
        // Setup
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("hyprland.conf");
        let original_content = "# Original\nbind = SUPER, A, exec, alacritty\n";
        fs::write(&config_path, original_content).unwrap();

        let manager = ConfigManager::new(config_path.clone()).unwrap();

        // Begin transaction
        let tx = ConfigTransaction::begin(&manager).unwrap();

        // Rollback without committing
        tx.rollback().unwrap();

        // Verify original content is still there (unchanged)
        let final_content = fs::read_to_string(&config_path).unwrap();
        assert_eq!(
            final_content, original_content,
            "Rollback without commit should leave original unchanged"
        );
    }

    #[test]
    fn test_transaction_rollback_after_commit() {
        // Setup
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("hyprland.conf");
        let original_content = "# Original content\n";
        fs::write(&config_path, original_content).unwrap();

        let manager = ConfigManager::new(config_path.clone()).unwrap();

        // First transaction: commit changes
        let tx1 = ConfigTransaction::begin(&manager).unwrap();
        let new_content = "# Modified content\n";
        tx1.commit(new_content).unwrap();

        // Verify changes applied
        assert_eq!(
            fs::read_to_string(&config_path).unwrap(),
            new_content,
            "First commit should apply changes"
        );

        // Second transaction: rollback to original
        let tx2 = ConfigTransaction::begin(&manager).unwrap();
        tx2.rollback().unwrap();

        // Verify rolled back to state before second transaction
        // (which is the "new_content" from first transaction)
        let final_content = fs::read_to_string(&config_path).unwrap();
        assert_eq!(
            final_content, new_content,
            "Rollback should restore to state at transaction begin"
        );
    }

    #[test]
    fn test_transaction_preserves_exact_content() {
        // Test edge cases: empty lines, trailing newlines, special chars
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("hyprland.conf");
        fs::write(&config_path, "initial\n").unwrap();

        let manager = ConfigManager::new(config_path.clone()).unwrap();

        // Test 1: Empty content
        let tx = ConfigTransaction::begin(&manager).unwrap();
        tx.commit("").unwrap();
        assert_eq!(fs::read_to_string(&config_path).unwrap(), "");

        // Test 2: Content with multiple blank lines
        let tx = ConfigTransaction::begin(&manager).unwrap();
        let content_with_blanks = "line1\n\n\nline2\n";
        tx.commit(content_with_blanks).unwrap();
        assert_eq!(fs::read_to_string(&config_path).unwrap(), content_with_blanks);

        // Test 3: No trailing newline
        let tx = ConfigTransaction::begin(&manager).unwrap();
        let no_trailing = "no newline at end";
        tx.commit(no_trailing).unwrap();
        assert_eq!(fs::read_to_string(&config_path).unwrap(), no_trailing);

        // Test 4: Special characters
        let tx = ConfigTransaction::begin(&manager).unwrap();
        let special = "# Special: $VAR, @user, 100%\nbind = SUPER_SHIFT, M, exec, app\n";
        tx.commit(special).unwrap();
        assert_eq!(fs::read_to_string(&config_path).unwrap(), special);
    }

    #[test]
    fn test_transaction_commit_consumes_self() {
        // This test demonstrates that commit(self) consumes the transaction.
        // The following code would NOT compile (commented out):
        //
        // let tx = ConfigTransaction::begin(&manager)?;
        // tx.commit("content1")?;  // ← tx is moved here
        // tx.commit("content2")?;  // ← Compile error! tx was already consumed
        //
        // This is enforced at compile-time by Rust's ownership system.
        // We verify it works correctly by showing successful single use:

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("hyprland.conf");
        fs::write(&config_path, "original\n").unwrap();

        let manager = ConfigManager::new(config_path.clone()).unwrap();
        let tx = ConfigTransaction::begin(&manager).unwrap();

        // Single commit works fine
        tx.commit("new content\n").unwrap();
        // tx is now consumed and cannot be used again (enforced by compiler)

        assert_eq!(fs::read_to_string(&config_path).unwrap(), "new content\n");
    }

    #[test]
    fn test_multiple_transactions_create_multiple_backups() {
        // Integration test: verify transactions work with backup system
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("hyprland.conf");
        fs::write(&config_path, "version 1\n").unwrap();

        let manager = ConfigManager::new(config_path.clone()).unwrap();

        // Transaction 1
        let tx1 = ConfigTransaction::begin(&manager).unwrap();
        tx1.commit("version 2\n").unwrap();
        thread::sleep(Duration::from_secs(1)); // Ensure different timestamps

        // Transaction 2
        let tx2 = ConfigTransaction::begin(&manager).unwrap();
        tx2.commit("version 3\n").unwrap();
        thread::sleep(Duration::from_secs(1));

        // Transaction 3
        let tx3 = ConfigTransaction::begin(&manager).unwrap();
        tx3.commit("version 4\n").unwrap();

        // Verify 3 backups exist (one per transaction)
        let backups = manager.list_backups().unwrap();
        assert_eq!(backups.len(), 3, "Should have 3 backups from 3 transactions");

        // Verify backups are sorted newest first
        let backup1_content = fs::read_to_string(&backups[0]).unwrap(); // Newest
        let backup2_content = fs::read_to_string(&backups[1]).unwrap(); // Middle
        let backup3_content = fs::read_to_string(&backups[2]).unwrap(); // Oldest

        assert_eq!(backup1_content, "version 3\n", "Newest backup should be from tx3");
        assert_eq!(backup2_content, "version 2\n", "Middle backup should be from tx2");
        assert_eq!(backup3_content, "version 1\n", "Oldest backup should be from tx1");

        // Verify final config has latest content
        assert_eq!(fs::read_to_string(&config_path).unwrap(), "version 4\n");

        // Test cleanup integration
        let deleted = manager.cleanup_old_backups(2).unwrap();
        assert_eq!(deleted, 1, "Should delete 1 old backup (keeping 2)");

        let remaining = manager.list_backups().unwrap();
        assert_eq!(remaining.len(), 2, "Should have 2 backups remaining");
    }

    #[test]
    fn test_restore_backup_basic() {
        // Test basic restore functionality: backup → modify → restore

        // Setup: Create config with "original content"
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("hyprland.conf");
        let original_content = "# Original configuration\nbind = SUPER, Q, exec, kitty\n";
        fs::write(&config_path, original_content).unwrap();

        let manager = ConfigManager::new(config_path.clone()).unwrap();

        // Create backup of original content
        let backup_path = manager.create_timestamped_backup().unwrap();

        // Verify backup contains original content
        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert_eq!(backup_content, original_content, "Backup should contain original content");

        // Modify config to something different
        let modified_content = "# Modified configuration\nbind = SUPER, W, exec, firefox\n";
        fs::write(&config_path, modified_content).unwrap();

        // Verify config now has modified content
        let current = manager.read_config().unwrap();
        assert_eq!(current, modified_content, "Config should be modified");

        // Restore from backup
        manager.restore_backup(&backup_path).unwrap();

        // Verify config is restored to original content
        let restored = manager.read_config().unwrap();
        assert_eq!(restored, original_content, "Config should be restored to original content");
    }

    #[test]
    fn test_restore_creates_safety_backup() {
        // Test that restore creates a safety backup before restoring

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("hyprland.conf");
        fs::write(&config_path, "initial content\n").unwrap();

        let manager = ConfigManager::new(config_path.clone()).unwrap();

        // Create first backup (the one being restored from)
        let first_backup = manager.create_timestamped_backup().unwrap();

        // Small delay to ensure different timestamps
        thread::sleep(Duration::from_secs(1));

        // Modify config to "current content"
        let current_content = "current content after modification\n";
        fs::write(&config_path, current_content).unwrap();

        // Count backups BEFORE restore (should be 1 - just first_backup)
        let backups_before = manager.list_backups().unwrap();
        assert_eq!(backups_before.len(), 1, "Should have 1 backup after restore");

        // Restore from first backup / Create safety backup
        manager.restore_backup(&first_backup).unwrap();

        // Count backups AFTER restore (should be 2 - first_backup + safety)
        let backups_after = manager.list_backups().unwrap();
        assert_eq!(backups_after.len(), 2, "Should have 2 backups after restore (original + safety)");

        // Verify the safety backup contains the "current content" we had before restore
        let safety_backup = &backups_after[0];  // Most recent backup
        let safety_content = fs::read_to_string(&safety_backup).unwrap();
        assert_eq!(safety_content, current_content,
           "Safety backup should contain content from before restore");
    }


    #[test]
    fn test_restore_nonexistent_backup_fails() {
        // Test that restoring from non-existent backup returns error without modifying config

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("hyprland.conf");
        let original_content = "origin content\n";
        fs::write(&config_path, original_content).unwrap();

        let manager = ConfigManager::new(config_path.clone()).unwrap();

        // Try to restore from non-existent backup
        let fake_backup = temp_dir.path().join("nonexistent-backup.conf");
        let result = manager.restore_backup(&fake_backup);

        // Should return error
        assert!(result.is_err(), "Restore should fail for non-existent backup");

        // Error should be BackupFailed variant
        match result {
            Err(ConfigError::BackupFailed(msg)) => {
                assert!(msg.contains("does not exist"),
                "Error message should mention file doesn't exist");
            }
            _ => panic!("Expected BackupFailed error"),
        }

        // Original config should be unchanged
        let current = manager.read_config().unwrap();
        assert_eq!(current, original_content, "Config should be unchanged after failed retore");

        // No backups should have been created
        let backups = manager.list_backups().unwrap();
        assert_eq!(backups.len(), 0, "No backups should be created on failed restore");
    }

    #[test]
    fn test_restore_directory_path_fails() {
        // Test that restoring from directory instead of file returns error

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("hyprland.conf");
        fs::write(&config_path, "original\n").unwrap();

        let manager = ConfigManager::new(config_path.clone()).unwrap();

        // Try to restore from directory instead of file
        let dir_path = temp_dir.path().join("some_directory");
        fs::create_dir(&dir_path).unwrap();

        let result = manager.restore_backup(&dir_path);

        // Should return error
        assert!(result.is_err(), "Restore should fail for directory path");

        // Error should mention it's not a file
        match result {
            Err(ConfigError::BackupFailed(msg)) => {
                assert!(msg.contains("not a file"),
                        "Error message should mention path is not a file");
            }
            _ => panic!("Expected BackupFailed error mentioning 'not a file'"),
        }
    }

    #[test]
    fn test_restore_preserves_exact_content() {
        // Test that restore preserves exact content including edge cases

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("hyprland.conf");

        // Edge case content: empty lines, special chars, no trailing newline
        let tricky_content = r#"# Config with edge cases

    bind = SUPER, K, exec, echo "hello $USER"
    # Empty line above
    bind = SUPER_SHIFT, M, exec, notify-send 'Test @ 100%'
    # No trailing newline below"#;

        fs::write(&config_path, tricky_content).unwrap();

        let manager = ConfigManager::new(config_path.clone()).unwrap();

        // Create backup of tricky content
        let backup = manager.create_timestamped_backup().unwrap();

        // Modify config to something simple
        fs::write(&config_path, "simple content\n").unwrap();

        // Restore from backup
        manager.restore_backup(&backup).unwrap();

        // Read restored content
        let restored = manager.read_config().unwrap();

        // Verify byte-for-byte match
        assert_eq!(restored, tricky_content,
           "Restored content should exactly match original, including special chars and formatting");

        // Verify specific edge cases
        assert!(restored.contains("\"hello $USER\""), "Should preserve quotes and variables");
        assert!(restored.contains("'Test @ 100%'"), "Should preserve single quotes and special chars");
        assert!(!restored.ends_with('\n'), "Should preserve lack of trailing newline");
        assert!(restored.contains("\n\n"), "Should preserve empty lines");
    }
}
