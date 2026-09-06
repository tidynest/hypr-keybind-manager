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
//! - Every bind flag combination (bind, bindel, bindd, bindnt, ...)
//! - Variable substitution ($mainMod), defined before use like Hyprland
//! - `submap = name` sections, recorded on each binding
//! - `source = file` lines, followed relative to the including file
//! - Comments, including trailing `# comment` on bind lines
//!
//! # Architecture
//! `scan_lines` classifies every line of one file. `parse_config_tree` runs
//! it over the main file and everything it sources, collecting bindings and
//! the list of files visited. The config writer uses the same scanner so a
//! line is understood the same way when read and when rewritten.
//!
//! # Security
//! The parser only reads and structures data - it never executes commands
//! or modifies files. All validation happens in validator.rs after parsing.

use nom::{IResult, Parser, sequence::preceded};
use nom::{
    bytes::complete::{tag, take_until, take_while, take_while1},
    character::complete::{char, space0},
    combinator::{map_res, opt},
};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

use crate::core::types::{BindType, KeyCombo, Keybinding, Modifier};

/// How deep `source =` chains are followed before giving up
const MAX_SOURCE_DEPTH: usize = 8;

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

/// Everything read from a config file and the files it sources
#[derive(Debug, Default)]
pub struct ParsedConfig {
    /// Bindings in file order, sourced files spliced in where they are sourced
    pub bindings: Vec<Keybinding>,
    /// The main file first, then every sourced file in the order visited
    pub files: Vec<PathBuf>,
    /// Every `$name = value` seen, later definitions win
    pub variables: HashMap<String, String>,
}

/// What one config line is
#[derive(Debug)]
pub enum LineKind {
    /// A bind line, with its submap filled in
    Binding(Keybinding),
    /// `submap = name`, or `submap = reset` as `None`
    Submap(Option<String>),
    /// `source = path`, the path as written
    Source(String),
    /// Comments, blank lines, variables, settings
    Other,
}

/// Parse a complete Hyprland config file, following `source =` lines
///
/// # Arguments
/// * `content` - The full config file content as a string
/// * `file_path` - Path to the config file, used to resolve relative sources
///
/// # Returns
/// A vector of successfully parsed keybindings, or a ParseError
///
/// # Example
/// ```ignore
/// let config = std::fs::read_to_string("hyprland.conf")?;
/// let bindings = parse_config_file(&config, Path::new("hyprland.conf"))?;
/// ```
pub fn parse_config_file(content: &str, file_path: &Path) -> Result<Vec<Keybinding>, ParseError> {
    Ok(parse_config_tree(content, file_path)?.bindings)
}

/// Parse a config file and everything it sources
///
/// Sourced files that cannot be read are skipped with a warning, so one
/// missing include does not stop the whole config from loading. Glob
/// patterns in `source =` lines are not expanded.
pub fn parse_config_tree(content: &str, file_path: &Path) -> Result<ParsedConfig, ParseError> {
    let mut parsed = ParsedConfig {
        files: vec![file_path.to_path_buf()],
        ..ParsedConfig::default()
    };
    parse_into(content, file_path, &mut parsed, 0)?;
    Ok(parsed)
}

fn parse_into(
    content: &str,
    file_path: &Path,
    parsed: &mut ParsedConfig,
    depth: usize,
) -> Result<(), ParseError> {
    for (line, kind) in scan_lines(content, &mut parsed.variables) {
        match kind {
            Ok(LineKind::Binding(binding)) => parsed.bindings.push(binding),
            Ok(LineKind::Source(target)) => {
                let Some(target) = resolve_source(&target, file_path) else {
                    eprintln!(
                        "⚠ Skipping source line {line} of {}: glob patterns are not supported",
                        file_path.display()
                    );
                    continue;
                };
                if parsed.files.contains(&target) || depth >= MAX_SOURCE_DEPTH {
                    continue;
                }
                match fs::read_to_string(&target) {
                    Ok(sourced) => {
                        parsed.files.push(target.clone());
                        parse_into(&sourced, &target, parsed, depth + 1)?;
                    }
                    Err(e) => eprintln!(
                        "⚠ Cannot read sourced file {} (line {line} of {}): {e}",
                        target.display(),
                        file_path.display()
                    ),
                }
            }
            Ok(_) => {}
            Err(ParseError::InvalidSyntax { line, message }) if depth > 0 => {
                return Err(ParseError::InvalidSyntax {
                    line,
                    message: format!("{}: {message}", file_path.display()),
                });
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

/// Resolves a `source =` target against the file that contains it
fn resolve_source(target: &str, from: &Path) -> Option<PathBuf> {
    // ponytail: no glob crate; a pattern is skipped rather than half-expanded
    if target.contains(['*', '?', '[']) {
        return None;
    }
    let expanded = shellexpand::tilde(target.trim()).into_owned();
    let path = PathBuf::from(expanded);
    if path.is_absolute() {
        Some(path)
    } else {
        Some(from.parent().unwrap_or(Path::new(".")).join(path))
    }
}

/// Classifies every line of one file, in order
///
/// Variables defined in the file are added to `variables` as they are met,
/// so a `$mainMod` used below its definition resolves. Returns the
/// 1-based line number with each result.
pub fn scan_lines(
    content: &str,
    variables: &mut HashMap<String, String>,
) -> Vec<(usize, Result<LineKind, ParseError>)> {
    let mut submap: Option<String> = None;

    content
        .lines()
        .enumerate()
        .map(|(index, line)| {
            let line_num = index + 1;
            let stripped = strip_comment(line);
            let trimmed = stripped.trim();

            let kind = if trimmed.is_empty() {
                Ok(LineKind::Other)
            } else if let Some((name, value)) = variable_definition(trimmed) {
                variables.insert(name, value);
                Ok(LineKind::Other)
            } else if let Some(name) = directive_value(trimmed, "submap") {
                submap = (name != "reset").then(|| name.to_string());
                Ok(LineKind::Submap(submap.clone()))
            } else if let Some(target) = directive_value(trimmed, "source") {
                Ok(LineKind::Source(target.to_string()))
            } else if trimmed.starts_with("bind") {
                parse_scanned_bind(trimmed, variables, line_num).map(|mut binding| {
                    binding.submap = submap.clone();
                    LineKind::Binding(binding)
                })
            } else {
                Ok(LineKind::Other)
            };

            (line_num, kind)
        })
        .collect()
}

fn parse_scanned_bind(
    line: &str,
    variables: &HashMap<String, String>,
    line_num: usize,
) -> Result<Keybinding, ParseError> {
    let flags: String = line[4..]
        .chars()
        .take_while(char::is_ascii_alphabetic)
        .collect();
    if let Err(unknown) = BindType::from_flags(&flags) {
        return Err(ParseError::InvalidSyntax {
            line: line_num,
            message: format!("unknown bind flag '{unknown}' in 'bind{flags}'"),
        });
    }

    let substituted = substitute_variables(line, variables);
    parse_bind_line(&substituted)
        .map(|(_, binding)| binding)
        .map_err(|e| ParseError::InvalidSyntax {
            line: line_num,
            message: format!("{:?}", e),
        })
}

/// Removes a trailing `# comment`; `##` is a literal `#` as in Hyprland
fn strip_comment(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '#' {
            if chars.peek() == Some(&'#') {
                chars.next();
                out.push('#');
                continue;
            }
            break;
        }
        out.push(c);
    }
    out
}

/// `$name = value` split into its parts
fn variable_definition(line: &str) -> Option<(String, String)> {
    let rest = line.strip_prefix('$')?;
    let (name, value) = rest.split_once('=')?;
    Some((name.trim().to_string(), value.trim().to_string()))
}

/// The value of a `keyword = value` line, if `line` starts with `keyword`
fn directive_value<'a>(line: &'a str, keyword: &str) -> Option<&'a str> {
    let rest = line.strip_prefix(keyword)?.trim_start();
    Some(rest.strip_prefix('=')?.trim())
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
    contents
        .lines()
        .filter_map(|line| variable_definition(line.trim()))
        .collect()
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
/// Format: bind[flags] = MODIFIERS, KEY, [DESCRIPTION,] DISPATCHER, ARGS
/// Example: bind = SUPER, K, exec, firefox
///
/// The description field is only read when the `d` flag is present.
/// Returns a Keybinding struct or nom error
pub fn parse_bind_line(input: &str) -> IResult<&str, Keybinding> {
    let (input, bind_type) = parse_bind_type(input)?;
    let (input, _) = (space0, char('='), space0).parse(input)?;
    let (input, key_combo) = parse_key_combo(input)?;
    let (input, _) = (space0, char(','), space0).parse(input)?;

    let (input, description) = if bind_type.has_description() {
        let (input, description) = take_until(",")(input)?;
        let (input, _) = (space0, char(','), space0).parse(input)?;
        (input, Some(description.trim().to_string()))
    } else {
        (input, None)
    };

    let (input, (dispatcher, args)) = parse_dispatcher(input)?;

    Ok((
        input,
        Keybinding {
            key_combo,
            bind_type,
            dispatcher,
            args,
            description,
            submap: None,
        },
    ))
}

/// Parse the bind keyword and its flag letters (`bind`, `bindel`, `bindd`, ...)
///
/// # Returns
///
/// The parsed BindType, or a nom error if a flag letter is unknown.
pub fn parse_bind_type(input: &str) -> IResult<&str, BindType> {
    map_res(
        (tag("bind"), take_while(|c: char| c.is_ascii_alphabetic())),
        |(_, flags): (&str, &str)| BindType::from_flags(flags),
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
    let key = key.trim();

    Ok((input, KeyCombo::new(modifiers, key)))
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
