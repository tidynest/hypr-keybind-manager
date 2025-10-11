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
            critical_patterns:   Self::build_critical_patterns(),
            dangerous_commands:  Self::build_dangerous_commands(),
            suspicious_commands: Self::build_suspicious_commands(),
            safe_commands:       Self::build_safe_commands(),
        }
    }

    /// Assesses the danger level of a command string
    pub fn assess_command(&self, _command: &str) -> DangerAssessment {
        // TODO: Implement in response to failing tests
        todo!("assess command not yet implemented")
    }

    /// Builds regex patterns for critical system-destroying commands
    fn build_critical_patterns() -> Vec<Regex> {
        // TODO: Implement in response to failing tests
        Vec::new()
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


