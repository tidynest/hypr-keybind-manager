use crate::core::{ConflictDetector, KeyCombo, Keybinding};
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

