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

//! IPC module tests
//!
//! Contains test suites for Hyprland IPC functionality.
//! Note: Some tests require a running Hyprland instance and are marked #[ignore].

use crate::{
    config::ConfigError,
    core::{BindType, KeyCombo, Keybinding, Modifier},
    ipc::{ClientMode, HyprlandClient},
};

/// Helper: Creates a safe test binding
fn create_safe_binding(key: &str, app: &str) -> Keybinding {
    Keybinding {
        key_combo: KeyCombo::new(vec![Modifier::Super], key),
        bind_type: BindType::Bind,
        dispatcher: "exec".to_string(),
        args: Some(app.to_string()),
    }
}

#[test]
fn test_dryrun_mode_validates_safe_binding() {
    let client = HyprlandClient::new(ClientMode::DryRun);
    let binding = create_safe_binding("K", "firefox");

    let result = client.add_bind(&binding);
    assert!(
        result.is_ok(),
        "Safe binding should validate in DryRun mode"
    );
}

#[test]
fn test_dryrun_mode_blocks_injection() {
    let client = HyprlandClient::new(ClientMode::DryRun);

    // Injection attempt with semicolon
    let malicious = Keybinding {
        key_combo: KeyCombo::new(vec![Modifier::Super], "K"),
        bind_type: BindType::Bind,
        dispatcher: "exec".to_string(),
        args: Some("firefox; echo hacked".to_string()),
    };

    let result = client.add_bind(&malicious);
    assert!(
        result.is_err(),
        "Injection should be blocked even in DryRun"
    );

    match result.unwrap_err() {
        ConfigError::ValidationFailed(msg) => {
            assert!(
                msg.contains("metacharacter") || msg.contains("';'"),
                "Error should mention metacharacters: {}",
                msg
            );
        }
        other => panic!("Expected ValidationFailed, got {:?}", other),
    }
}

#[test]
fn test_readonly_mode_blocks_modifications() {
    let client = HyprlandClient::new(ClientMode::ReadOnly);
    let binding = create_safe_binding("M", "kitty");

    let result = client.add_bind(&binding);
    assert!(result.is_err(), "ReadOnly mode should block modifications");

    match result.unwrap_err() {
        ConfigError::IpcCommandFailed(msg) => {
            assert!(
                msg.contains("read-only"),
                "Error should mention read-only mode: {}",
                msg
            );
        }
        other => panic!("Expected IpcCommandFailed, got {:?}", other),
    }
}

#[test]
fn test_command_build_single_modifier() {
    let client = HyprlandClient::new(ClientMode::DryRun);
    let binding = create_safe_binding("K", "firefox");

    let cmd = client.build_keyword_command("bind", &binding);

    // Should be: "SUPER, K, exec, firefox"
    assert!(cmd.contains("SUPER"), "Should contain SUPER");
    assert!(cmd.contains("K"), "Should contain K");
    assert!(cmd.contains("exec"), "Should contain exec");
    assert!(cmd.contains("firefox"), "Should contain firefox");
}

#[test]
fn test_command_building_multiple_modifiers() {
    let client = HyprlandClient::new(ClientMode::DryRun);
    let binding = Keybinding {
        key_combo: KeyCombo::new(vec![Modifier::Super, Modifier::Shift], "M"),
        bind_type: BindType::Bind,
        dispatcher: "exec".to_string(),
        args: Some("kitty".to_string()),
    };

    let cmd = client.build_keyword_command("bind", &binding);

    // Should contain both modifiers joined with underscore
    assert!(
        cmd.contains("SUPER") && cmd.contains("SHIFT"),
        "Should contain both modifiers: {}",
        cmd
    );
}

#[test]
fn test_command_building_no_args() {
    let client = HyprlandClient::new(ClientMode::DryRun);

    let binding = Keybinding {
        key_combo: KeyCombo::new(vec![Modifier::Super], "Q"),
        bind_type: BindType::Bind,
        dispatcher: "killactive".to_string(),
        args: None,
    };

    let cmd = client.build_keyword_command("bind", &binding);

    // Should be: "SUPER, Q, killactive"
    assert!(cmd.contains("SUPER"), "Should contain SUPER");
    assert!(cmd.contains("Q"), "Should contain Q");
    assert!(cmd.contains("killactive"), "Should contain killactive");
    assert!(!cmd.ends_with(','), "Should not have trailing comma");
}

#[test]
fn test_remove_bind_validates() {
    let client = HyprlandClient::new(ClientMode::DryRun);
    let binding = create_safe_binding("X", "firefox");

    let result = client.remove_bind(&binding);
    assert!(result.is_ok(), "Safe binding should validate for removal");
}

#[test]
fn test_reload_in_dryrun() {
    let client = HyprlandClient::new(ClientMode::DryRun);

    let result = client.reload();
    assert!(result.is_ok(), "Reload should succeed in DryRun mode");
}

#[test]
fn test_reload_blocked_in_readonly() {
    let client = HyprlandClient::new(ClientMode::ReadOnly);

    let result = client.reload();
    assert!(result.is_err(), "Reload should be blocked in ReadOnly mode");
}

#[test]
fn test_multiple_safe_bindings() {
    let client = HyprlandClient::new(ClientMode::DryRun);

    let bindings = vec![
        create_safe_binding("K", "firefox"),
        create_safe_binding("M", "kitty"),
        create_safe_binding("B", "brave"),
    ];

    for binding in bindings {
        let result = client.add_bind(&binding);
        assert!(
            result.is_ok(),
            "Safe binding {} should validate",
            binding.key_combo.key
        );
    }
}

// Integration test - requires Hyprland running
// This test is IGNORED by default and should ONLY be run in a VM
#[test]
#[ignore]
fn test_live_mode_integration() {
    // ⚠️ WARNING: This test requires Hyprland to be running
    // ⚠️ Only run this in a VM with: cargo test -- --ignored

    let client = HyprlandClient::new(ClientMode::Live);

    // Use a safe, harmless binding for testing
    let binding = create_safe_binding("F12", "notify-send 'Test binding'");

    // This will actually send to Hyprland
    let result = client.add_bind(&binding);

    // If Hyprland is running, should succeed
    // If not running, should get HyprlandNotRunning error
    match result {
        Ok(()) => println!("✓ Successfully sent command to Hyprland"),
        Err(ConfigError::HyprlandNotRunning(msg)) => {
            println!("⚠ Hyprland not running: {}", msg);
        }
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}
