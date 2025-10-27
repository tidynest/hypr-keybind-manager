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
