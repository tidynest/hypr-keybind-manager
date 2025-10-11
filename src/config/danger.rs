//! Dangerous command pattern detection (Layer 2 security).
//!
//! This module implements pattern-based detection of system-destroying
//! commands that pass injection validation but are inherently dangerous.
//!
//! # Security Layers
//! - **Layer 1** (`core/validator.rs`): Blocks shell injection syntax
//! - **Layer 2** (this module): Blocks dangerous command semantics
//! - **Layer 3** (`config/validator.rs`): Validates entire config
//! - **Layer 4** (`execution/sandbox.rs`): Sandboxed execution (Phase 3 Step 6)
//!
//! # Detection Techniques
//! 1. **Critical Pattern Matching**: Regex for system-destroying commands
//! 2. **Command Categorization**: HashSet lookup (rm, chmod, sudo)
//! 3. **Entropy Analysis**: Shannon entropy for encoded payloads
//!
//! # Future Enhancements
//! More critical patterns will be added iteratively, including:
//! - Additional disk operations (mkfs variants, fdisk)
//! - Permission escalation patterns (sudo combinations)
//! - Network-based attacks (reverse shells, port binding)
//! - Cryptomining payloads (xmrig, ethminer patterns)
//! - Systemd manipulation (systemctl disable security services)
//!
//! # References
//! - OWASP Command Injection Prevention Cheat Sheet
//! - CVE-2024-42029 (xdg-desktop-portal-hyprland)
//! - Shannon Entropy: C.E. Shannon (1948), "A Mathematical Theory of Communication"

use regex::Regex;
use std::collections::HashSet;

#[allow(dead_code)]
/// Security danger level for caommands
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DangerLevel {
    /// No known security concerns
    Safe = 0,
    /// Potentially risky but may have legitimate uses
    Suspicious = 1,
    /// Can cause significant damage (file deletion, permission changes)
    Dangerous = 2,
    /// Immediate system destruction (rm -rf /, dd to disk, fork bombs)
    Critical = 3,
}

/// Assessment result with contextual information
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub struct DangerAssessment {
    /// Assessed danger level
    pub danger_level: DangerLevel,
    /// Human-readable explanation of the risk
    pub reason: String,
    /// Suggested mitigation or alternative
    pub recommendation: String,
    /// The specific pattern that matched (if any)
    pub matched_pattern: Option<String>,
}

#[allow(dead_code)]
/// Pattern-based dangerous command detector
pub struct DangerDetector {
    critical_patterns:   Vec<Regex>,
    dangerous_commands:  HashSet<String>,
    suspicious_commands: HashSet<String>,
    safe_commands:       HashSet<String>,
}

impl DangerDetector {
    /// Creates a new detector with all patterns loaded
    pub fn new() -> Self {
        Self {
            critical_patterns: Self::build_critical_patterns(),
            dangerous_commands: Self::build_dangerous_commands(),
            suspicious_commands: Self::build_suspicious_commands(),
            safe_commands: Self::build_safe_commands(),
        }
    }

    /// Assesses the danger level of a command string
    ///
    /// Checks critical patterns first (Round 1), then will check command
    /// categorization (Round 2) and entropy (Round 3) in future iterations.
    pub fn assess_command(&self, command: &str) -> DangerAssessment {
        // Round 1: Check critical patterns (system-destroying commands)
        for (i, pattern) in self.critical_patterns.iter().enumerate() {
            if pattern.is_match(command) {
                return match i {
                    0 | 1 => DangerAssessment {  // Both rm patterns
                        danger_level: DangerLevel::Critical,
                        reason: "Recursive filesystem deletion from root directory".to_string(),
                        recommendation: "NEVER execute this command. It will destroy your entire system.".to_string(),
                        matched_pattern: Some("rm -rf /".to_string()),
                    },
                    2 => DangerAssessment {  // dd pattern
                        danger_level: DangerLevel::Critical,
                        reason: "Direct write to disk device - will destroy all data and partition table".to_string(),
                        recommendation: "Remove this keybinding immediately. This overwrites raw disk sectors.".to_string(),
                        matched_pattern: Some("dd to disk device".to_string()),
                    },
                    3 => DangerAssessment {  // Fork bomb
                        danger_level: DangerLevel::Critical,
                        reason: "Fork bomb detected - exponential process spawning".to_string(),
                        recommendation: "This will crash or hang your system. Remove immediately.".to_string(),
                        matched_pattern: Some("fork bomb".to_string()),
                    },
                    _ => unreachable!("Pattern index out of range."),
                };
            }
        }

        // No critical patterns matched - safe for now
        // (Round 2 and 3 checks will be added here later
        DangerAssessment {
            danger_level: DangerLevel::Safe,
            reason: "No dangerous patterns detected".to_string(),
            recommendation: String::new(),
            matched_pattern: None,
        }
    }


    /// Builds regex patterns for critical system-destroying commands
    ///
    /// Round 1: Three critical patterns
    /// - Pattern 0: rm -rf / (filesystem destruction)
    /// - Pattern 1: dd to disk (disk device overwrite)
    /// - Pattern 2: Fork bomb (resource exhaustion)
    fn build_critical_patterns() -> Vec<Regex> {
        vec![
            // Pattern 0a: rm -rf / (r before f)
            // Matches: rm + space + flags with 'r'/'R' then 'f'/'F' + "/" + end
            // Examples: "rm -rf /", "rm -Rf /", "rm -r -f /"
            Regex::new(r"rm\s+.*[rR].*[fF].*\s+/\s*$")
                .expect("rm -rf / pattern (r before f) should be valid regex"),

            // Pattern 0b: rm -fr / (f before r)
            // Matches: rm + space + flags with 'f'/'F' then 'r'/'R' + "/" + end
            // Examples: "rm -fr /", "rm -fR /", "rm -f -r /"
            Regex::new(r"rm\s+.*[fF].*[rR].*\s+/\s*$")
                .expect("rm -fr / pattern (f before r) should be valid regex"),

            // Pattern 1: dd to disk devices
            // Matches: dd + "of=" + SATA (sda-sdz) or NVMe (nvme0n1, nvme1n1, etc.)
            // Examples: "dd if=/dev/zero of=/dev/sda", "dd if=/dev/urandom of=/dev/nvme0n1"
            Regex::new(r"dd\s+.*of=/dev/(sd[a-z]|nvme\d+n\d+)")
                .expect("dd to disk pattern should be valid regex"),

            // Pattern 2: Fork bomb
            // Matches: :(){ :|:& };: with optional whitespace
            // This bash syntax creates exponential process spawning
            Regex::new(r":\(\)\s*\{\s*:\s*\|\s*:\s*&\s*\}\s*;\s*:")
                .expect("fork bomb pattern should be valid regex"),
        ]
    }

    fn build_dangerous_commands() -> HashSet<String> {
        // TODO: Implement later
        HashSet::new()
    }

    fn build_suspicious_commands() -> HashSet<String> {
        // TODO: Implement later
        HashSet::new()
    }

    fn build_safe_commands() -> HashSet<String> {
        // TODO: Implement later
        HashSet::new()
    }

    #[allow(dead_code)]
    /// Calculate Shannon entropy of a string (bits per character)
    fn calculate_entropy(&self, _s: &str) -> f32 {
        // TODO: Implement in Round 3
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // ROUND 1: Critical Pattern Detection
    // ========================================================================

    #[test]
    fn test_detect_rm_rf_root_critical() {
        let detector = DangerDetector::new();

        // Test various forms of "rm -rf /"
        let test_cases = vec![
            "rm -rf /",         // Standard form
            "rm -rf / ",        // Trailing space
            "rm -Rf /",         // Capital R
            "rm -Rf / ",        // Capital R + Trailing space
            "rm -fR /",         // Reversed flags
            "rm -fR / ",        // Reversed flags + Trailing space
            "rm -r -f /",       // Separated flags
            "rm -r -f / ",      // Separated flags + Trailing space
        ];

        for command in test_cases {
            let assessment = detector.assess_command(command);

            assert_eq!(
                assessment.danger_level,
                DangerLevel::Critical,
                "Command '{}´ should be Critical danger",
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
            "dd if=/dev/zero of=/dev/sda",      // Standard SATA disk
            "dd if=/dev/zero of=/dev/sdb",      // Second disk
            "dd if=/dev/urandom of=/dev/sda",   // Random data
            "dd if=/dev/zero of=/dev/nvme0n1",  // NVMe disk
            "dd if=/dev/zero of=/dev/nvme1n1",  // Second NVMe
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

            assert!(
                assessment.recommendation.to_lowercase().contains("never")
                    || assessment.recommendation.to_lowercase().contains("remove"),
                "Should recommend never executing this: {}",
                assessment.recommendation
            );
        }
    }

    #[test]
    fn test_detect_fork_bomb_critical() {
        let detector = DangerDetector::new();

        // Fork bomb: :(){ :|:& };:
        // This creates a function that calls itself twice,
        // exponentially consuming all system resources
        let fork_bomb = ":(){ :|:& };:";

        let assessment = detector.assess_command(fork_bomb);

        assert_eq!(
            assessment.danger_level,
            DangerLevel::Critical,
            "Fork bomb should be Critical danger"
        );

        assert!(
            assessment.reason.to_lowercase().contains("fork")
                || assessment.reason.to_lowercase().contains("resource"),
            "Should identify fork bomb or resource exhaustion: {}",
            assessment.reason
        );

        assert!(
            assessment.recommendation.to_lowercase().contains("crash")
                || assessment.recommendation.to_lowercase().contains("hang"),
            "Should warn about system crash/hang: {}",
            assessment.recommendation
        );
    }
}


