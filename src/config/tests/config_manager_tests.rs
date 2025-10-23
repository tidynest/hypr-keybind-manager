use super::super::*;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use std::thread;
use tempfile::TempDir;

/// Helper: Creates a temporary config file for testing.
fn create_test_config() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("hyprland.conf");
    fs::write(
        &config_path,
        "# Test config\nbind = SUPER, Q, exec, firefox\n")
        .unwrap();
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
    let safety_content = fs::read_to_string(safety_backup).unwrap();
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
    assert_eq!(current, original_content, "Config should be unchanged after failed restore");

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
