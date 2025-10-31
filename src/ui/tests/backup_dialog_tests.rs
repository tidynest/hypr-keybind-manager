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

//! Backup dialog tests
//!
//! Tests for backup management dialog functionality:
//! - Timestamp formatting and parsing
//! - Fallback behaviour for malformed filenames
//! - Edge cases (missing underscore, invalid lengths, etc.)

use std::path::PathBuf;

use crate::ui::components::backup_dialog::BackupDialog;

#[test]
fn test_format_backup_display_valid_timestamp() {
    let path = PathBuf::from(
        "/backups/hyprland.con
  f.2025-10-15_143025",
    );
    let result = BackupDialog::format_backup_display(&path);
    assert_eq!(result, "2025-10-15 14:30:25");
}

#[test]
fn test_format_backup_display_different_timestamp() {
    let path = PathBuf::from(
        "/backups/hyprland.con
  f.2024-12-31_235959",
    );
    let result = BackupDialog::format_backup_display(&path);
    assert_eq!(result, "2024-12-31 23:59:59");
}

#[test]
fn test_format_backup_display_invalid_format() {
    // Invalid: wrong number of parts
    let path = PathBuf::from("/backups/hyprland.conf");
    let result = BackupDialog::format_backup_display(&path);
    assert_eq!(
        result, "hyprland.conf",
        "Should
  fall back to filename"
    );
}

#[test]
fn test_format_backup_display_malformed_timestamp() {
    // Valid structure but malformed timestamp (7 digits instead of 6)
    let path = PathBuf::from("/backups/hyprland.conf.2025-10-15_1430255");
    let result = BackupDialog::format_backup_display(&path);
    // Should fall back to original filename since time_part.len() != 6
    assert_eq!(result, "hyprland.conf.2025-10-15_1430255");
}

#[test]
fn test_format_backup_display_no_underscore() {
    // Missing underscore in timestamp
    let path = PathBuf::from("/backups/hyprland.conf.20251015143025");
    let result = BackupDialog::format_backup_display(&path);
    // Should fall back to original filename since split('_') won't give 2 parts
    assert_eq!(result, "hyprland.conf.20251015143025");
}

#[test]
fn test_format_backup_display_unknown_backup() {
    // Path with no filename
    let path = PathBuf::from("/");
    let result = BackupDialog::format_backup_display(&path);
    assert_eq!(result, "Unknown backup");
}
