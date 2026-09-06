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

//! Hyprland Lua config support (Hyprland 0.55 and later)
//!
//! A Lua config is a program, so it is run rather than parsed. The file is
//! executed in a sandboxed Lua 5.5 state with a recording stand-in for the
//! `hl` API (`lua_prelude.lua`). Every `hl.bind` call is captured together
//! with the file and line it came from.
//!
//! # Editing
//! A bind can be rewritten when its line is a single `hl.bind(...)` statement
//! with a dispatcher action and only that bind came from the line. Such a
//! line is replaced whole, with literal values. Binds created in loops,
//! binds whose action is a Lua function, and binds using options this module
//! cannot write back are reported read-only with a reason. New binds are
//! appended to the end of the main file.
//!
//! # Security
//! The config runs with no `io`, no `load`, no `os.execute`, `require`
//! limited to files inside the config directory, a memory limit and an
//! instruction limit. Values written back are rendered as Lua string
//! literals or validated as literals, so text from the edit dialog cannot
//! inject code.

use mlua::{HookTriggers, Lua, Table, Value};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::core::types::{BindType, KeyCombo, Keybinding, Modifier};

const PRELUDE: &str = include_str!("lua_prelude.lua");
const MEMORY_LIMIT: usize = 64 << 20;
const INSTRUCTION_LIMIT: u32 = 20_000_000;

/// Header under which added binds are appended to the main file
pub const ADDED_HEADER: &str = "-- Keybindings added by hypr-keybind-manager";

/// Lua bind option names and the hyprlang flag letter each maps to
const OPTION_FLAGS: [(&str, char); 10] = [
    ("repeating", 'e'),
    ("locked", 'l'),
    ("release", 'r'),
    ("click", 'c'),
    ("non_consuming", 'n'),
    ("mouse", 'm'),
    ("transparent", 't'),
    ("ignore_mods", 'i'),
    ("long_press", 'o'),
    ("dont_inhibit", 'p'),
];

/// Dispatchers offered by the edit dialog for Lua configs, `hl.dsp.` paths
/// plus `exec` and `execr` which stand for `exec_cmd` and `exec_raw`.
pub const LUA_DISPATCHERS: &[&str] = &[
    "exec",
    "execr",
    "window.close",
    "window.kill",
    "window.float",
    "window.fullscreen",
    "window.fullscreen_state",
    "window.pseudo",
    "window.pin",
    "window.center",
    "window.move",
    "window.resize",
    "window.drag",
    "window.swap",
    "window.cycle_next",
    "window.bring_to_top",
    "window.alter_zorder",
    "window.tag",
    "window.clear_tags",
    "window.set_prop",
    "window.signal",
    "window.toggle_swallow",
    "window.deny_from_group",
    "focus",
    "layout",
    "submap",
    "workspace.toggle_special",
    "workspace.move",
    "workspace.rename",
    "workspace.change_id",
    "workspace.swap_monitors",
    "group.toggle",
    "group.lock",
    "group.lock_active",
    "group.active",
    "group.next",
    "group.prev",
    "group.move_window",
    "cursor.move",
    "cursor.move_to_corner",
    "pass",
    "send_shortcut",
    "send_key_state",
    "global",
    "dpms",
    "event",
    "exit",
    "force_idle",
    "force_renderer_reload",
    "release_input_capture",
    "no_op",
];

/// One recorded `hl.bind` call
#[derive(Clone, Debug)]
pub struct LuaBinding {
    pub binding: Keybinding,
    /// The keys string as Hyprland saw it, e.g. `SUPER + Q`
    pub keys: String,
    pub file: PathBuf,
    pub line: usize,
    /// Why the bind cannot be rewritten, `None` when it can
    pub read_only: Option<String>,
}

/// Everything recorded while running a Lua config
#[derive(Debug, Default)]
pub struct LuaConfig {
    pub bindings: Vec<LuaBinding>,
    /// The main file first, then every required module in load order
    pub files: Vec<PathBuf>,
}

/// Runs the Lua config at `path` and collects its binds
pub fn parse_lua_config(path: &Path) -> Result<LuaConfig, String> {
    let source =
        fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let dir = path
        .parent()
        .unwrap_or(Path::new("."))
        .canonicalize()
        .map_err(|e| format!("cannot resolve config directory: {e}"))?;

    let lua = Lua::new();
    lua.set_memory_limit(MEMORY_LIMIT)
        .map_err(|e| e.to_string())?;
    lua.set_hook(
        HookTriggers {
            every_nth_instruction: Some(INSTRUCTION_LIMIT),
            ..HookTriggers::default()
        },
        |_, _| {
            Err(mlua::Error::runtime(
                "the config executed too many instructions; is there an endless loop?",
            ))
        },
    )
    .map_err(|e| e.to_string())?;

    let read_module = {
        let dir = dir.clone();
        lua.create_function(move |_, name: String| {
            Ok(match read_module(&dir, &name) {
                Some((source, path)) => (Some(source), Some(path)),
                None => (None, None),
            })
        })
        .map_err(|e| e.to_string())?
    };
    // Level 0 is this function, 1 is hl.bind in the prelude, 2 is the config code
    let locate = lua
        .create_function(|lua, ()| {
            Ok(lua
                .inspect_stack(2, |debug| {
                    let source = debug.source();
                    let file = source
                        .source
                        .as_deref()
                        .map(|s| s.strip_prefix('@').unwrap_or(s).to_string())
                        .unwrap_or_default();
                    (file, debug.curr_line().max(0) as usize)
                })
                .unwrap_or_default())
        })
        .map_err(|e| e.to_string())?;
    let prelude: Table = lua
        .load(PRELUDE)
        .set_name("=hypr-keybind-manager prelude")
        .call((read_module, locate))
        .map_err(|e| format!("prelude failed: {e}"))?;
    let env: Table = prelude.get("env").map_err(|e| e.to_string())?;

    lua.load(&source)
        .set_name(format!("@{}", path.display()))
        .set_environment(env)
        .exec()
        .map_err(|e| format!("Lua config error: {e}"))?;

    let mut files = vec![path.to_path_buf()];
    let required: Table = prelude.get("files").map_err(|e| e.to_string())?;
    for file in required.sequence_values::<String>() {
        files.push(PathBuf::from(file.map_err(|e| e.to_string())?));
    }

    let records: Table = prelude.get("records").map_err(|e| e.to_string())?;
    let mut raw = Vec::new();
    for record in records.sequence_values::<Table>() {
        let record = record.map_err(|e| e.to_string())?;
        if record
            .get::<Option<bool>>("removed")
            .map_err(|e| e.to_string())?
            .unwrap_or(false)
        {
            continue;
        }
        raw.push(convert_record(&record).map_err(|e| e.to_string())?);
    }

    Ok(LuaConfig {
        bindings: mark_read_only(raw, &source, path),
        files,
    })
}

/// Reads `name` as a module inside `dir`; `None` when it is not there or outside
fn read_module(dir: &Path, name: &str) -> Option<(String, String)> {
    let relative = name.replace('.', "/");
    [format!("{relative}.lua"), format!("{relative}/init.lua")]
        .into_iter()
        .filter_map(|candidate| dir.join(candidate).canonicalize().ok())
        .find(|path| path.starts_with(dir) && path.is_file())
        .and_then(|path| {
            let source = fs::read_to_string(&path).ok()?;
            Some((source, path.to_string_lossy().into_owned()))
        })
}

/// A recorded bind before read-only checks
struct Recorded {
    binding: Keybinding,
    keys: String,
    file: PathBuf,
    line: usize,
    kind: String,
    unsupported_options: Vec<String>,
}

fn convert_record(record: &Table) -> mlua::Result<Recorded> {
    let keys: String = record.get("keys")?;
    let kind: String = record.get("kind")?;
    let path: Option<String> = record.get("path")?;
    let args: Option<String> = record.get("args")?;
    let exec: Option<String> = record.get("exec")?;
    let opts: Table = record.get("opts")?;
    let file: String = record.get("file")?;
    let line: usize = record.get("line")?;
    let submap: Option<String> = record.get("submap")?;

    let mut flags = String::new();
    let mut description = None;
    let mut unsupported_options = Vec::new();
    for pair in opts.pairs::<String, Value>() {
        let (name, value) = pair?;
        match name.as_str() {
            "description" | "desc" => {
                if let Value::String(text) = value {
                    description = Some(text.to_str()?.to_string());
                }
            }
            other => match OPTION_FLAGS.iter().find(|(option, _)| *option == other) {
                Some((_, flag)) => {
                    if matches!(value, Value::Boolean(true)) {
                        flags.push(*flag);
                    }
                }
                None => unsupported_options.push(other.to_string()),
            },
        }
    }

    let (dispatcher, args) = match (kind.as_str(), path.as_deref()) {
        ("dispatcher", Some("exec_cmd")) => ("exec".to_string(), exec.or(args)),
        ("dispatcher", Some("exec_raw")) => ("execr".to_string(), exec.or(args)),
        ("dispatcher", Some(path)) => (path.to_string(), args.filter(|a| !a.is_empty())),
        ("function", _) => ("function".to_string(), None),
        _ => ("lua".to_string(), None),
    };

    Ok(Recorded {
        keys: keys.clone(),
        binding: Keybinding {
            key_combo: parse_keys(&keys),
            bind_type: BindType::from_flags(&flags)
                .unwrap_or_default()
                .with_description(description.is_some()),
            dispatcher,
            args,
            description,
            submap,
        },
        file: PathBuf::from(file),
        line,
        kind,
        unsupported_options,
    })
}

/// Splits `"SUPER + SHIFT + Q"` into modifiers and key
///
/// Anything before the last `+` must be a modifier; otherwise the whole
/// string is kept as the key so it round-trips unchanged.
pub fn parse_keys(keys: &str) -> KeyCombo {
    let parts: Vec<&str> = keys
        .split('+')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();
    let Some((key, modifiers)) = parts.split_last() else {
        return KeyCombo::new(Vec::new(), keys.trim());
    };
    let mut parsed = Vec::with_capacity(modifiers.len());
    for modifier in modifiers {
        let modifier = match modifier.to_uppercase().as_str() {
            "SUPER" | "MOD4" | "WIN" => Modifier::Super,
            "CTRL" | "CONTROL" => Modifier::Ctrl,
            "SHIFT" => Modifier::Shift,
            "ALT" | "MOD1" => Modifier::Alt,
            _ => return KeyCombo::new(Vec::new(), keys.trim()),
        };
        parsed.push(modifier);
    }
    KeyCombo::new(parsed, key)
}

/// Decides which recorded binds can be rewritten
fn mark_read_only(recorded: Vec<Recorded>, main_source: &str, main_path: &Path) -> Vec<LuaBinding> {
    let mut per_line: HashMap<(PathBuf, usize), usize> = HashMap::new();
    for r in &recorded {
        *per_line.entry((r.file.clone(), r.line)).or_default() += 1;
    }
    let mut sources: HashMap<PathBuf, String> = HashMap::new();
    sources.insert(main_path.to_path_buf(), main_source.to_string());

    recorded
        .into_iter()
        .map(|r| {
            let at = format!("{}:{}", r.file.display(), r.line);
            let read_only = if r.kind == "function" {
                Some(format!("runs a Lua function; edit it in {at}"))
            } else if r.kind != "dispatcher" {
                Some(format!("its action is not a dispatcher; edit it in {at}"))
            } else if per_line
                .get(&(r.file.clone(), r.line))
                .copied()
                .unwrap_or(0)
                > 1
            {
                Some(format!("created by a loop at {at}; edit the Lua code"))
            } else if !r.unsupported_options.is_empty() {
                Some(format!(
                    "uses the {} option, which the editor cannot write back; edit it in {at}",
                    r.unsupported_options.join(", ")
                ))
            } else {
                let source = sources
                    .entry(r.file.clone())
                    .or_insert_with(|| fs::read_to_string(&r.file).unwrap_or_default());
                let line = source
                    .lines()
                    .nth(r.line.saturating_sub(1))
                    .unwrap_or("")
                    .trim();
                (!(line.starts_with("hl.bind(") && line.ends_with(')')))
                    .then(|| format!("is part of a larger statement at {at}; edit the Lua code"))
            };
            LuaBinding {
                binding: r.binding,
                keys: r.keys,
                file: r.file,
                line: r.line,
                read_only,
            }
        })
        .collect()
}

/// Renders a binding as an `hl.bind(...)` line
pub fn render_lua_bind(binding: &Keybinding) -> Result<String, String> {
    render_lua_bind_with_keys(binding, &lua_string(&keys_string(binding)))
}

/// `SUPER + SHIFT + Q` for a binding
fn keys_string(binding: &Keybinding) -> String {
    let mut keys: Vec<String> = binding
        .key_combo
        .modifiers
        .iter()
        .map(|m| m.to_string())
        .collect();
    keys.push(binding.key_combo.key.clone());
    keys.join(" + ")
}

/// Renders a replacement for `original`, keeping a `mainMod .. " + Q"` style key expression
///
/// The recorded keys value and the literal suffix on the line give the
/// variable's value, so the same variable is reused when the new keys start
/// with it. Otherwise the keys are written as a plain string.
fn render_lua_bind_like(
    binding: &Keybinding,
    original: &str,
    old_keys: &str,
) -> Result<String, String> {
    let new_keys = keys_string(binding);
    let keys = variable_key_prefix(original, old_keys)
        .and_then(|(name, value)| {
            let rest = new_keys.strip_prefix(&value)?;
            Some(format!("{name} .. {}", lua_string(rest)))
        })
        .unwrap_or_else(|| lua_string(&new_keys));
    render_lua_bind_with_keys(binding, &keys)
}

/// `(mainMod, "SUPER")` when `line` starts with `hl.bind(mainMod .. "...", ` and
/// the recorded keys value ends with that literal
fn variable_key_prefix(line: &str, old_keys: &str) -> Option<(String, String)> {
    let rest = line.trim_start().strip_prefix("hl.bind(")?.trim_start();
    let name_len = rest
        .char_indices()
        .take_while(|(i, c)| c.is_alphanumeric() || *c == '_' || (*i > 0 && c.is_ascii_digit()))
        .count();
    let (name, rest) = rest.split_at(name_len);
    if name.is_empty() || name.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    let rest = rest.trim_start().strip_prefix("..")?.trim_start();
    let literal = rest.strip_prefix('"')?;
    let end = literal.find('"')?;
    let suffix = &literal[..end];
    if suffix.contains('\\') {
        return None;
    }
    let value = old_keys.strip_suffix(suffix)?;
    Some((name.to_string(), value.to_string()))
}

fn render_lua_bind_with_keys(binding: &Keybinding, keys: &str) -> Result<String, String> {
    let args = binding.args.as_deref().unwrap_or("");
    let action = match binding.dispatcher.as_str() {
        "exec" => format!("hl.dsp.exec_cmd({})", lua_string(args)),
        "execr" => format!("hl.dsp.exec_raw({})", lua_string(args)),
        "function" | "lua" => {
            return Err("This binding runs Lua code and cannot be written by the editor".into());
        }
        path => {
            validate_dispatcher_path(path)?;
            validate_lua_args(args)?;
            format!("hl.dsp.{path}({args})")
        }
    };

    let mut opts: Vec<String> = OPTION_FLAGS
        .iter()
        .filter(|(_, flag)| binding.bind_type.has(*flag))
        .map(|(option, _)| format!("{option} = true"))
        .collect();
    if let Some(description) = &binding.description {
        opts.push(format!("description = {}", lua_string(description)));
    }
    let opts = if opts.is_empty() {
        String::new()
    } else {
        format!(", {{ {} }}", opts.join(", "))
    };

    Ok(format!("hl.bind({keys}, {action}{opts})"))
}

fn validate_dispatcher_path(path: &str) -> Result<(), String> {
    let valid = !path.is_empty()
        && path.split('.').all(|segment| {
            !segment.is_empty()
                && segment
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        });
    if valid {
        Ok(())
    } else {
        Err(format!(
            "'{path}' is not a dispatcher name; use a name like window.close or exec"
        ))
    }
}

/// Accepts only text that is a Lua literal (table, string, number, boolean)
fn validate_lua_args(args: &str) -> Result<(), String> {
    if args.trim().is_empty() {
        return Ok(());
    }
    let lua = Lua::new();
    lua.set_memory_limit(1 << 20).map_err(|e| e.to_string())?;
    let empty = lua.create_table().map_err(|e| e.to_string())?;
    let value: Value = lua
        .load(format!("return {args}"))
        .set_name("=arguments")
        .set_environment(empty)
        .eval()
        .map_err(|e| {
            format!("Arguments must be a Lua literal such as {{ direction = \"left\" }}: {e}")
        })?;
    if matches!(value, Value::Function(_)) {
        return Err("Arguments cannot be a function".into());
    }
    Ok(())
}

/// A double-quoted Lua string literal
fn lua_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\{:03}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Works out the new text of every file that has to change
///
/// `bindings` is the wanted list. Each bind recorded in `config` that is
/// missing from it has its line replaced by a new bind in the same submap,
/// or removed. New global binds go to the end of the main file. Returns
/// `(file, new content)` pairs; an empty list means nothing changed.
pub fn rewrite_lua_files(
    config: &LuaConfig,
    bindings: &[Keybinding],
) -> Result<Vec<(PathBuf, String)>, String> {
    let mut removed: Vec<&LuaBinding> = config.bindings.iter().collect();
    let mut added: Vec<Keybinding> = Vec::new();
    for binding in bindings {
        match removed.iter().position(|r| r.binding == *binding) {
            Some(i) => {
                removed.remove(i);
            }
            None => added.push(binding.clone()),
        }
    }
    if removed.is_empty() && added.is_empty() {
        return Ok(Vec::new());
    }
    if let Some(locked) = removed.iter().find(|r| r.read_only.is_some()) {
        return Err(format!(
            "{} → {} cannot be changed: it {}",
            locked.binding.key_combo,
            locked.binding.dispatcher,
            locked.read_only.as_deref().unwrap_or("is read-only")
        ));
    }

    let mut edits: HashMap<&Path, Vec<(usize, Option<Keybinding>)>> = HashMap::new();
    for r in removed {
        let replacement = added
            .iter()
            .position(|a| a.submap == r.binding.submap)
            .map(|i| added.remove(i));
        edits
            .entry(r.file.as_path())
            .or_default()
            .push((r.line, replacement));
    }
    if let Some(inside) = added.iter().find_map(|a| a.submap.as_deref()) {
        return Err(format!(
            "Adding a binding inside submap '{inside}' is not supported for Lua configs. \
             Add it inside hl.define_submap in the file."
        ));
    }

    let main = config.files.first().ok_or("no config file recorded")?;
    let mut outputs = Vec::new();
    for file in &config.files {
        let file_edits = edits.remove(file.as_path()).unwrap_or_default();
        let is_main = file == main;
        if file_edits.is_empty() && !(is_main && !added.is_empty()) {
            continue;
        }

        let original =
            fs::read_to_string(file).map_err(|e| format!("cannot read {}: {e}", file.display()))?;
        let mut lines: Vec<Option<String>> =
            original.lines().map(|l| Some(l.to_string())).collect();
        for (line, replacement) in file_edits {
            let Some(slot) = lines.get_mut(line.saturating_sub(1)) else {
                return Err(format!("{}:{line} no longer exists", file.display()));
            };
            let indent: String = slot
                .as_deref()
                .unwrap_or("")
                .chars()
                .take_while(|c| c.is_whitespace())
                .collect();
            *slot = match replacement {
                Some(binding) => {
                    let original = slot.as_deref().unwrap_or("");
                    let old_keys = config
                        .bindings
                        .iter()
                        .find(|b| b.file == *file && b.line == line)
                        .map(|b| b.keys.as_str())
                        .unwrap_or("");
                    Some(format!(
                        "{indent}{}",
                        render_lua_bind_like(&binding, original, old_keys)?
                    ))
                }
                None => None,
            };
        }

        let mut out: Vec<String> = lines.into_iter().flatten().collect();
        if is_main && !added.is_empty() {
            if !out.iter().any(|l| l.trim() == ADDED_HEADER) {
                out.push(String::new());
                out.push(ADDED_HEADER.to_string());
            }
            for binding in added.drain(..) {
                out.push(render_lua_bind(&binding)?);
            }
        }

        let mut text = out.join("\n");
        if original.ends_with('\n') || original.is_empty() {
            text.push('\n');
        }
        outputs.push((file.clone(), text));
    }

    Ok(outputs)
}
