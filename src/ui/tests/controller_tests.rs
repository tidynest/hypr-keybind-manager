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

//! Controller tests
//!
//! Test for the MVC Controller logic

use std::{fs, path::PathBuf};
use tempfile::TempDir;

use crate::ui::Controller;

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

    assert!(
        controller.is_ok(),
        "Controller should be created successfully"
    );
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
    assert_eq!(
        bindings.len(),
        5,
        "Should return all 5 keybindings successfully"
    );
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
    assert_eq!(
        conflicts.len(),
        1,
        "Should detect 1 conflict (SUPER+K used twice)"
    );

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
