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

//! MVC Controller - Mediates between Model (ConfigManager) and View (GTK4 components)
//!
//! # Responsibilities
//!
//! - Load keybindings from ConfigManager
//! - Filter/search keybindings
//! - Detect conflicts using ConflictDetector
//! - Validate commands using ConfigValidator
//! - Provide data to View in UI-friendly format
//!
//! # Architecture
//!
//! The Controller holds references to Model components but doesn't know
//! about GTK4 widgets. This keeps business logic separate from presentation.

use std::{
    cell::RefCell,
    fs,
    fs::read_to_string,
    path::{Path, PathBuf},
    process::Command,
    rc::Rc,
};

use crate::config::{validator::ConfigValidator, ConfigError, ConfigManager};
use crate::core::{
    parser::parse_config_file, validator as injection_validator, Conflict, ConflictDetector,
    KeyCombo, Keybinding, Modifier,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ImportMode {
    /// Replace all existing bindings with imported ones
    Replace,
    /// Merge imported bindings with existing (skip duplicates)
    Merge,
}

#[derive(Clone, Debug, PartialEq)]
pub enum KeyComboAvailability {
    Incomplete,
    Available,
    InUse(Vec<Keybinding>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct KeyComboAssistance {
    pub availability: KeyComboAvailability,
    pub suggestions: Vec<KeyCombo>,
}

/// MVC Controller coordinating Model and View
///
/// Holds shared references to Model components and provides
/// methods for View to query/manipulate data.
pub struct Controller {
    /// Configuration file manager (shared mutable reference)
    config_manager: Rc<RefCell<ConfigManager>>,
    /// Current list of loaded keybindings
    keybindings: RefCell<Vec<Keybinding>>,
    /// Conflict detector (rebuild when keybindings change)
    conflict_detector: RefCell<ConflictDetector>,
    /// Current search query (for preserving filters state)
    current_search_query: RefCell<String>,
    /// Undo history of complete binding snapshots
    undo_stack: RefCell<Vec<Vec<Keybinding>>>,
    /// Redo history of complete binding snapshots
    redo_stack: RefCell<Vec<Vec<Keybinding>>>,
}

const HISTORY_LIMIT: usize = 20;

impl Controller {
    /// Creates a new Controller with the given config file path
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to Hyprland configuration file
    ///
    /// # Returns
    ///
    /// * `Ok(Controller)` - Successfully initialised
    /// * `Err(ConfigError)` - Config file not found or unreadable
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hypr_keybind_manager::ui::Controller;
    /// use std::path::PathBuf;
    ///
    /// let controller = Controller::new(
    ///     PathBuf::from("~/.config/hypr/hyprland.conf")
    /// )?;
    /// # Ok::<(), hypr_keybind_manager::config::ConfigError>(())
    /// ```
    pub fn new(config_path: PathBuf) -> Result<Self, ConfigError> {
        // Create ConfigManager
        let config_manager = ConfigManager::new(config_path)?;
        let config_manager = Rc::new(RefCell::new(config_manager));

        // Creates empty Controller (data loaded later via load_keybindings)
        Ok(Self {
            config_manager,
            keybindings: RefCell::new(Vec::new()),
            conflict_detector: RefCell::new(ConflictDetector::new()),
            current_search_query: RefCell::new(String::new()),
            undo_stack: RefCell::new(Vec::new()),
            redo_stack: RefCell::new(Vec::new()),
        })
    }

    /// Gets the config file path
    pub fn config_path(&self) -> PathBuf {
        self.config_manager.borrow().config_path().to_path_buf()
    }

    /// Loads keybindings from config file
    ///
    /// This reads the config file, parses all keybindings, and rebuilds
    /// the conflict detector. Call this on startup and after config changes.
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - Number of keybindings loaded
    /// * `Err(ConfigError)` - Failed to read or parse config
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use hypr_keybind_manager::ui::Controller;
    /// # use std::path::PathBuf;
    /// # let controller = Controller::new(PathBuf::from("hyprland.conf"))?;
    /// let count = controller.load_keybindings()?;
    /// println!("Loaded {} keybindings", count);
    /// # Ok::<(), hypr_keybind_manager::config::ConfigError>(())
    /// ```
    pub fn load_keybindings(&self) -> Result<usize, ConfigError> {
        // Read config content from ConfigManager
        let config_manager = self.config_manager.borrow();
        let content = config_manager.read_config()?;

        // Parse keybindings using existing parser
        let bindings = parse_config_file(&content, Path::new(""))
            .map_err(|e| ConfigError::ValidationFailed(e.to_string()))?;

        let count = bindings.len();

        // Store keybindings
        *self.keybindings.borrow_mut() = bindings.clone();

        // Rebuild conflict detector
        let mut detector = ConflictDetector::new();
        for binding in bindings {
            detector.add_binding(binding);
        }
        *self.conflict_detector.borrow_mut() = detector;

        Ok(count)
    }

    fn record_undo_snapshot(&self) {
        let snapshot = self.keybindings.borrow().clone();
        let mut undo_stack = self.undo_stack.borrow_mut();
        undo_stack.push(snapshot);
        if undo_stack.len() > HISTORY_LIMIT {
            undo_stack.remove(0);
        }
        self.redo_stack.borrow_mut().clear();
    }

    fn rebuild_conflict_detector_from_bindings(bindings: &[Keybinding]) -> ConflictDetector {
        let mut detector = ConflictDetector::new();
        for binding in bindings {
            detector.add_binding(binding.clone());
        }
        detector
    }

    fn write_snapshot(&self, bindings: &[Keybinding]) -> Result<(), String> {
        self.config_manager
            .borrow_mut()
            .write_bindings(bindings)
            .map_err(|e| format!("Failed to write changes to config: {}", e))
    }

    fn replace_bindings(&self, new_bindings: Vec<Keybinding>) {
        let detector = Self::rebuild_conflict_detector_from_bindings(&new_bindings);
        *self.keybindings.borrow_mut() = new_bindings;
        *self.conflict_detector.borrow_mut() = detector;
    }

    pub fn clear_history(&self) {
        self.undo_stack.borrow_mut().clear();
        self.redo_stack.borrow_mut().clear();
    }

    /// Returns all loaded keybindings
    ///
    /// # Returns
    ///
    /// A clone of the keybinding list (cheap, uses Rc internally)
    pub fn get_keybindings(&self) -> Vec<Keybinding> {
        self.keybindings.borrow().clone()
    }

    /// Filters keybindings by search query
    ///
    /// Searches in:
    /// - Key combination (e.g., "SUPER+K")
    /// - Dispatcher name (e.g., "exec")
    /// - Arguments (e.g., "firefox")
    ///
    /// Search is case-insensitive.
    ///
    /// # Arguments
    ///
    /// * `query` - Search term (empty = return all)
    ///
    /// # Returns
    ///
    /// Filtered list of keybindings matching query
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use hypr_keybind_manager::ui::Controller;
    /// # use std::path::PathBuf;
    /// # let controller = Controller::new(PathBuf::from("hyprland.conf"))?;
    /// # controller.load_keybindings()?;
    /// // Find all bindings with "firefox"
    /// let firefox_bindings = controller.filter_keybindings("firefox");
    /// # Ok::<(), hypr_keybind_manager::config::ConfigError>(())
    /// ```
    pub fn filter_keybindings(&self, query: &str) -> Vec<Keybinding> {
        // Empty query returns all bindings
        if query.trim().is_empty() {
            return self.get_keybindings();
        }

        let query_lower = query.to_lowercase();

        self.keybindings
            .borrow()
            .iter()
            .filter(|binding| {
                // Search in the key combo
                let key_combo_str = format!("{}", binding.key_combo).to_lowercase();
                if key_combo_str.contains(&query_lower) {
                    return true;
                }

                // Search in dispatcher
                if binding.dispatcher.to_lowercase().contains(&query_lower) {
                    return true;
                }

                // Search in args
                if let Some(args) = &binding.args {
                    if args.to_lowercase().contains(&query_lower) {
                        return true;
                    }
                }

                false
            })
            .cloned()
            .collect()
    }

    /// Updates the current search query
    ///
    /// This method stores the search query in the Controller's state.
    /// Call this whenever the user types in the search box.
    ///
    /// # Arguments
    ///
    /// * `query` - The new search query text
    pub fn set_search_query(&self, query: String) {
        *self.current_search_query.borrow_mut() = query;
    }

    /// Gets the current search query
    ///
    /// # Returns
    ///
    /// The currently active search query string
    pub fn get_search_query(&self) -> String {
        self.current_search_query.borrow().clone()
    }

    /// Returns the current view of keybindings (respecting active search filter)
    ///
    /// If a search query is active, returns filtered results.
    /// If no search query, returns all keybindings.
    ///
    /// # Returns
    ///
    /// The keybindings that should currently be displayed in the UI
    pub fn get_current_view(&self) -> Vec<Keybinding> {
        let query = self.current_search_query.borrow().clone();
        self.filter_keybindings(&query)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.borrow().is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.borrow().is_empty()
    }

    /// Returns bindings currently using the provided key combo.
    ///
    /// When `exclude` is set, that exact binding is ignored. This is used by the
    /// edit dialog so a binding does not report itself as a conflict.
    pub fn get_bindings_for_key_combo(
        &self,
        key_combo: &KeyCombo,
        exclude: Option<&Keybinding>,
    ) -> Vec<Keybinding> {
        self.keybindings
            .borrow()
            .iter()
            .filter(|binding| binding.key_combo == *key_combo)
            .filter(|binding| exclude != Some(*binding))
            .cloned()
            .collect()
    }

    /// Returns whether the given combo is free to use.
    pub fn is_key_combo_available(
        &self,
        key_combo: &KeyCombo,
        exclude: Option<&Keybinding>,
    ) -> bool {
        self.get_bindings_for_key_combo(key_combo, exclude).is_empty()
    }

    /// Builds inline assistance data for the edit dialog.
    pub fn get_key_combo_assistance(
        &self,
        key_combo: Option<&KeyCombo>,
        exclude: Option<&Keybinding>,
    ) -> KeyComboAssistance {
        let Some(key_combo) = key_combo else {
            return KeyComboAssistance {
                availability: KeyComboAvailability::Incomplete,
                suggestions: Vec::new(),
            };
        };

        let in_use = self.get_bindings_for_key_combo(key_combo, exclude);
        if in_use.is_empty() {
            KeyComboAssistance {
                availability: KeyComboAvailability::Available,
                suggestions: Vec::new(),
            }
        } else {
            KeyComboAssistance {
                availability: KeyComboAvailability::InUse(in_use),
                suggestions: self.suggest_key_combos(&key_combo.modifiers, exclude, 5, key_combo),
            }
        }
    }

    /// Suggests nearby free combos using the same modifier set.
    pub fn suggest_key_combos(
        &self,
        modifiers: &[Modifier],
        exclude: Option<&Keybinding>,
        limit: usize,
        original: &KeyCombo,
    ) -> Vec<KeyCombo> {
        let modifiers = modifiers.to_vec();

        candidate_keys()
            .into_iter()
            .map(|key| KeyCombo::new(modifiers.clone(), key))
            .filter(|candidate| candidate != original)
            .filter(|candidate| self.is_key_combo_available(candidate, exclude))
            .take(limit)
            .collect()
    }

    /// Returns all detected conflicts
    ///
    /// A conflict occurs when multiple keybindings use the same
    /// key combination (e.g., both bind SUPER+K).
    ///
    /// # Returns
    ///
    /// List of conflicts detected by ConflictDetector
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use hypr_keybind_manager::ui::Controller;
    /// # use std::path::PathBuf;
    /// # let controller = Controller::new(PathBuf::from("hyprland.conf"))?;
    /// # controller.load_keybindings()?;
    /// let conflicts = controller.get_conflicts();
    /// println!("Found {} conflicts", conflicts.len());
    /// # Ok::<(), hypr_keybind_manager::config::ConfigError>(())
    /// ```
    pub fn get_conflicts(&self) -> Vec<Conflict> {
        self.conflict_detector.borrow().find_conflicts()
    }

    /// Validates a keybinding using all security layers
    ///
    /// Runs:
    /// - Layer 1: Injection prevention (core/validator.rs)
    /// - Layer 2: Danger detection (config/danger)
    /// - Layer 3: Config validation (config/validator.rs)
    ///
    /// # Arguments
    ///
    /// * `binding` - Keybinding to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Binding is safe
    /// * `Err(String)` - Validation failed with reason
    pub fn validate_keybinding(&self, binding: &Keybinding) -> Result<(), String> {
        // Layer 1: Injection check
        injection_validator::validate_keybinding(binding)
            .map_err(|e| format!("Security violation: {}", e))?;

        // Layer 2 & 3: Danger + config validation
        let validator = ConfigValidator::new();
        let binding_str = format!("{}", binding);
        let report = validator.validate_config(&binding_str);

        if report.has_errors() {
            return Err("Validation errors detected".to_string());
        }

        if report.has_critical_dangers() {
            return Err("Critical dangers detected".to_string());
        }

        Ok(())
    }

    /// Returns total count of loaded keybindings
    pub fn keybinding_count(&self) -> usize {
        self.keybindings.borrow().len()
    }

    /// Returns count of detected conflicts
    pub fn conflict_count(&self) -> usize {
        self.get_conflicts().len()
    }

    /// Deletes a keybinding and writes changes to disk
    ///
    /// This removes the binding from the in-memory list and immediately
    /// writes the updated list back to the config file, creating a backup.
    ///
    /// # Arguments
    /// * `binding` - The keybinding to delete
    ///
    /// # Returns
    /// * `Ok(())` - Successfully deleted and saved
    /// * `Err(String)` - Failed to write changes
    ///
    /// # Example
    /// ```no_run
    /// # use hypr_keybind_manager::{
    ///       core::{Keybinding, KeyCombo, Modifier, BindType},
    ///       ui::Controller
    ///   };
    ///
    /// # use std::path::PathBuf;
    ///
    /// # fn main() -> Result<(), String> {
    /// let controller = Controller::new(PathBuf::from("test.conf"))
    ///     .map_err(|e| e.to_string())?;
    ///
    /// let binding = Keybinding {
    ///     key_combo: KeyCombo::new(vec![Modifier::Super], "K"),
    ///     bind_type: BindType::Bind,
    ///     dispatcher: "exec".to_string(),
    ///     args: Some("firefox".to_string()),
    /// };
    ///
    /// controller.delete_keybinding(&binding)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn delete_keybinding(&self, binding: &Keybinding) -> Result<(), String> {
        self.record_undo_snapshot();
        let mut bindings = self.keybindings.borrow_mut();
        bindings.retain(|b| b != binding);
        let updated_bindings = bindings.clone();
        drop(bindings);

        if let Err(e) = self.write_snapshot(&updated_bindings) {
            let previous = self.undo_stack.borrow_mut().pop();
            if let Some(previous) = previous {
                self.replace_bindings(previous);
            }
            return Err(e);
        }

        self.replace_bindings(updated_bindings);

        Ok(())
    }

    /// Adds a new keybinding to the configuration
    ///
    /// This method:
    /// 1. Adds the binding to the in-memory list
    /// 2. Writes changes to disk (creates automatic backup)
    /// 3. Rebuilds the conflict detector
    ///
    /// # Arguments
    /// * `binding` - The new keybinding to add
    ///
    /// # Returns
    /// * `Ok(())` if successful
    /// * `Err(String)` with error message if operation fails
    ///
    /// # Example
    /// ```ignore
    /// match controller.add_keybinding(new_binding) {
    ///     Ok(()) => println!("Keybinding added successfully"),
    ///     Err(e) => eprintln!("Failed to add: {}", e),
    /// }
    /// ```
    pub fn add_keybinding(&self, binding: Keybinding) -> Result<(), String> {
        self.record_undo_snapshot();
        let mut bindings = self.keybindings.borrow_mut();
        bindings.push(binding.clone());
        let updated_bindings = bindings.clone();
        drop(bindings);

        if let Err(e) = self.write_snapshot(&updated_bindings) {
            let previous = self.undo_stack.borrow_mut().pop();
            if let Some(previous) = previous {
                self.replace_bindings(previous);
            }
            return Err(e);
        }

        self.replace_bindings(updated_bindings);

        Ok(())
    }

    /// Lists all available backup files, sorted newest first
    pub fn list_backups(&self) -> Result<Vec<PathBuf>, String> {
        self.config_manager
            .borrow()
            .list_backups()
            .map_err(|e| format!("Failed to list backups: {}", e))
    }

    /// Restores the configuration from a backup file.
    ///
    /// Creates a safety backup before restoring, then reloads keybindings from the restored config.
    ///
    /// # Arguments
    ///
    /// * `backup_path` - Path to the backup file to restore from
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Successfully restored and reloaded
    /// * `Err(String)` - Restore failed (original config unchanged)
    pub fn restore_backup(&self, backup_path: &Path) -> Result<(), String> {
        // Restore the backup via ConfigManager
        self.config_manager
            .borrow()
            .restore_backup(backup_path)
            .map_err(|e| format!("Failed to restore backup: {}", e))?;

        // Reload keybindings from the restored config
        self.load_keybindings()
            .map_err(|e| format!("Failed to reload keybindings: {}", e))?;
        self.clear_history();

        Ok(())
    }

    /// Deletes a specific backup file.
    ///
    /// # Arguments
    ///
    /// * `backup_path` - Path to the backup file to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Successfully deleted
    /// * `Err(String)` - Delete failed (file not found, permission error, etc.)
    pub fn delete_backup(&self, backup_path: &Path) -> Result<(), String> {
        // Delete the backup file
        fs::remove_file(backup_path).map_err(|e| format!("Failed to delete backup: {}", e))?;

        Ok(())
    }

    /// Exports a keybinding configuration file to a specific file path
    ///
    /// # Arguments
    ///
    /// * `export_path` - Path to export file that's created
    ///
    /// # Returns
    ///
    /// * `OK(())` - Successfully exported
    /// * `Err(String)` - Export failed (...)
    pub fn export_to(&self, export_path: &Path) -> Result<(), String> {
        // Get bindings from controller's storage
        let bindings = self.keybindings.borrow();

        self.config_manager
            .borrow_mut()
            .export_to(export_path, &bindings)
            .map_err(|e| format!("Failed to export config: {}", e))?;

        Ok(())
    }

    pub fn import_from(&self, import_path: &Path, mode: ImportMode) -> Result<(), String> {
        self.record_undo_snapshot();

        // Read the import file
        let content = read_to_string(import_path)
            .map_err(|e| format!("Failed to read import file: {}", e))?;

        // Parse bindings from import file
        let imported_bindings = parse_config_file(&content, import_path)
            .map_err(|e| format!("Failed to parse import file: {}", e))?;

        // Handle import mode
        match mode {
            ImportMode::Replace => {
                // Replace: Clear all and add imported
                self.keybindings.borrow_mut().clear();
                self.keybindings
                    .borrow_mut()
                    .extend(imported_bindings.clone());
            }
            ImportMode::Merge => {
                // Merge: Add imported, skip duplicates
                let mut existing = self.keybindings.borrow_mut();
                for binding in imported_bindings.clone() {
                    // Check if binding already exists (same key combo)
                    let exists = existing.iter().any(|b| b.key_combo == binding.key_combo);
                    if !exists {
                        existing.push(binding);
                    }
                }
            }
        }

        let bindings: Vec<_> = self.keybindings.borrow().clone();
        if let Err(e) = self
            .config_manager
            .borrow_mut()
            .write_bindings(&bindings)
            .map_err(|e| format!("Failed to write imported bindings: {}", e))
        {
            let previous = self.undo_stack.borrow_mut().pop();
            if let Some(previous) = previous {
                self.replace_bindings(previous);
            }
            return Err(e);
        }

        self.replace_bindings(bindings);

        Ok(())
    }

    /// Updates an existing keybinding with new values
    ///
    /// This method:
    /// 1. Finds the old binding in the list
    /// 2. Replaces it with the new binding
    /// 3. Writes changes to disk (creates automatic backup)
    /// 4. Rebuilds the conflict detector
    ///
    /// # Arguments
    /// * `old` - The binding to replace
    /// * `new` - The new binding values
    ///
    /// # Returns
    /// * `Ok(())` if successful
    /// * `Err(String)` with error message if operation fails
    ///
    /// # Example
    /// ```ignore
    /// match controller.update_keybinding(&old_binding, new_binding) {
    ///     Ok(()) => println!("Keybinding updated successfully"),
    ///     Err(e) => eprintln!("Failed to update: {}", e),
    /// }
    /// ```
    pub fn update_keybinding(&self, old: &Keybinding, new: Keybinding) -> Result<(), String> {
        self.record_undo_snapshot();
        let mut bindings = self.keybindings.borrow_mut();
        let position = bindings.iter().position(|b| b == old);

        match position {
            Some(pos) => {
                bindings[pos] = new.clone();
            }
            None => {
                return Err("Binding not found in the keybinding list".to_string());
            }
        }
        let updated_bindings = bindings.clone();
        drop(bindings);

        if let Err(e) = self.write_snapshot(&updated_bindings) {
            let previous = self.undo_stack.borrow_mut().pop();
            if let Some(previous) = previous {
                self.replace_bindings(previous);
            }
            return Err(e);
        }

        self.replace_bindings(updated_bindings);

        Ok(())
    }

    pub fn undo(&self) -> Result<(), String> {
        let Some(previous) = self.undo_stack.borrow_mut().pop() else {
            return Err("Nothing to undo".to_string());
        };

        let current = self.keybindings.borrow().clone();
        self.redo_stack.borrow_mut().push(current);

        if let Err(e) = self.write_snapshot(&previous) {
            let redo = self.redo_stack.borrow_mut().pop();
            if let Some(redo) = redo {
                self.undo_stack.borrow_mut().push(redo);
            }
            return Err(e);
        }

        self.replace_bindings(previous);
        Ok(())
    }

    pub fn redo(&self) -> Result<(), String> {
        let Some(next) = self.redo_stack.borrow_mut().pop() else {
            return Err("Nothing to redo".to_string());
        };

        let current = self.keybindings.borrow().clone();
        self.undo_stack.borrow_mut().push(current);

        if let Err(e) = self.write_snapshot(&next) {
            let undo = self.undo_stack.borrow_mut().pop();
            if let Some(undo) = undo {
                self.redo_stack.borrow_mut().push(undo);
            }
            return Err(e);
        }

        self.replace_bindings(next);
        Ok(())
    }

    /// Applies changes to running Hyprland instance
    ///
    /// Triggers Hyprland to reload its configuration file, making all
    /// pending changes take effect immediately without restart.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Hyprland reloaded successfully
    /// * `Err(String)` - Hyprland not running or reload failed
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use hypr_keybind_manager::ui::Controller;
    /// # use std::path::PathBuf;
    /// # let controller = Controller::new(PathBuf::from("hyprland.conf")).unwrap();
    /// // User made changes via add/update/delete...
    /// controller.apply_to_hyprland()?;  // Apply to running Hyprland
    /// # Ok::<(), String>(())
    /// ```
    pub fn apply_to_hyprland(&self) -> Result<(), String> {
        Command::new("hyprctl")
            .arg("reload")
            .output()
            .map_err(|e| format!("Failed to run hyprctl: {}", e))?;

        Ok(())
    }
}

fn candidate_keys() -> Vec<&'static str> {
    let mut keys = Vec::with_capacity(48);
    keys.extend([
        "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q",
        "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
    ]);
    keys.extend(["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]);
    const FUNCTION_KEYS: [&str; 12] = [
        "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12",
    ];
    keys.extend(FUNCTION_KEYS);

    keys
}
