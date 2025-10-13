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

use std::path::PathBuf;
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::{ConfigError, ConfigManager};
use crate::core::{Conflict, ConflictDetector, Keybinding};

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
}

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
        })
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
        use crate::core::parser::parse_config_file;
        use std::path::Path;

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
    /// - Layer 2: Danger detection (config/danger.rs)
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
        use crate::config::validator::ConfigValidator;
        use crate::core::validator as injection_validator;

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

}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    /// Helper: Creates test config with known content
    fn create_test_config() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("hyprland.conf");

        let content = r#"
# Test configuration
bind = SUPER, K, exec, firefox
bind = SUPER, M, exec, kitty
bind = SUPER, Q, exec, killactive
bind = SUPER, F, togglefloating

# Conflict: duplicate SUPER+K
bind = SUPER, K, exec, chrome
"#;

        fs::write(&config_path, content).unwrap();
        (temp_dir, config_path)
    }

    #[test]
    fn test_controller_creation() {
        let (_temp_dir, config_path) = create_test_config();
        let controller = Controller::new(config_path);

        assert!(controller.is_ok(), "Controller should be created successfully");
    }

    #[test]
    fn test_load_keybindings() {
        let (_temp_dir, config_path) = create_test_config();
        let controller = Controller::new(config_path).unwrap();

        let count = controller.load_keybindings();
        assert!(count.is_ok(), "Should load keybindings successfully");

        let count = count.unwrap();
        assert_eq!(count, 5, "Should load 5 keybindings successfully");
    }

    #[test]
    fn test_get_keybindings() {
        let (_temp_dir, config_path) = create_test_config();
        let controller = Controller::new(config_path).unwrap();
        controller.load_keybindings().unwrap();

        let bindings = controller.get_keybindings();
        assert_eq!(bindings.len(), 5, "Should return all 5 keybindings successfully");
    }

    #[test]
    fn test_filter_keybindings_by_app() {
        let (_temp_dir, config_path) = create_test_config();
        let controller = Controller::new(config_path).unwrap();
        controller.load_keybindings().unwrap();

        let filtered = controller.filter_keybindings("firefox");
        assert_eq!(filtered.len(), 1, "Should find 1 binding with 'firefox'");
    }

    #[test]
    fn test_filter_keybindings_by_key() {
        let (_temp_dir, config_path) = create_test_config();
        let controller = Controller::new(config_path).unwrap();
        controller.load_keybindings().unwrap();

        let filtered = controller.filter_keybindings("SUPER+K");
        assert_eq!(filtered.len(), 2, "Should find 2 bindings with 'SUPER+K'");
    }

    #[test]
    fn test_filter_keybindings_empty_query() {
        let (_temp_dir, config_path) = create_test_config();
        let controller = Controller::new(config_path).unwrap();
        controller.load_keybindings().unwrap();

        let filtered = controller.filter_keybindings("");
        assert_eq!(filtered.len(), 5, "Empty query should return all bindings");
    }

    #[test]
    fn test_detect_conflicts() {
        let (_temp_dir, config_path) = create_test_config();
        let controller = Controller::new(config_path).unwrap();
        controller.load_keybindings().unwrap();

        let conflicts = controller.get_conflicts();
        assert_eq!(conflicts.len(), 1, "Should detect 1 conflict (SUPER+K used twice");

        let conflict = &conflicts[0];
        assert_eq!(
            conflict.conflicting_bindings.len(),
            2,
            "Conflict should involve 2 bindings"
        );
    }

    #[test]
    fn test_keybinding_count() {
        let (_temp_dir, config_path) = create_test_config();
        let controller = Controller::new(config_path).unwrap();
        controller.load_keybindings().unwrap();

        assert_eq!(controller.keybinding_count(), 5);
        assert_eq!(controller.conflict_count(), 1);
    }
}

