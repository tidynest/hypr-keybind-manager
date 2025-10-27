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
//! # Detection Techniques (All Rounds Complete ✅)
//! 1. **Round 1 - Critical Pattern Matching**: Regex for system-destroying commands
//! 2. **Round 2 - Command Categorisation**: HashSet lookup (rm, chmod, sudo)
//! 3. **Round 3 - Entropy Analysis**: Shannon entropy for encoded payloads ✅
//!
//! # Round 3: Shannon Entropy Detection
//!
//! Uses information theory to detect obfuscated/encoded commands.
//!
//! ## Empirically Validated Thresholds
//!
//! Based on analysis of 30 test cases, we use **empirical thresholds** rather than
//! theoretical maximums:
//!
//! | Encoding | Theoretical Max | Measured Range | Our Threshold |
//! |----------|----------------|----------------|---------------|
//! | Base64   | 6.0 bits/char  | 2.5-4.5 bits   | **4.0 bits**  |
//! | Hex      | 4.0 bits/char  | 2.0-4.0 bits   | **3.0 bits**  |
//! | Normal text | ~4.7 bits/char | 2.0-4.2 bits | (baseline)    |
//!
//! **Why the gap?** Real-world encoded commands have lower entropy than theory predicts
//! due to:
//! - Short string length (10-50 chars vs infinite assumed by theory)
//! - Source data patterns (commands have structure: `/bin/bash`, `rm -rf`)
//! - Padding and special characters (base64 `=` padding reduces entropy)
//!
//! ## Detection Order Matters
//!
//! We check **hex before base64** because the hex alphabet is a subset of base64:
//! ```text
//! [0-9a-fA-F] ⊂ [A-Za-z0-9+/=]
//! ```
//! If we checked base64 first, hex-encoded attacks would be misclassified.
//!
//! ## Comprehensive Documentation
//!
//! For detailed theory, implementation rationale, and empirical findings, see:
//! [`../docs/ENTROPY_DETECTION.md`](../docs/ENTROPY_DETECTION.md)
//!
//! Topics covered in the detailed documentation:
//! - Shannon's entropy formula and information theory foundations
//! - Why theoretical maximums don't match reality
//! - Threshold selection methodology (with all 30 test cases)
//! - False positive prevention strategies
//! - Detection order rationale (hex → base64 → tools)
//!
//! # Detection Architecture
//!
//! Commands are categorised into four security levels:
//! - **Safe**: Whitelisted commands (browsers, editors, system tools)
//! - **Suspicious**: Often malicious (base64, wget, high entropy) but has legitimate uses
//! - **Dangerous**: Serious security risk (chmod 777, sudo, pipe to shell)
//! - **Critical**: Immediate system destruction (rm -rf /, dd, fork bombs)
//!
//! # Detection Strategy (6-Step Process)
//!
//! 1. **Fast path**: Check safe whitelist first (O(1) HashSet lookup)
//! 2. **Critical patterns**: Regex matching for system destruction (Round 1)
//! 3. **Dangerous arguments**: Context-aware analysis (chmod 777, pipe to shell)
//! 4. **Dangerous commands**: Word boundary matching (privilege escalation, disk ops)
//! 5. **Entropy analysis**: Mathematical detection of encoded payloads (Round 3) ✅
//! 6. **Suspicious commands**: Flag encoding tools, downloaders (after entropy check)
//!
//! # References
//!
//! - **Comprehensive entropy documentation**: [`../docs/ENTROPY_DETECTION.md`](../docs/ENTROPY_DETECTION.md)
//! - **OWASP**: Command Injection Prevention Cheat Sheet
//! - **CVE-2024-42029**: xdg-desktop-portal-hyprland vulnerability
//! - **Shannon (1948)**: "A Mathematical Theory of Communication"
//! - **MITRE ATT&CK T1059**: Command and Scripting Interpreter
//! - **MITRE ATT&CK T1027**: Obfuscated Files or Information

use regex::Regex;
use std::collections::HashSet;

pub mod types;
pub mod patterns;
pub mod entropy;

pub use types::{DangerAssessment, DangerLevel};

/// Pattern-based dangerous command detector
pub struct DangerDetector {
    critical_patterns: Vec<Regex>,
    dangerous_commands: HashSet<String>,
    suspicious_commands: HashSet<String>,
    safe_commands: HashSet<String>,
}

impl Default for DangerDetector {
    fn default() -> Self {
        Self {
            critical_patterns:   patterns::build_critical_patterns(),
            dangerous_commands:  patterns::build_dangerous_commands(),
            suspicious_commands: patterns::build_suspicious_commands(),
            safe_commands:       patterns::build_safe_commands(),
        }
    }
}

impl DangerDetector {
    /// Creates a new detector with all patterns loaded
    pub fn new() -> Self {
        Self::default()
    }

    /// Assesses the danger level of a command string
    ///
    /// # Detection Order (Optimised for Performance and Accuracy)
    /// 1. **Safe whitelist** - Fast path for known-safe commands (O(1))
    /// 2. **Critical patterns** - System-destroying regex (Round 1)
    /// 3. **Dangerous arguments** - Secondary pattern analysis (chmod 777, etc.)
    /// 4. **Dangerous commands** - Word boundary matching (privilege escalation)
    /// 5. **Entropy analysis** - Detect encoded payloads BEFORE suspicious tools (Round 3)
    /// 6. **Suspicious commands** - Flag obfuscation tools (after entropy check)
    ///
    /// # Examples
    /// ```
    /// use hypr_keybind_manager::config::danger::{DangerDetector, DangerLevel};
    ///
    /// let detector = DangerDetector::new();
    ///
    /// // Safe command
    /// let assessment = detector.assess_command("firefox");
    /// assert_eq!(assessment.danger_level, DangerLevel::Safe);
    ///
    /// // Dangerous command
    /// let assessment = detector.assess_command("chmod 777 ~/.ssh");
    /// assert_eq!(assessment.danger_level, DangerLevel::Dangerous);
    /// ```
    pub fn assess_command(&self, command: &str) -> DangerAssessment {
        // Step 1: Fast path - Check safe whitelist first
        // This is O(1) and avoids unnecessary checks for common commands
        let words: Vec<&str> = command.split_whitespace().collect();
        if let Some(first_word) = words.first() {
            if self.safe_commands.contains(*first_word) {
                return DangerAssessment {
                    danger_level: DangerLevel::Safe,
                    reason: "Known safe command".to_string(),
                    recommendation: String::new(),
                    matched_pattern: None,
                };
            }
        }

        // Step 2: Check critical patterns (Round 1 - system destruction)
        for (i, pattern) in self.critical_patterns.iter().enumerate() {
            if pattern.is_match(command) {
                return match i {
                    0 | 1 => DangerAssessment {
                        danger_level: DangerLevel::Critical,
                        reason: "Recursive filesystem deletion from root directory".to_string(),
                        recommendation:
                        "NEVER execute this command. It will destroy your entire system."
                            .to_string(),
                        matched_pattern: Some("rm -rf /".to_string()),
                    },
                    2 => DangerAssessment {
                        danger_level: DangerLevel::Critical,
                        reason: "Direct write to disk device - will destroy all data and partition table"
                            .to_string(),
                        recommendation:
                        "Remove this keybinding immediately. This overwrites raw disk sectors."
                            .to_string(),
                        matched_pattern: Some("dd to disk device".to_string()),
                    },
                    3 => DangerAssessment {
                        danger_level: DangerLevel::Critical,
                        reason: "Fork bomb detected - exponential process spawning".to_string(),
                        recommendation: "This will crash or hang your system. Remove immediately."
                            .to_string(),
                        matched_pattern: Some("fork bomb".to_string()),
                    },
                    _ => unreachable!("Pattern index out of range."),
                };
            }
        }

        // Step 3: Check for dangerous argument patterns (secondary analysis)
        // Some commands are only dangerous with specific arguments
        if let Some(assessment) = self.check_dangerous_arguments(command) {
            return assessment;
        }

        // Step 4: Check dangerous commands (word boundary matching)
        // Use word splitting to avoid false positives (e.g., "chmod" in comments)
        for word in &words {
            if self.dangerous_commands.contains(*word) {
                return DangerAssessment {
                    danger_level: DangerLevel::Dangerous,
                    reason: format!(
                        "Command '{}' can cause serious security issues or data loss",
                        word
                    ),
                    recommendation:
                    "Review this command carefully. Consider safer alternatives or additional safeguards."
                        .to_string(),
                    matched_pattern: Some(word.to_string()),
                };
            }
        }

        // Step 5: Check for encoded/obfuscated content FIRST (before checking tool names)
        // This ensures we catch the actual malicious payload, not just the tool
        let full_command = command.to_string();

        // Check individual tokens for encoded data
        // IMPORTANT: Check hex BEFORE base64 because hex alphabet is subset of base64
        for word in &words {
            // Skip very short tokens to reduce false positives
            if word.len() < 8 {
                continue;
            }

            // Skip command names themselves (they shouldn't be treated as encoded data)
            // This prevents false positives on commands like "uuencode", "base64", etc.
            if self.suspicious_commands.contains(*word)
                || self.dangerous_commands.contains(*word)
                || self.safe_commands.contains(*word) {
                continue;
            }

            // Check for hex-encoded data FIRST (more restrictive alphabet)
            if entropy::is_likely_hex(word) {
                return DangerAssessment {
                    danger_level: DangerLevel::Suspicious,
                    reason: format!(
                        "Possible hex-encoded data detected: '{}'. \
                         High entropy suggests obfuscation.",
                        word
                    ),
                    recommendation: format!(
                        "Decode and inspect before executing: echo {} | xxd -r -p",
                        word
                    ),
                    matched_pattern: Some("hex encoding".to_string()),
                };
            }

            // Check for base64-encoded data (after hex check)
            if entropy::is_likely_base64(word) {
                return DangerAssessment {
                    danger_level: DangerLevel::Suspicious,
                    reason: format!(
                        "Possible base64-encoded command detected: '{}'. \
                         This may hide malicious intent.",
                        word
                    ),
                    recommendation: format!(
                        "Decode and inspect before executing: echo {} | base64 -d",
                        word
                    ),
                    matched_pattern: Some("base64 encoding".to_string()),
                };
            }
        }

        // Also check the full command line for quoted encoded strings
        // (catches cases like: perl -e 'print pack("H*", "726d202d7266202f")')
        let quoted_strings: Vec<&str> = full_command
            .split('"')
            .enumerate()
            .filter_map(|(i, s)| if i % 2 == 1 { Some(s) } else { None })
            .collect();

        for quoted in quoted_strings {
            if quoted.len() >= 8 {
                // Check hex FIRST (more restrictive)
                if entropy::is_likely_hex(quoted) {
                    return DangerAssessment {
                        danger_level: DangerLevel::Suspicious,
                        reason: format!(
                            "Possible hex-encoded payload in quotes: \"{}\". \
                             High entropy suggests obfuscation.",
                            quoted
                        ),
                        recommendation: "Decode and inspect the quoted string before executing."
                            .to_string(),
                        matched_pattern: Some("hex in quotes".to_string()),
                    };
                }

                // Then check base64
                if entropy::is_likely_base64(quoted) {
                    return DangerAssessment {
                        danger_level: DangerLevel::Suspicious,
                        reason: format!(
                            "Possible base64-encoded payload in quotes: \"{}\". \
                             This may hide malicious commands.",
                            quoted
                        ),
                        recommendation: "Decode and inspect the quoted string before executing."
                            .to_string(),
                        matched_pattern: Some("base64 in quotes".to_string()),
                    };
                }
            }
        }

        // Step 6: Check for suspicious command tools (after checking encoded content)
        // These are tools that might be legitimate but often appear in attacks
        for word in &words {
            if self.suspicious_commands.contains(*word) {
                return DangerAssessment {
                    danger_level: DangerLevel::Suspicious,
                    reason: format!(
                        "Command '{}' is often used in malicious contexts but may be legitimate",
                        word
                    ),
                    recommendation: "Verify this command is necessary. Ensure you trust its source."
                        .to_string(),
                    matched_pattern: Some(word.to_string()),
                };
            }
        }

        // No patterns matched - safe
        DangerAssessment {
            danger_level: DangerLevel::Safe,
            reason: "No dangerous patterns detected".to_string(),
            recommendation: String::new(),
            matched_pattern: None,
        }
    }

    /// Checks for dangerous argument patterns (secondary analysis)
    ///
    /// Some commands are only dangerous with specific arguments:
    /// - `chmod 777` - World-writable (bad)
    /// - `chmod 644` - Normal permissions (fine)
    ///
    /// This function performs more expensive pattern matching only when
    /// basic command name suggests potential danger.
    fn check_dangerous_arguments(&self, command: &str) -> Option<DangerAssessment> {
        // Pattern 1: chmod 777 (world-writable, world-executable)
        if command.contains("chmod") && command.contains("777") {
            return Some(DangerAssessment {
                danger_level: DangerLevel::Dangerous,
                reason: "Setting 777 permissions makes files world-writable and executable".to_string(),
                recommendation: "Use restrictive permissions like 644 (files) or 755 (executables). NEVER use 777.".to_string(),
                matched_pattern: Some("chmod 777".to_string()),
            });
        }

        // Pattern 2: Pipe to shell (classic RCE pattern)
        // Matches: "| sh", "| bash", "curl | sh", "wget | bash"
        if (command.contains("| sh") || command.contains("| bash"))
            && (command.contains("curl") || command.contains("wget") || command.contains("fetch"))
        {
            return Some(DangerAssessment {
                danger_level: DangerLevel::Dangerous,
                reason: "Downloading and executing untrusted code (Remote Code Execution pattern)"
                    .to_string(),
                recommendation:
                "Download first, inspect the script, then execute manually if safe.".to_string(),
                matched_pattern: Some("pipe to shell".to_string()),
            });
        }

        // Pattern 3: Recursive rm (not root, but still dangerous)
        if command.contains("rm") && (command.contains("-rf") || command.contains("-fr")) {
            return Some(DangerAssessment {
                danger_level: DangerLevel::Dangerous,
                reason: "Recursive file deletion - can destroy entire directories".to_string(),
                recommendation: "Double-check the path. Consider using 'trash' command instead for reversibility.".to_string(),
                matched_pattern: Some("rm -rf".to_string()),
            });
        }

        // Pattern 4: Firewall manipulation
        if command.contains("iptables") && command.contains("-F") {
            return Some(DangerAssessment {
                danger_level: DangerLevel::Dangerous,
                reason: "Flushing firewall rules removes all network protection".to_string(),
                recommendation: "Only do this if you understand the security implications.".to_string(),
                matched_pattern: Some("iptables -F".to_string()),
            });
        }

        None
    }
}

#[cfg(test)]
mod tests;