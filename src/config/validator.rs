// Copyright 2025 bakri (tidynest@proton.me)
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

//! Config-level validation coordinator (Layer 3 security).
//!
//! This module coordinates all validation layers:
//! - **Layer 1** (`core/validator.rs`): Injection prevention
//! - **Layer 2** (`config/danger_old`): Dangerous command detection
//! - **Layer 3** (this module): Complete config validation
//!
//! The validator parses the entire config file and runs all security
//! checks, collecting issues into a structured report that the transaction
//! system can use to decide whether to allow or block the commit.
//!
//! # Example
//! ```
//! use hypr_keybind_manager::config::validator::ConfigValidator;
//!
//! let validator = ConfigValidator::new();
//! let report = validator.validate_config("bind = SUPER, K, exec, firefox");
//!
//! if report.has_errors() {
//!     println!("Validation failed - cannot commit");
//! } else {
//!     println!("Validation passed - safe to commit");
//! }
//! ```

use crate::config::danger::{DangerAssessment, DangerDetector, DangerLevel};
use crate::core::validator as injection_validator;
use crate::core::parser::parse_config_file;
use std::path::Path;

/// Validation severity level
///
/// Determines how the transaction system should handle the issue:
/// - **Error**: Blocks commit (security violation)
/// - **Warning**: Allows commit but warns user (suspicious but not critical)
/// - **Info**: Informational only (currently unused)
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValidationLevel {
    /// Blocks commit - security violations detected
    Error,
    /// Allows commit - suspicious but not critical
    Warning,
    /// Information only
    Info,
}

/// A single validation issue found in the config
///
/// Each issue has:
/// - `binding_index`: Which binding (0-based) has the issue
/// - `level`: How severe (Error blocks, Warning allows)
/// - `message`: Human-readable description
/// - `suggestion`: Optional fix recommendation
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ValidationIssue {
    /// Index of the binding with the issue (0-based)
    pub binding_index: usize,
    /// Severity level (Error/Warning/Info)
    pub validation_level: ValidationLevel,
    /// Human-readable description of the issue
    pub message: String,
    /// Optional suggestion for fixing the issue
    pub suggestion: Option<String>,
}

/// Complete validation report for a config file
///
/// Contains all issues found during validation, organized by severity
/// and type. The transaction system uses this to decide whether to
/// allow or block the commit.
#[allow(dead_code)]
#[derive(Debug)]
pub struct ValidationReport {
    /// All validation issues (errors + warnings)
    pub issues: Vec<ValidationIssue>,
    /// Highest danger level found (for quick checks)
    pub highest_danger: DangerLevel,
    /// Dangerous commands with their assessments (for detailed reporting)
    pub dangerous_commands: Vec<(usize, DangerAssessment)>,
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationReport {
    #[allow(dead_code)]
    /// Creates a new empty validation report
    pub fn new() -> Self {
        Self {
            issues: Vec::new(),
            highest_danger: DangerLevel::Safe,
            dangerous_commands: Vec::new(),
        }
    }

    /// Returns true if the report contains any Error-level issues
    ///
    /// Error-level issues block commits. These are typically Layer 1
    /// injection attempts or invalid syntax.
    #[allow(dead_code)]
    pub fn has_errors(&self) -> bool {
        self.issues.iter().any(|issue| issue.validation_level == ValidationLevel::Error)
    }

    /// Returns true if any Critical-level dangers were detected
    ///
    /// Critical dangers (rm -rf /, dd to disk, fork bombs) always block
    /// commits, even though they don't use shell metacharacters.
    #[allow(dead_code)]
    pub fn has_critical_dangers(&self) -> bool {
        self.highest_danger == DangerLevel::Critical
    }

    /// Adds an Error-level issue to the report
    ///
    /// Errors block commits. Use this for Layer 1 injection attempts
    /// or syntax violations.
    pub fn add_error(&mut self, binding_index: usize, message: String) {
        self.issues.push(ValidationIssue {
            binding_index,
            validation_level: ValidationLevel::Error,
            message,
            suggestion: None,
        });
    }

    /// Adds a Warning-level issue to the report
    ///
    /// Warnings allow commits but inform the user. Use this for Layer 2
    /// suspicious or dangerous (but not critical) commands.
    pub fn add_warning(&mut self, binding_index: usize, message: String, suggestion: Option<String>) {
        self.issues.push(ValidationIssue {
            binding_index,
            validation_level: ValidationLevel::Warning,
            message,
            suggestion,
        });
    }

    /// Records a dangerous command assessment
    ///
    /// Stores the full DangerAssessment for detailed reporting and updates
    /// the highest danger level if necessary. Critical dangers block commits.
    pub fn record_danger(&mut self, binding_index: usize, danger_assessment: DangerAssessment) {
        // Update highest danger level
        if danger_assessment.danger_level > self.highest_danger {
            self.highest_danger = danger_assessment.danger_level;
        }

        // Store the assessment for detailed reporting
        self.dangerous_commands.push((binding_index, danger_assessment));
    }
}

/// Config validator coordinating all security layers
///
/// This is the main entry point for config validation. It coordinates
/// Layer 1 (injection) and Layer 2 (danger) validation, collecting all
/// issues into a structured report.
#[allow(dead_code)]
pub struct ConfigValidator {
    danger_detector: DangerDetector,
}

impl Default for ConfigValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigValidator {
    /// Creates a new config validator with all detection layers
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            danger_detector: DangerDetector::new(),
        }
    }

    /// Validates a complete config file content
    ///
    /// Runs all validation layers:
    /// 1. Parse the config file
    /// 2. For each binding, check Layer 1 (injection)
    /// 3. For exec dispatchers, check Layer 2 (danger)
    /// 4. Collect all issues into a report
    ///
    /// # Arguments
    ///
    /// * `content` - The complete config file content as a string
    ///
    /// # Returns
    ///
    /// A `ValidationReport` containing all issues found. The report's
    /// `has_errors()` and `has_critical_dangers()` methods indicate
    /// whether the config should be blocked.
    ///
    /// # Example
    ///
    /// ```
    /// use hypr_keybind_manager::config::validator::ConfigValidator;
    ///
    /// let validator = ConfigValidator::new();
    /// let config = "bind = SUPER, K, exec, firefox";
    /// let report = validator.validate_config(config);
    ///
    /// if report.has_errors() {
    ///     println!("Cannot commit - validation errors found");
    /// }
    /// ```
    #[allow(dead_code)]
    pub fn validate_config(&self, content: &str) -> ValidationReport {
        let mut report = ValidationReport::new();

        // Step 1: Parse the config file
        let bindings = match parse_config_file(content, Path::new("")) {
            Ok(b) => b,
            Err(e) => {
                // Parse error - add as error and return immediately
                report.add_error(0, format!("Parse error: {}", e));
                return report;
            }
        };

        // Step 2: Validate each binding
        for (binding_index, binding) in bindings.iter().enumerate() {
            // Layer 1: Injection prevention check
            if let Err(e) = injection_validator::validate_keybinding(binding) {
                report.add_error(
                    binding_index,
                    format!("Security violation: {}", e)
                );
                // Don't check Layer 2 if Layer 1 failed (injection attempt)
                continue;
            }

            // Layer 2: Danger assessment (only for exec dispatcher)
            if binding.dispatcher == "exec" {
                if let Some(args) = &binding.args {
                    let danger = self.danger_detector.assess_command(args);

                    match danger.danger_level {
                        DangerLevel::Critical => {
                            // Critical dangers - block commits
                            report.record_danger(binding_index, danger.clone());  // Records danger
                        }
                        DangerLevel::Dangerous => {
                            // Dangerous commands - warn but allow
                            report.record_danger(binding_index, danger.clone());
                            report.add_warning(
                                binding_index,
                                format!("Dangerous command: {}", danger.reason),
                                Some(danger.recommendation.clone())
                            );
                        }
                        DangerLevel::Suspicious => {
                            // Suspicious commands - warn but allow
                            report.add_warning(
                                binding_index,
                                format!("Suspicious command: {}", danger.reason),
                                Some(danger.recommendation.clone())
                            );
                        }
                        DangerLevel::Safe => {
                            // Safe - no action needed
                        }
                    }
                }
            }
        }

        report
    }
}
