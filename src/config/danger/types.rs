//! Security danger level types and assessments

/// Security danger level for commands
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DangerLevel {
    /// No known security concerns (whitelisted commands)
    Safe = 0,
    /// Potentially risky but may have legitimate uses (encoding tools, downloaders)
    Suspicious = 1,
    /// Can cause significant damage (file deletion, permission changes, privilege escalation)
    Dangerous = 2,
    /// Immediate system destruction (rm -rf /, dd to disk, fork bombs)
    Critical = 3,
}

/// Assessment result with contextual information
#[derive(Clone, Debug, PartialEq)]
pub struct DangerAssessment {
    /// Assessed danger level
    pub danger_level: crate::config::danger::DangerLevel,
    /// Human-readable explanation of the risk
    pub reason: String,
    /// Suggested mitigation or alternative
    pub recommendation: String,
    /// The specific pattern that matched (if any)
    pub matched_pattern: Option<String>,
}
