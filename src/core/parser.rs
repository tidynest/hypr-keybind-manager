//! src/core/parser.rs
//!
//! Hyprland configuration file parser
//!
//! This module parses Hyprland config files to extract keybindings.
//! It handles:
//! - All bind types (bind, binde, bindl, bindm, bindr, bindel)
//! - Variable substitution ($mainMod)
//! - Comments and whitespace
//! - Line numbers for error reporting
//!
//! # Architecture
//! The parser uses nom combinators for composable, type-safe parsing.
//! It performs two-pass parsing:
//! 1. First pass: Collect variable definitions
//! 2. Second pass: Parse bindings with variable substitution
//!
//! # Security
//! The parser only reads and structures data - it never executes commands
//! or modifies files. All validation happens in validator.rs after parsing.

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{char, space0},
    combinator::{map, opt},
    sequence::preceded, IResult, Parser
};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

use crate::core::types::{BindType, Keybinding, KeyCombo, Modifier};

/// Parse errors with line number context
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Parse error on line {line}: {message}")]
    InvalidSyntax { line: usize, message: String },

    #[error("Undefined variable '${variable}' on line {line}")]
    UndefinedVariable { variable: String, line: usize },

    #[error("IO error reading config: {0}")]
    IoError(#[from] std::io::Error),
}

/// Parse a complete Hyprland config file
///
/// # Arguments
/// * `content` - The full config file content as a string
/// * `file_path` - Path to the config file (for error messages)
///
/// # Returns
/// A vector of successfully parsed keybindings, or a ParseError
///
/// # Example
/// ```ignore
/// let config = std::fs::read_to_string("hyprland.conf")?;
/// let bindings = parse_config_file(&config, Path::new("hyprland.conf"))?;
/// ```
pub fn parse_config_file(content: &str, _file_path: &Path) -> Result<Vec<Keybinding>, ParseError> {
    // First pass: Collect variable definitions
    let variables = collect_variables(content);

    // Second pass: Parse bindings with variable substitution
    let mut keybindings = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1;  // Human-readable numbers start at 1

        // Skip empty lines and comments
        let line_trimmed = line.trim();
        if line_trimmed.is_empty() || line_trimmed.starts_with('#') {
            continue;
        }

        // Only process bind lines
        if !line_trimmed.starts_with("bind") {
            continue;
        }

        // Substitute variables before parsing
        let substituted = substitute_variables(line_trimmed, &variables);

        // Parse the bind line
        match parse_bind_line(&substituted) {
            Ok((_, binding)) => keybindings.push(binding),
            Err(e) => {
                return Err(ParseError::InvalidSyntax {
                    line: line_num,
                    message: format!("{:?}", e)
                });
            }
        }
    }

    Ok(keybindings)
}

/// Collect variable definitions from config
///
/// Hyprland configs use variables like:
/// ```hyprland
/// $mainMod = SUPER
/// $terminal = kitty
/// ```
///
/// Returns a HashMap mapping variable names to their values
fn collect_variables(contents: &str) -> HashMap<String, String> {
    let mut variables = HashMap::new();

    for line in contents.lines() {
        let line_trimmed = line.trim();

        // Variable definition format: $name = value
        if line_trimmed.starts_with('$') {
            if let Some(equals_pos) = line_trimmed.find('=') {
                let var_name = line_trimmed[1..equals_pos].trim().to_string();
                let var_value = line_trimmed[equals_pos + 1..].trim().to_string();
                variables.insert(var_name, var_value);
            }
        }
    }

    variables
}

/// Substitute variables in a line
///
/// Replaces $varName with its value from the variables HashMap
fn substitute_variables(line: &str, variables: &HashMap<String, String>) -> String {
    let mut result = line.to_string();

    for (var_name, var_value) in variables {
        let pattern = format!("${}", var_name);
        result = result.replace(&pattern, var_value);
    }

    result
}

/// Parse a single bind line
///
/// Format: bind = MODIFIERS, KEY, DISPATCHER, ARGS
/// Example: bind = SUPER, K, exec, firefox
///
/// Returns a Keybinding struct or nom error
fn parse_bind_line(input: &str) -> IResult<&str, Keybinding> {
    // Parse: <bind_type> = <key_combo>, <dispatcher>, <args>
    let (input, bind_type) = parse_bind_type(input)?;
    let (input, _) = (space0, char('='), space0).parse(input)?;
    let (input, key_combo) = parse_key_combo(input)?;
    let (input, _) = (space0, char(','), space0).parse(input)?;
    let (input, (dispatcher, args)) = parse_dispatcher(input)?;

    Ok((
        input,
        Keybinding {
            key_combo,
            bind_type,
            dispatcher,
            args,
        },
    ))
}

/// Parse bind_type (bind, binde, bindl, bindm, bindr, bindel)
fn parse_bind_type(input: &str) -> IResult<&str, BindType> {
    map(
        alt((
            tag("bindel"),  // Must come before "binde" due to being a longer match
            tag("binde"),
            tag("bindl"),
            tag("bindm"),
            tag("bindr"),
            tag("bind"),
            )),
            |s: &str| match s {
                "bind" => BindType::Bind,
                "binde" => BindType::BindE,
                "bindl" => BindType::BindL,
                "bindm" => BindType::BindM,
                "bindr" => BindType::BindR,
                "bindel" => BindType::BindEL,
                _ => unreachable!(),
        },
    ).parse(input)
}

/// Parse key combination
///
/// Formats supported:
/// - "SUPER, K" (single modifier)
/// - "SUPER_SHIFT, K" (underscore-separated modifiers)
/// - "SUPER SHIFT, K" (space-separated modifiers)
/// - ", K" (no modifiers)
fn parse_key_combo(input: &str) -> IResult<&str, KeyCombo> {
    let (input, modifier_str) = take_until(",")(input)?;
    let modifier_str = modifier_str.trim();

    // Parse modifiers
    let modifiers = if modifier_str.is_empty() {
        Vec::new()
    } else {
        parse_modifiers(modifier_str)?
    };

    // Skip the comma
    let (input, _) = (space0, char(','), space0).parse(input)?;

    // Parse key name
    let (input, key) = take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)?;

    Ok((
        input,
        KeyCombo {
            modifiers,
            key: key.to_string(),
        },
    ))
}

/// Parse modifiers from a string
///
/// Handles formats:
/// - "SUPER" → [Super]
/// - "SUPER_SHIFT" → [Super, Shift]
/// - "SUPER SHIFT" → [Super, Shift]
fn parse_modifiers(input: &str) -> Result<Vec<Modifier>, nom::Err<nom::error::Error<&str>>> {
    let mut modifiers = Vec::new();

    // Split by underscore or space
    let parts: Vec<&str> = if input.contains('_') {
        input.split('_').collect()
    } else {
        input.split_whitespace().collect()
    };

    for part in parts {
        let modifier = match part.trim().to_uppercase().as_str() {
            "SUPER" | "MOD4" | "WIN" => Modifier::Super,
            "CTRL" | "CONTROL" => Modifier::Ctrl,
            "SHIFT" => Modifier::Shift,
            "ALT" | "MOD1" => Modifier::Alt,
            _ => continue,
        };
        modifiers.push(modifier);
    }

    Ok(modifiers)
}

/// Parse dispatcher and arguments
///
/// Format: DISPATCHER, ARGS (args are optional)
/// Examples:
/// - "exec, firefox" → ("exec", Some("firefox"))
/// - "killactive" → ("killactive", None)
fn parse_dispatcher(input: &str) -> IResult<&str, (String, Option<String>)> {
    let (input, dispatcher) = take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)?;

    // Check if there are arguments (after comma)
    let (input, args) = opt(preceded(
        (space0, char(','), space0),
        take_while1(|c: char| c != '\n'),
    )).parse(input)?;

    let args_trimmed = args.map(|s: &str| s.trim().to_string());

    Ok((input, (dispatcher.to_string(), args_trimmed)))
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
