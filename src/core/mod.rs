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

//! src/core/mod.rs
//!
//! Core business logic module
//!
//! This module contains the fundamental data structures and algorithms
//! for keybinding management, including:
//! - Type definitions for keybindings and key combinations
//! - Conflict detection using HashMap-based O(1) lookup
//! - Input validation with security whitelisting
//! - Configuration parsing
//!
//! All business logic is isolated from UI and I/O concerns to enable
//! comprehensive unit testing without requiring a display server.

pub mod conflict;
pub mod parser;
pub mod types;
pub mod validator;

pub use conflict::{ConflictDetector, Conflict};
pub use types::*;
pub use validator::{validate_keybinding, ValidationError};

#[cfg(test)]
mod tests;
