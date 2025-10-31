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

use super::super::*;
use std::{fs, thread, time::Duration};
use tempfile::TempDir;

use crate::{BindType, KeyCombo, Modifier::Super};

/// Helper to create a test keybinding
fn create_test_binding() -> Keybinding {
    Keybinding {
        key_combo: KeyCombo::new(vec![Super], "M"),
        bind_type: BindType::Bind,
        dispatcher: "exec".to_string(),
        args: Some("kitty".to_string()),
    }
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
    assert_eq!(
        backups_after.len(),
        1,
        "Backup should still exist after commit"
    );

    // Verify backup contains original content
    let backup_content = fs::read_to_string(&backups_after[0]).unwrap();
    assert_eq!(
        backup_content, original_content,
        "Backup should contain original content"
    );
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
    assert_eq!(
        fs::read_to_string(&config_path).unwrap(),
        content_with_blanks
    );

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
    assert_eq!(
        backups.len(),
        3,
        "Should have 3 backups from 3 transactions"
    );

    // Verify backups are sorted newest first
    let backup1_content = fs::read_to_string(&backups[0]).unwrap(); // Newest
    let backup2_content = fs::read_to_string(&backups[1]).unwrap(); // Middle
    let backup3_content = fs::read_to_string(&backups[2]).unwrap(); // Oldest

    assert_eq!(
        backup1_content, "version 3\n",
        "Newest backup should be from tx3"
    );
    assert_eq!(
        backup2_content, "version 2\n",
        "Middle backup should be from tx2"
    );
    assert_eq!(
        backup3_content, "version 1\n",
        "Oldest backup should be from tx1"
    );

    // Verify final config has latest content
    assert_eq!(fs::read_to_string(&config_path).unwrap(), "version 4\n");

    // Test cleanup integration
    let deleted = manager.cleanup_old_backups(2).unwrap();
    assert_eq!(deleted, 1, "Should delete 1 old backup (keeping 2)");

    let remaining = manager.list_backups().unwrap();
    assert_eq!(remaining.len(), 2, "Should have 2 backups remaining");
}

#[test]
fn test_injection_blocks_commit() {
    // Test: Layer 1 injection attempts should block commit

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("hyprland.conf");
    fs::write(&config_path, "bind = SUPER, K, exec, firefox\n").unwrap();

    let manager = ConfigManager::new(config_path.clone()).unwrap();
    let tx = ConfigTransaction::begin(&manager).unwrap();

    // Try to commit config with shell injection
    let malicious = "bind = SUPER, K, exec, firefox; rm -rf /\n";
    let result = tx.commit_with_validation(malicious);

    // Should fail with ValidationFailed
    assert!(result.is_err(), "Injection attempt should be blocked");

    match result.unwrap_err() {
        ConfigError::ValidationFailed(msg) => {
            assert!(
                msg.contains("validation error"),
                "Error should mention validation: {}",
                msg
            );
        }
        other => panic!("Expected ValidationFailed, got {:?}", other),
    }

    // Original config should be unchanged (transaction rolled back)
    let current = manager.read_config().unwrap();
    assert_eq!(
        current, "bind = SUPER, K, exec, firefox\n",
        "Original config should be untouched after failed validation"
    );
}

#[test]
fn test_critical_danger_blocks_commit() {
    // Test: Layer 2 critical dangers should block commit

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("hyprland.conf");
    fs::write(&config_path, "bind = SUPER, K, exec, firefox\n").unwrap();

    let manager = ConfigManager::new(config_path.clone()).unwrap();
    let tx = ConfigTransaction::begin(&manager).unwrap();

    // Try to commit config with critical danger (no injection, valid syntax)
    let dangerous = "bind = SUPER, K, exec, rm -rf /\n";
    let result = tx.commit_with_validation(dangerous);

    // Should fail with DangerousCommand
    assert!(result.is_err(), "Critical danger should be blocked");

    match result.unwrap_err() {
        ConfigError::DangerousCommand(msg) => {
            assert!(
                msg.contains("Critical danger") || msg.contains("blocked"),
                "Error should mention danger: {}",
                msg
            );
        }
        other => panic!("Expected DangerousCommand, got {:?}", other),
    }

    // Original config should be unchanged
    let current = manager.read_config().unwrap();
    assert_eq!(
        current, "bind = SUPER, K, exec, firefox\n",
        "Original config should be untouched after blocked danger"
    );
}

#[test]
fn test_warnings_allow_commit() {
    // Test: Warning-level issues should allow commit (informational only)
    // Note: Round 1 only has Critical patterns, so we test safe commands pass

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("hyprland.conf");
    fs::write(&config_path, "bind = SUPER, K, exec, firefox\n").unwrap();

    let manager = ConfigManager::new(config_path.clone()).unwrap();
    let tx = ConfigTransaction::begin(&manager).unwrap();

    // Commit safe config (no warnings in Round 1, but demonstrates success path)
    let safe = "bind = SUPER, M, exec, kitty\n";
    let result = tx.commit_with_validation(safe);

    // Should succeed
    assert!(result.is_ok(), "Safe config should commit successfully");

    // Config should be updated
    let current = manager.read_config().unwrap();
    assert_eq!(current, safe, "Config should be updated with safe content");

    // Backup should exist (from transaction begin)
    let backups = manager.list_backups().unwrap();
    assert_eq!(backups.len(), 1, "Transaction should have created backup");
}

#[test]
fn test_clean_config_commits() {
    // Test: Multi-binding clean config should commit without issues

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("hyprland.conf");
    fs::write(&config_path, "initial\n").unwrap();

    let manager = ConfigManager::new(config_path.clone()).unwrap();
    let tx = ConfigTransaction::begin(&manager).unwrap();

    // Commit multi-binding clean config
    let clean = r#"# Safe configuration
bind = SUPER, K, exec, firefox
bind = SUPER, M, exec, kitty
bind = SUPER, Q, killactive
bind = SUPER, F, togglefloating
"#;

    let result = tx.commit_with_validation(clean);

    // Should succeed
    assert!(
        result.is_ok(),
        "Clean config should commit successfully: {:?}",
        result
    );

    // Config should be updated exactly
    let current = manager.read_config().unwrap();
    assert_eq!(
        current, clean,
        "Config should match committed content exactly"
    );

    // Should have one backup from transaction
    let backups = manager.list_backups().unwrap();
    assert_eq!(
        backups.len(),
        1,
        "Should have backup from transaction begin"
    );
}

// ============================================================================
// Write Functionality Tests
// ============================================================================

#[test]
fn test_format_binding_with_modifiers() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("hyprland.conf");
    fs::write(&config_path, "initial\n").unwrap();

    let manager = ConfigManager::new(config_path).unwrap();

    let binding = Keybinding {
        key_combo: KeyCombo::new(vec![Super], "K"),
        bind_type: BindType::Bind,
        dispatcher: "exec".to_string(),
        args: Some("firefox".to_string()),
    };

    let formatted = manager.format_binding(&binding);

    // Should match Hyprland format: bind = SUPER, K, exec, firefox
    assert!(formatted.contains("bind"));
    assert!(formatted.contains("SUPER"));
    assert!(formatted.contains("K"));
    assert!(formatted.contains("exec"));
    assert!(formatted.contains("firefox"));
    assert!(formatted.contains("="));
    assert!(formatted.contains(","));
}

#[test]
fn test_format_binding_multiple_modifiers() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("hyprland.conf");
    fs::write(&config_path, "initial\n").unwrap();

    let manager = ConfigManager::new(config_path).unwrap();

    let binding = Keybinding {
        key_combo: KeyCombo::new(vec![Super, Shift], "M"),
        bind_type: BindType::Bind,
        dispatcher: "exec".to_string(),
        args: Some("kitty".to_string()),
    };

    let formatted = manager.format_binding(&binding);

    // Should have both modifiers joined with underscore
    assert!(formatted.contains("SUPER") || formatted.contains("SHIFT"));
    assert!(formatted.contains("_"));
}

#[test]
fn test_format_binding_no_args() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("hyprland.conf");
    fs::write(&config_path, "initial\n").unwrap();

    let manager = ConfigManager::new(config_path).unwrap();

    let binding = Keybinding {
        key_combo: KeyCombo::new(vec![Super], "Q"),
        bind_type: BindType::Bind,
        dispatcher: "killactive".to_string(),
        args: None,
    };

    let formatted = manager.format_binding(&binding);

    // Should have dispatcher but no args
    assert!(formatted.contains("killactive"));
    assert!(!formatted.ends_with(","));
}

#[test]
fn test_write_bindings_basic() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("hyprland.conf");

    // Create initial config with comments and settings
    let initial_content = r#"# My Hyprland Config
$mainMod = SUPER

# Window rules
windowrule = float, pavucontrol

# Original keybindings
bind = SUPER, K, exec, firefox
bind = SUPER, M, exec, kitty

# More settings below
decoration {
    rounding = 10
}
"#;
    fs::write(&config_path, initial_content).unwrap();

    let mut manager = ConfigManager::new(config_path.clone()).unwrap();

    // New bindings (modified)
    let new_bindings = vec![
        Keybinding {
            key_combo: KeyCombo::new(vec![Super], "K"),
            bind_type: BindType::Bind,
            dispatcher: "exec".to_string(),
            args: Some("brave".to_string()), // Changed from firefox
        },
        Keybinding {
            key_combo: KeyCombo::new(vec![Super], "M"),
            bind_type: BindType::Bind,
            dispatcher: "exec".to_string(),
            args: Some("alacritty".to_string()), // Changed from kitty
        },
    ];

    // Write new bindings
    manager.write_bindings(&new_bindings).unwrap();

    // Read result
    let result = fs::read_to_string(&config_path).unwrap();

    // Verify: Should have new bindings
    assert!(result.contains("brave"), "Should have new firefox → brave");
    assert!(
        result.contains("alacritty"),
        "Should have new kitty → alacritty"
    );

    // Verify: Should NOT have old bindings
    assert!(!result.contains("firefox"), "Should not have old firefox");
    assert!(!result.contains("kitty"), "Should not have old kitty");

    // Verify: Should preserve comments and settings
    assert!(
        result.contains("# My Hyprland Config"),
        "Should preserve header comment"
    );
    assert!(
        result.contains("$mainMod = SUPER"),
        "Should preserve variables"
    );
    assert!(
        result.contains("windowrule"),
        "Should preserve window rules"
    );
    assert!(
        result.contains("decoration"),
        "Should preserve decoration section"
    );
    assert!(result.contains("rounding = 10"), "Should preserve settings");
}

#[test]
fn test_write_bindings_preserves_structure() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("hyprland.conf");

    let initial = r#"# Top comment

# Keybindings start here
bind = SUPER, K, exec, firefox

# Bottom comment
"#;
    fs::write(&config_path, initial).unwrap();

    let mut manager = ConfigManager::new(config_path.clone()).unwrap();

    let new_bindings = vec![create_test_binding()];

    manager.write_bindings(&new_bindings).unwrap();

    let result = fs::read_to_string(&config_path).unwrap();

    // Should have both comments
    assert!(result.contains("# Top comment"));
    assert!(result.contains("# Bottom comment"));

    // Should have new binding
    assert!(result.contains("kitty"));

    // Should NOT have old binding
    assert!(!result.contains("firefox"));
}

#[test]
fn test_write_bindings_creates_backup() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("hyprland.conf");

    fs::write(&config_path, "bind = SUPER, K, exec, firefox\n").unwrap();

    let mut manager = ConfigManager::new(config_path.clone()).unwrap();

    // Check no backups initially
    let backups_before = manager.list_backups().unwrap();
    assert_eq!(backups_before.len(), 0);

    // Write new bindings
    let new_bindings = vec![create_test_binding()];

    manager.write_bindings(&new_bindings).unwrap();

    // Should have created a backup
    let backups_after = manager.list_backups().unwrap();
    assert_eq!(backups_after.len(), 1, "Should create backup automatically");

    // Backup should contain original content
    let backup_content = fs::read_to_string(&backups_after[0]).unwrap();
    assert!(
        backup_content.contains("firefox"),
        "Backup should have original binding"
    );
}
