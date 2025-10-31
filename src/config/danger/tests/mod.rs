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

//! Dangerous command detection tests (Layer 2 security)
//!
//! Contains test suites for pattern-based danger detection:
//! - Pattern tests (critical patterns, dangerous commands, safe whitelist)
//! - Entropy tests (Shannon entropy, base64/hex encoding detection)
//! - Integration tests (end-to-end danger assessment)

#[cfg(test)]
mod entropy_tests;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod patterns_tests;
