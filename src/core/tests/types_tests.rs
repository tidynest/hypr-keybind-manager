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

use crate::core::types::{BindType, KeyCombo, Keybinding, Modifier};

#[test]
fn test_modifier_display() {
    assert_eq!(format!("{}", Modifier::Super), "SUPER");
    assert_eq!(format!("{}", Modifier::Ctrl), "CTRL");
}

#[test]
fn test_bind_type_display() {
    assert_eq!(format!("{}", BindType::Bind), "bind");
    assert_eq!(format!("{}", BindType::BindEL), "bindel");
}

#[test]
fn test_key_combo_normalization() {
    // Test that order doesn't matter
    let combo1 = KeyCombo::new(vec![Modifier::Super, Modifier::Shift], "k");
    let combo2 = KeyCombo::new(vec![Modifier::Shift, Modifier::Super], "K");

    assert_eq!(combo1, combo2); // Should be equal after normalization
}

#[test]
fn test_key_combo_display() {
    let combo = KeyCombo::new(vec![Modifier::Super, Modifier::Shift], "K");
    let display = format!("{}", combo);

    // Should show modifiers in sorted order
    assert!(display.contains("SUPER"));
    assert!(display.contains("SHIFT"));
    assert!(display.contains("K"));
}

#[test]
fn test_keybinding_display() {
    let binding = Keybinding {
        key_combo: KeyCombo::new(vec![Modifier::Super], "K"),
        bind_type: BindType::Bind,
        dispatcher: "exec".to_string(),
        args: Some("firefox".to_string()),
    };

    let display = format!("{}", binding);
    assert!(display.contains("bind"));
    assert!(display.contains("SUPER"));
    assert!(display.contains("K"));
    assert!(display.contains("exec"));
    assert!(display.contains("firefox"));
}

#[test]
fn test_keybinding_no_args() {
    let binding = Keybinding {
        key_combo: KeyCombo::new(vec![Modifier::Super], "Q"),
        bind_type: BindType::Bind,
        dispatcher: "killactive".to_string(),
        args: None,
    };

    let display = format!("{}", binding);
    assert!(display.contains("killactive"));
    assert!(!display.ends_with(",")); // No trailing comma when no args
}
