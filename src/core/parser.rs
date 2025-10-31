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
};
use nom::{
    character::complete::{char, space0},
    combinator::{map, opt},
};
use nom::{sequence::preceded, IResult, Parser};
use std::{collections::HashMap, path::Path};
use thiserror::Error;

use crate::core::types::{BindType, KeyCombo, Keybinding, Modifier};

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
        let line_num = line_num + 1; // Human-readable numbers start at 1

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
                    message: format!("{:?}", e),
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
pub fn collect_variables(contents: &str) -> HashMap<String, String> {
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
pub fn substitute_variables(line: &str, variables: &HashMap<String, String>) -> String {
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
pub fn parse_bind_line(input: &str) -> IResult<&str, Keybinding> {
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
///
/// Recognizes all six Hyprland binding types and converts them to
/// the corresponding BindType enum variant. The order matters: `bindel`
/// must be checked before `binde` to avoid partial matches.
///
/// # Returns
///
/// The parsed BindType variant, or a nom parsing error if the input
/// doesn't start with a valid bind type keyword.
pub fn parse_bind_type(input: &str) -> IResult<&str, BindType> {
    map(
        alt((
            tag("bindel"), // Must come before "binde" due to being a longer match
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
    )
    .parse(input)
}

/// Parse key combination
///
/// Formats supported:
/// - "SUPER, K" (single modifier)
/// - "SUPER_SHIFT, K" (underscore-separated modifiers)
/// - "SUPER SHIFT, K" (space-separated modifiers)
/// - ", K" (no modifiers)
pub fn parse_key_combo(input: &str) -> IResult<&str, KeyCombo> {
    let (input, modifier_str) = take_until(",")(input)?;
    let modifier_str = modifier_str.trim();

    // Parse modifiers
    let modifiers = if modifier_str.is_empty() {
        Vec::new()
    } else {
        parse_modifiers(modifier_str)?
    };

    // Parse key name (skipping the comma)
    let (input, _) = (space0, char(','), space0).parse(input)?;

    let (input, key) = take_until(",")(input)?;
    let key = key.trim().to_string();

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
pub fn parse_modifiers(input: &str) -> Result<Vec<Modifier>, nom::Err<nom::error::Error<&str>>> {
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
pub fn parse_dispatcher(input: &str) -> IResult<&str, (String, Option<String>)> {
    let (input, dispatcher) = take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)?;

    // Check if there are arguments (after comma)
    let (input, args) = opt(preceded(
        (space0, char(','), space0),
        take_while1(|c: char| c != '\n'),
    ))
    .parse(input)?;

    let args_trimmed = args.map(|s: &str| s.trim().to_string());

    Ok((input, (dispatcher.to_string(), args_trimmed)))
}
