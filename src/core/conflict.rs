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
    /// Time complexity: 0(1) average case
    pub fn add_binding(&mut self, binding: Keybinding) {
        // Clone KeyCombo for HashMap ownership
        self.bindings
            .entry(binding.key_combo.clone())
            .or_default()
            .push(binding);
    }

    /// Finds all conflicts (key combos with 2 or more bindings).
    ///
    /// Time complexity: 0(n) where n = number of unique key combos.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{BindType, Modifier};

    /// Helper to create test bindings
    fn test_binding(modifiers: Vec<Modifier>, key: &str, app: &str) -> Keybinding {
        Keybinding {
            key_combo: KeyCombo::new(modifiers, key),
            bind_type: BindType::Bind,
            dispatcher: "exec".to_string(),
            args: Some(app.to_string()),
        }
    }

    #[test]
    fn test_no_conflicts_when_empty() {
        let detector = ConflictDetector::new();
        assert_eq!(detector.find_conflicts().len(), 0);
        assert_eq!(detector.total_bindings(), 0);
    }

    #[test]
    fn test_no_conflicts_with_unique_bindings() {
        let mut detector = ConflictDetector::new();

        detector.add_binding(test_binding(vec![Modifier::Super], "K", "firefox"));
        detector.add_binding(test_binding(vec![Modifier::Super], "J", "kitty"));
        detector.add_binding(test_binding(vec![Modifier::Super, Modifier::Shift], "K", "chrome"));

        assert_eq!(detector.find_conflicts().len(), 0);
        assert_eq!(detector.total_bindings(), 3);
    }

    #[test]
    fn test_detects_simple_conflict() {
        let mut detector = ConflictDetector::new();

        // Same key combo, different actions
        detector.add_binding(test_binding(vec![Modifier::Super], "K", "firefox"));
        detector.add_binding(test_binding(vec![Modifier::Super], "K", "chrome"));

        let conflicts = detector.find_conflicts();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflicting_bindings.len(), 2);

        let expected_combo = KeyCombo::new(vec![Modifier::Super], "K");
        assert_eq!(conflicts[0].key_combo, expected_combo);
    }

    #[test]
    fn test_detects_triple_conflict() {
        let mut detector = ConflictDetector::new();

        detector.add_binding(test_binding(vec![Modifier::Super], "K", "firefox"));
        detector.add_binding(test_binding(vec![Modifier::Super], "K", "chrome"));
        detector.add_binding(test_binding(vec![Modifier::Super], "K", "brave"));

        let conflicts = detector.find_conflicts();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflicting_bindings.len(), 3);
    }

    #[test]
    fn test_modifier_order_independence() {
        let mut detector = ConflictDetector::new();

        // SUPER+SHIFT vs SHIFT+SUPER detected as same combo
        // Works because KeyCombo::new() sorts modifiers
        detector.add_binding(test_binding(
            vec![Modifier::Super, Modifier::Shift],
            "K",
            "app1"
        ));

        detector.add_binding(test_binding(
            vec![Modifier::Shift, Modifier::Super],
            "K",
            "app2"
        ));

        let conflicts = detector.find_conflicts();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflicting_bindings.len(), 2);
    }

    #[test]
    fn test_has_conflict_method() {
        let mut detector = ConflictDetector::new();

        let combo = KeyCombo::new(vec![Modifier::Super], "K");

        detector.add_binding(test_binding(vec![Modifier::Super], "K", "firefox"));
        assert!(!detector.has_conflict(&combo));

        detector.add_binding(test_binding(vec![Modifier::Super], "K", "chrome"));
        assert!(detector.has_conflict(&combo));
    }

    #[test]
    fn test_multiple_independent_conflicts() {
        let mut detector = ConflictDetector::new();

        // Conflict 1: SUPER+K
        detector.add_binding(test_binding(vec![Modifier::Super], "K", "firefox"));
        detector.add_binding(test_binding(vec![Modifier::Super], "K", "chrome"));

        // Conflict 2: SUPER+J
        detector.add_binding(test_binding(vec![Modifier::Super], "J", "kitty"));
        detector.add_binding(test_binding(vec![Modifier::Super], "J", "alacritty"));

        // No conflict: SUPER+L
        detector.add_binding(test_binding(vec![Modifier::Super], "L", "swaylock"));

        let conflicts = detector.find_conflicts();
        assert_eq!(conflicts.len(), 2);
        assert_eq!(detector.total_bindings(), 5);
    }

    #[test]
    fn test_total_bindings_count() {
        let mut detector = ConflictDetector::new();

        detector.add_binding(test_binding(vec![Modifier::Super], "K", "firefox"));
        detector.add_binding(test_binding(vec![Modifier::Super], "J", "kitty"));
        detector.add_binding(test_binding(vec![Modifier::Super], "K", "chrome"));

        assert_eq!(detector.total_bindings(), 3);
    }
}
