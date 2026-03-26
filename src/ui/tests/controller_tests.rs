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

use crate::{
    core::{BindType, KeyCombo, Keybinding, Modifier},
    ui::controller::{KeyComboAssistance, KeyComboAvailability},
    ui::Controller,
};

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

fn create_modifier_order_config() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("hyprland.conf");

    let content = r#"
bind = SUPER ALT, 1, exec, firefox
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

// ============================================================================
// Search Query State Management Tests
// ============================================================================

#[test]
fn test_search_query_storage() {
    let (_temp_dir, config_path) = create_test_config();
    let controller = Controller::new(config_path).unwrap();
    controller.load_keybindings().unwrap();

    // Initially empty query
    let initial_query = controller.get_search_query();
    assert_eq!(initial_query, "", "Initial query should be empty");

    // Set search query
    controller.set_search_query("firefox".to_string());

    // Verify it's stored
    let stored_query = controller.get_search_query();
    assert_eq!(stored_query, "firefox", "Query should be stored");
}

#[test]
fn test_get_current_view_respects_filter() {
    let (_temp_dir, config_path) = create_test_config();
    let controller = Controller::new(config_path).unwrap();
    controller.load_keybindings().unwrap();

    // No filter = all keybindings
    let all = controller.get_current_view();
    assert_eq!(all.len(), 5, "Should return all 5 keybindings");

    // Set filter to "firefox"
    controller.set_search_query("firefox".to_string());
    let filtered = controller.get_current_view();
    assert_eq!(filtered.len(), 1, "Should return only firefox binding");
    assert!(filtered[0].args.as_ref().unwrap().contains("firefox"));

    // Change filter to "SUPER+M"
    controller.set_search_query("SUPER+M".to_string());
    let filtered = controller.get_current_view();
    assert_eq!(filtered.len(), 1, "Should return only SUPER+M binding");
}

#[test]
fn test_search_persists_after_delete() {
    let (_temp_dir, config_path) = create_test_config();
    let controller = Controller::new(config_path).unwrap();
    controller.load_keybindings().unwrap();

    // Set filter to "firefox"
    controller.set_search_query("firefox".to_string());

    // Get the binding to delete
    let bindings_before = controller.get_current_view();
    assert_eq!(bindings_before.len(), 1, "Should have one firefox binding");

    let to_delete = bindings_before[0].clone();

    // Delete it
    controller.delete_keybinding(&to_delete).unwrap();

    // Verify search query is STILL stored
    let query = controller.get_search_query();
    assert_eq!(query, "firefox", "Search query should persist after delete");

    // Get current view (should be empty now - no firefox bindings left)
    let bindings_after = controller.get_current_view();
    assert_eq!(
        bindings_after.len(),
        0,
        "Should have no firefox bindings after delete"
    );

    // But total keybindings should be 4 remaining
    let all_bindings = controller.get_keybindings();
    assert_eq!(
        all_bindings.len(),
        4,
        "Should have 4 total bindings remaining"
    );
}

#[test]
fn test_search_persists_after_add() {
    let (_temp_dir, config_path) = create_test_config();
    let controller = Controller::new(config_path).unwrap();
    controller.load_keybindings().unwrap();

    // Set filter to "firefox"
    controller.set_search_query("firefox".to_string());

    // Add a new binding (doesn't match filter)
    let new_binding = Keybinding {
        bind_type: BindType::Bind,
        key_combo: KeyCombo::new(vec![Modifier::Super], "X"),
        dispatcher: "exec".to_string(),
        args: Some("code".to_string()),
    };

    controller.add_keybinding(new_binding).unwrap();

    // Search query should persist
    let query = controller.get_search_query();
    assert_eq!(query, "firefox", "Search query should persist after add");

    // Current view should still show only firefox (new binding doesn't match)
    let filtered = controller.get_current_view();
    assert_eq!(filtered.len(), 1, "Should still show only firefox binding");
    assert!(filtered[0].args.as_ref().unwrap().contains("firefox"));

    // But total count should be 6 now
    let all = controller.get_keybindings();
    assert_eq!(all.len(), 6, "Should have 6 total bindings");
}

#[test]
fn test_search_persists_after_update() {
    let (_temp_dir, config_path) = create_test_config();
    let controller = Controller::new(config_path).unwrap();
    controller.load_keybindings().unwrap();

    // Set filter to "firefox"
    controller.set_search_query("firefox".to_string());

    // Get the firefox binding
    let old_binding = controller.get_current_view()[0].clone();

    // Update it to "brave"
    let mut new_binding = old_binding.clone();
    new_binding.args = Some("brave".to_string());

    controller
        .update_keybinding(&old_binding, new_binding)
        .unwrap();

    // Search query should persist
    let query = controller.get_search_query();
    assert_eq!(query, "firefox", "Search query should persist after update");

    // Current view should be empty (no firefox bindings anymore)
    let filtered = controller.get_current_view();
    assert_eq!(
        filtered.len(),
        0,
        "Should have no firefox bindings after update"
    );

    // But changing filter to "brave" should show the updated binding
    controller.set_search_query("brave".to_string());
    let brave_filtered = controller.get_current_view();
    assert_eq!(brave_filtered.len(), 1, "Should find updated brave binding");
}

#[test]
fn test_key_combo_assistance_marks_busy_combo_as_in_use() {
    let (_temp_dir, config_path) = create_test_config();
    let controller = Controller::new(config_path).unwrap();
    controller.load_keybindings().unwrap();

    let combo = KeyCombo::new(vec![Modifier::Super], "k");
    let assistance = controller.get_key_combo_assistance(Some(&combo), None);

    match assistance.availability {
        KeyComboAvailability::InUse(bindings) => {
            assert_eq!(bindings.len(), 2, "SUPER+K should be reported as busy");
        }
        other => panic!("expected busy combo, got {:?}", other),
    }
}

#[test]
fn test_key_combo_assistance_marks_free_combo_as_available() {
    let (_temp_dir, config_path) = create_test_config();
    let controller = Controller::new(config_path).unwrap();
    controller.load_keybindings().unwrap();

    let combo = KeyCombo::new(vec![Modifier::Super], "X");
    let assistance = controller.get_key_combo_assistance(Some(&combo), None);

    assert_eq!(
        assistance,
        KeyComboAssistance {
            availability: KeyComboAvailability::Available,
            suggestions: Vec::new(),
        }
    );
}

#[test]
fn test_key_combo_assistance_ignores_original_binding_when_editing() {
    let (_temp_dir, config_path) = create_test_config();
    let controller = Controller::new(config_path).unwrap();
    controller.load_keybindings().unwrap();

    let original = controller
        .get_keybindings()
        .into_iter()
        .find(|binding| binding.args.as_deref() == Some("kitty"))
        .unwrap();

    let assistance =
        controller.get_key_combo_assistance(Some(&original.key_combo), Some(&original));

    assert_eq!(
        assistance.availability,
        KeyComboAvailability::Available,
        "Editing a binding without changing its combo should not self-conflict"
    );
}

#[test]
fn test_key_combo_suggestions_skip_used_combos_and_preserve_modifier_set() {
    let (_temp_dir, config_path) = create_test_config();
    let controller = Controller::new(config_path).unwrap();
    controller.load_keybindings().unwrap();

    let original = KeyCombo::new(vec![Modifier::Super], "K");
    let suggestions = controller.suggest_key_combos(&original.modifiers, None, 5, &original);

    assert_eq!(suggestions.len(), 5);
    assert_eq!(suggestions[0], KeyCombo::new(vec![Modifier::Super], "A"));
    assert!(
        suggestions
            .iter()
            .all(|combo| combo.modifiers == vec![Modifier::Super]),
        "Suggestions should keep the original modifier set"
    );
    assert!(
        suggestions
            .iter()
            .all(|combo| controller.is_key_combo_available(combo, None)),
        "Suggestions should only contain free combos"
    );
}

#[test]
fn test_key_combo_assistance_matches_loaded_binding_with_different_modifier_order() {
    let (_temp_dir, config_path) = create_modifier_order_config();
    let controller = Controller::new(config_path).unwrap();
    controller.load_keybindings().unwrap();

    let typed_combo = KeyCombo::new(vec![Modifier::Alt, Modifier::Super], "1");
    let assistance = controller.get_key_combo_assistance(Some(&typed_combo), None);

    match assistance.availability {
        KeyComboAvailability::InUse(bindings) => {
            assert_eq!(bindings.len(), 1);
            assert_eq!(bindings[0].args.as_deref(), Some("firefox"));
        }
        other => panic!("expected in-use combo, got {:?}", other),
    }
}

#[test]
fn test_undo_reverts_added_binding() {
    let (_temp_dir, config_path) = create_test_config();
    let controller = Controller::new(config_path).unwrap();
    controller.load_keybindings().unwrap();

    let new_binding = Keybinding {
        bind_type: BindType::Bind,
        key_combo: KeyCombo::new(vec![Modifier::Super], "X"),
        dispatcher: "exec".to_string(),
        args: Some("code".to_string()),
    };

    controller.add_keybinding(new_binding).unwrap();
    assert_eq!(controller.keybinding_count(), 6);
    assert!(controller.can_undo());

    controller.undo().unwrap();

    assert_eq!(controller.keybinding_count(), 5);
    assert!(!controller
        .filter_keybindings("code")
        .iter()
        .any(|b| b.args.as_deref() == Some("code")));
    assert!(controller.can_redo());
}

#[test]
fn test_redo_reapplies_undone_change() {
    let (_temp_dir, config_path) = create_test_config();
    let controller = Controller::new(config_path).unwrap();
    controller.load_keybindings().unwrap();

    let new_binding = Keybinding {
        bind_type: BindType::Bind,
        key_combo: KeyCombo::new(vec![Modifier::Super], "X"),
        dispatcher: "exec".to_string(),
        args: Some("code".to_string()),
    };

    controller.add_keybinding(new_binding).unwrap();
    controller.undo().unwrap();
    controller.redo().unwrap();

    assert_eq!(controller.keybinding_count(), 6);
    assert_eq!(controller.filter_keybindings("code").len(), 1);
}

#[test]
fn test_undo_reverts_updated_binding() {
    let (_temp_dir, config_path) = create_test_config();
    let controller = Controller::new(config_path).unwrap();
    controller.load_keybindings().unwrap();

    let original = controller.filter_keybindings("firefox")[0].clone();
    let mut updated = original.clone();
    updated.args = Some("brave".to_string());

    controller.update_keybinding(&original, updated).unwrap();
    assert_eq!(controller.filter_keybindings("brave").len(), 1);

    controller.undo().unwrap();

    assert_eq!(controller.filter_keybindings("brave").len(), 0);
    assert_eq!(controller.filter_keybindings("firefox").len(), 1);
}
