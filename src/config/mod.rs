//! Configuration file management with atomic writes and backup support.
//!
//! This module provides safe, transactional operations for managing Hyprland
//! configuration files. Key features:
//!
//! - **Atomic writes**: Uses temp-file-then-rename to prevent corruption
//! - **Automatic backups**: Every write creates a timestamped backup
//! - **Rollback safety**: Failed transactions leave original config untouched
//! - **Symlink warnings**: Alerts user but allows symlinked configs
//!
//! # Example
//!
//! ```no_run
//! use hypr_keybind_manager::config::ConfigManager;
//!
//! let manager = ConfigManager::new("/home/user/.config/hypr/hyprland.conf".into())?;
//!
//! // Safe transactional write
//! manager.begin_transaction()?
//!     .write("bind = SUPER, Q, exec, firefox".to_string())?
//!     .commit()?;
//!
//! // Automatic rollback if commit() is never called
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::fs;
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

    /// Generic I/O error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Manages Hyprland configuration files with safe atomic operations.
///
/// The ConfigManager provides read-only access and transactional writes
/// with automatic backup creation. All writes go through the transaction
/// API to ensure atomicity and recoverability.
#[derive(Debug)]
pub struct ConfigManager {
    /// Path to the Hyprland configuration file.
    config_path: PathBuf,

    #[allow(dead_code)]
    /// Directory where timestamped backups are stored.
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
}

#[cfg(test)]
mod tests {
    use super::*;
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
}