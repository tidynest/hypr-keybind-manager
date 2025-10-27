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

use crate::config::{danger::DangerLevel, validator::ConfigValidator};

#[test]
fn test_injection_blocked_by_layer1() {
    // Layer 1 should block shell injection attempts
    let validator = ConfigValidator::new();
    let config = "bind = SUPER, K, exec, firefox; rm -rf /";

    let report = validator.validate_config(config);

    // Should have errors (injection attempt)
    assert!(report.has_errors(), "Injection should be blocked");

    // Should have exactly one error
    assert_eq!(report.issues.len(), 1, "Should have one injection error");

    // Error should be from Layer 1
    assert!(
        report.issues[0].message.to_lowercase().contains("security"),
        "Error should mention security violation: {}",
        report.issues[0].message
    );
}

#[test]
fn test_danger_warned_by_layer2() {
    // Layer 2 should warn about dangerous (but not critical) commands
    // Note: We don't have Dangerous-level patterns yet (only Critical in Round 1)
    // So we'll test that safe commands don't trigger warnings
    let validator = ConfigValidator::new();
    let config = "bind = SUPER, K, exec, firefox";

    let report = validator.validate_config(config);

    // Should not have errors (no injection)
    assert!(!report.has_errors(), "Safe command should not have errors");

    // Should not have warnings (safe command)
    assert_eq!(report.issues.len(), 0, "Safe command should have no warnings");
}

#[test]
fn test_critical_danger_reported() {
    // Critical dangers should be reported and block commit
    let validator = ConfigValidator::new();
    let config = "bind = SUPER, K, exec, rm -rf /";

    let report = validator.validate_config(config);

    // Should have critical danger
    assert!(report.has_critical_dangers(), "rm -rf / should be critical");

    // Should NOT have errors (critical blocks commit)
    assert!(!report.has_errors(), "Critical danger should not be an error - it's a danger");

    // Should record the danger in dangerous_commands
    assert_eq!(
        report.dangerous_commands.len(), 1,
        "Should record one dangerous command"
    );

    let (_index, assessment) = &report.dangerous_commands[0];
    assert_eq!(
        assessment.danger_level,
        DangerLevel::Critical,
        "Should be Critical level"
    );
}

#[test]
fn test_both_layers_triggered() {
    // Config with both injection AND critical danger
    let validator = ConfigValidator::new();
    let config = r#"
bind = SUPER, K, exec, firefox; rm -rf /
bind = SUPER, M, exec, dd if=/dev/zero of=/dev/sda
"#;

    let report = validator.validate_config(config);

    // Should have errors from injection only
    assert!(report.has_errors(), "Should have errors");
    assert_eq!(
        report.issues.len(), 1,
        "Should have one issue (injection only - critical dangers aren't errors)"
    );

    // First binding: injection (Layer 1)
    assert!(
        report.issues[0].message.to_lowercase().contains("security"),
        "First issue should be injection"
    );

    // Should have critical danger
    assert!(report.has_critical_dangers(), "Should detect critical danger");
}

#[test]
fn test_safe_config_passes() {
    // Clean config with no issues should pass validation
    let validator = ConfigValidator::new();
    let config = r#"
# Safe configuration
bind = SUPER, K, exec, firefox
bind = SUPER, M, exec, kitty
bind = SUPER, Q, killactive
bind = SUPER, F, togglefloating
"#;

    let report = validator.validate_config(config);

    // Should have no errors
    assert!(!report.has_errors(), "Safe config should have no errors");

    // Should have no warnings
    assert_eq!(report.issues.len(), 0, "Safe config should have no issues");

    // Should have no dangers
    assert!(!report.has_critical_dangers(), "Safe config should have no dangers");
    assert_eq!(report.highest_danger, DangerLevel::Safe);
}

