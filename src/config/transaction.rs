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

//! Configuration transaction management with automatic backups
//!
//! Provides atomic write operations with ACID guarantees.

use atomic_write_file::AtomicWriteFile;
use std::{fs, io::Write, path::PathBuf};

use crate::config::{
    danger,
    validator::{
        ConfigValidator,
        ValidationLevel::{Error, Warning},
    },
    ConfigError, ConfigManager,
};

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
/// use hypr_keybind_manager::config::{ConfigManager, ConfigTransaction};
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
    ///   outlive this reference (enforced by lifetime `'a`).
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
    /// use hypr_keybind_manager::config::{ConfigManager, ConfigTransaction};
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

    /// Commits the transaction with comprehensive validation
    ///
    /// This method validates the config before committing:
    /// 1. Runs all validation layers (injection + danger detection)
    /// 2. Blocks on Error-level issues (Layer 1 injection, Layer 2 critical)
    /// 3. Warns on Warning-level issues (Layer 2 suspicious/dangerous)
    /// 4. Commits if validation passes
    ///
    /// # Arguments
    ///
    /// * `new_content` - The complete new configuration content to write
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Content validated and written successfully
    /// * `Err(ConfigError::ValidationFailed)` - Layer 1 injection detected
    /// * `Err(ConfigError::DangerousCommand)` - Layer 2 critical danger detected
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
    /// let new_content = "bind = SUPER, K, exec, firefox\n";
    ///
    /// match tx.commit_with_validation(new_content) {
    ///     Ok(()) => println!("✓ Configuration updated successfully"),
    ///     Err(e) => eprintln!("✗ Commit blocked: {}", e),
    /// }
    /// # Ok::<(), hypr_keybind_manager::config::ConfigError>(())
    /// ```
    pub fn commit_with_validation(self, new_content: &str) -> Result<(), ConfigError> {
        // Step 1: Run comprehensive validation
        let validator = ConfigValidator::new();
        let report = validator.validate_config(new_content);

        // Step 2: Block on errors (Layer 1: Injection)
        if report.has_errors() {
            let error_count = report
                .issues
                .iter()
                .filter(|i| i.validation_level == Error)
                .count();

            eprintln!("\n❌ VALIDATION FAILED:\n");
            for issue in &report.issues {
                if issue.validation_level == Error {
                    eprintln!("  Binding {}: {}", issue.binding_index, issue.message);
                }
            }
            eprintln!("\nThis configuration will NOT be committed.");
            eprintln!(
                "Fix the {} error(s) above before proceeding.\n",
                error_count
            );

            return Err(ConfigError::ValidationFailed(format!(
                "{} validation error(s) detected",
                error_count
            )));
        }

        // Step 3: Block on critical dangers (Layer 2: System destruction)
        if report.has_critical_dangers() {
            eprintln!("\n⚠️  CRITICAL DANGER DETECTED:\n");
            for (binding_idx, danger) in &report.dangerous_commands {
                if danger.danger_level == danger::DangerLevel::Critical {
                    eprintln!("  Binding {}: {}", binding_idx, danger.reason);
                    eprintln!("  Recommendation: {}\n", danger.recommendation);
                }
            }
            eprintln!("This configuration will NOT be committed.");
            eprintln!("Remove dangerous commands before proceeding.\n");

            return Err(ConfigError::DangerousCommand(
                "Critical danger detected - commit blocked.".to_string(),
            ));
        }

        // Step 4: Show warnings, but allow commit (Layer 2: Suspicious/Dangerous but not Critical
        let warnings = report
            .issues
            .iter()
            .filter(|i| i.validation_level == Warning)
            .collect::<Vec<_>>();

        if !warnings.is_empty() {
            eprintln!("\n⚠️  Configuration Warnings:\n");
            for issue in &warnings {
                eprintln!("  Binding {}: {}", issue.binding_index, issue.message);
                if let Some(suggestion) = &issue.suggestion {
                    eprintln!("   Suggestion: {}", suggestion);
                }
            }
            eprintln!("\nProceeding with commit (warnings are informational).\n");
        }

        // Step 5: All checks passed. Proceed with atomic commit
        self.commit(new_content)
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
        // Open file for atomic writing
        let mut file = AtomicWriteFile::options()
            .open(&self.manager.config_path)
            .map_err(|e| {
                ConfigError::WriteFailed(format!("Failed to open for atomic write: {}", e))
            })?;

        // Write content
        file.write_all(new_content.as_bytes())
            .map_err(|e| ConfigError::WriteFailed(format!("Failed to write content: {}", e)))?;

        // Commit atomically
        file.commit().map_err(|e| {
            ConfigError::WriteFailed(format!("Failed to commit atomic write: {}", e))
        })?;

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
    /// // Decide not to proceed with changes
    /// // (e.g., user cancelled, validation failed, etc.)
    /// println!("Rolling back - changes abandoned");
    /// tx.rollback()?;  // Restore original
    ///
    /// # Ok::<(), hypr_keybind_manager::config::ConfigError>(())
    /// ```
    pub fn rollback(&self) -> Result<(), ConfigError> {
        // Check if backup path is available
        if let Some(backup_path) = &self.backup_path {
            // Read backup content
            let backup_content = fs::read_to_string(backup_path)?;

            // Open file for atomic writing
            let mut file = AtomicWriteFile::options()
                .open(&self.manager.config_path)
                .map_err(|e| {
                    ConfigError::WriteFailed(format!("Failed to open for atomic write: {}", e))
                })?;

            // Write backup content
            file.write_all(backup_content.as_bytes())
                .map_err(|e| ConfigError::WriteFailed(format!("Failed to write content: {}", e)))?;

            // Commit atomically
            file.commit()
                .map_err(|e| ConfigError::WriteFailed(format!("Failed to commit: {}", e)))?;

            Ok(())
        } else {
            // This should not happen in normal usage (begin() always creates backup)
            Err(ConfigError::BackupFailed(
                "No backup available for rollback".to_string(),
            ))
        }
    }
}
