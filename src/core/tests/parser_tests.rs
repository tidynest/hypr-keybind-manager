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

//! Parser module tests
//!
//! Tests for parsing Hyprland configuration files:
//! - Bind type parsing (bind, binde, bindel, etc.)
//! - Modifier parsing (SUPER, CTRL, SHIFT, ALT)
//! - Key combination parsing
//! - Dispatcher and arguments parsing
//! - Variable substitution
//! - Full config file parsing

use crate::core::{
    parser::*,
    types::{BindType, Modifier},
};
use std::path::Path;

#[test]
fn test_parse_bind_type() {
    assert!(matches!(
        parse_bind_type("bind = SUPER, K"),
        Ok((_, BindType::Bind))
    ));
    assert!(matches!(
        parse_bind_type("binde = SUPER, K"),
        Ok((_, BindType::BindE))
    ));
    assert!(matches!(
        parse_bind_type("bindel = SUPER, K"),
        Ok((_, BindType::BindEL))
    ));
}

#[test]
fn test_parse_modifiers() {
    let mods = parse_modifiers("SUPER").unwrap();
    assert_eq!(mods.len(), 1);
    assert_eq!(mods[0], Modifier::Super);

    let mods = parse_modifiers("SUPER_SHIFT").unwrap();
    assert_eq!(mods.len(), 2);

    let mods = parse_modifiers("SUPER SHIFT").unwrap();
    assert_eq!(mods.len(), 2);
}

#[test]
fn test_dispatcher() {
    let (_, (disp, args)) = parse_dispatcher("exec, firefox").unwrap();
    assert_eq!(disp, "exec");
    assert_eq!(args, Some("firefox".to_string()));

    let (_, (disp, args)) = parse_dispatcher("killactive").unwrap();
    assert_eq!(disp, "killactive");
    assert_eq!(args, None);
}

#[test]
fn test_parse_bind_line() {
    let result = parse_bind_line("bind = SUPER, K, exec, firefox");
    assert!(result.is_ok());

    let (_, binding) = result.unwrap();
    assert!(matches!(binding.bind_type, BindType::Bind));
    assert_eq!(binding.key_combo.key, "K");
    assert_eq!(binding.dispatcher, "exec");
    assert_eq!(binding.args, Some("firefox".to_string()));
}

#[test]
fn test_variable_substitution() {
    let content = "$mainMod = SUPER\nbind = $mainMod, K, exec, firefox";
    let vars = collect_variables(content);
    assert_eq!(vars.get("mainMod"), Some(&"SUPER".to_string()));

    let substituted = substitute_variables("bind = $mainMod, K", &vars);
    assert_eq!(substituted, "bind = SUPER, K");
}

#[test]
fn test_parse_config_file() {
    let config = r#"
# Comment line
$mainMod = SUPER

bind = $mainMod, K, exec, firefox
binde = $mainMod SHIFT, R, exec, wofi
"#;
    let result = parse_config_file(config, Path::new("test.conf"));
    assert!(result.is_ok());

    let bindings = result.unwrap();
    assert_eq!(bindings.len(), 2);
}
