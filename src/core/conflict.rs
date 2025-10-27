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

//! Real-time keybinding conflict detection
//!
//! This module implements O(1) conflict detection using HashMap-based
//! indexing. When multiple keybindings use the same key combination,
//! they are flagged as conflicts for user resolution.
//!
//! # Performance
//! - Add binding: O(1) average case
//! - Check conflict: O(1) average case
//! - List all conflicts: O(n) where n = number of unique key combos
//!
//! For typical configs (100-500 bindings), conflict checking completes
//! in <5 microseconds.

use std::collections::HashMap;
use crate::core::types::{Keybinding, KeyCombo};

/// Detects keybinding conflicts in O(1) time using HashMap-based indexing.
///
/// Uses a HashMap where keys are KeyCombos and values are vectors of all
/// bindings using that combo. A conflict exists when any vector has length > 1.
pub struct ConflictDetector {
    /// Maps KeyCombo to all bindings using that combination.
    bindings: HashMap<KeyCombo, Vec<Keybinding>>,
}

/// Represents a detected conflict between keybindings.
#[derive(Clone, Debug, PartialEq)]
pub struct Conflict {
    /// The key combination that has conflicts
    pub key_combo: KeyCombo,

    /// All bindings using this key combo (always 2 or more)
    pub conflicting_bindings: Vec<Keybinding>,
}

impl ConflictDetector {
    /// Creates a new empty conflict detector.
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    /// Adds a keybinding to the detector.
    ///
    /// Time complexity: O(1) average case
    pub fn add_binding(&mut self, binding: Keybinding) {
        // Clone KeyCombo for HashMap ownership
        self.bindings
            .entry(binding.key_combo.clone())
            .or_default()
            .push(binding);
    }

    /// Finds all conflicts (key combos with 2 or more bindings).
    ///
    /// Time complexity: O(n) where n = number of unique key combos.
    pub fn find_conflicts(&self) -> Vec<Conflict> {
        self.bindings
            .iter()
            .filter(|(_, bindings)| bindings.len() > 1)
            .map(|(key_combo, bindings)| Conflict{
                key_combo: key_combo.clone(),
                conflicting_bindings: bindings.clone(),
            })
            .collect()
    }

    /// Checks if a specific key combo has conflicts.
    ///
    /// Returns true if this KeyCombo has 2 or more bindings.
    pub fn has_conflict(&self, key_combo: &KeyCombo) -> bool {
        self.bindings
            .get(key_combo)
            .map(|bindings| bindings.len() > 1)
            .unwrap_or(false)
    }

    /// Returns the total number of bindings tracked.
    pub fn total_bindings(&self) -> usize {
        self.bindings.values().map(|v| v.len()).sum()
    }
}

impl Default for ConflictDetector {
    fn default() -> Self {
        Self::new()
    }
}

