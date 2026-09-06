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

//! Tests for reading and rewriting Lua configs

use crate::core::{
    lua_config::{ADDED_HEADER, parse_lua_config, render_lua_bind, rewrite_lua_files},
    types::{BindType, KeyCombo, Keybinding, Modifier},
};
use std::{fs, path::PathBuf};
use tempfile::TempDir;

const CONFIG: &str = r#"local mainMod = "SUPER"
local terminal = "kitty"
local gaming = require("gaming")

hl.bind(mainMod .. " + Q", hl.dsp.exec_cmd(terminal), { description = "Terminal" })
hl.bind(mainMod .. " + C", hl.dsp.window.close())
hl.bind("XF86AudioMute", hl.dsp.exec_cmd("wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle"), { locked = true, repeating = true })
for i = 1, 3 do
    hl.bind(mainMod .. " + " .. i, hl.dsp.focus({ workspace = i }))
end
hl.bind(mainMod .. " + G", gaming.toggle)
local handle = hl.bind(mainMod .. " + H", hl.dsp.window.pseudo())
hl.bind(mainMod .. " + mouse:272", hl.dsp.window.drag(), { mouse = true })
hl.bind(mainMod .. " + Z", hl.dsp.exec_cmd("gone"))
hl.unbind(mainMod .. " + Z")
hl.bind(mainMod .. " + D", hl.dsp.exec_cmd("dev"), { device = { list = { "kb" } } })
hl.bind(mainMod .. " + R", hl.dsp.submap("resize"))
hl.define_submap("resize", function()
    hl.bind("right", hl.dsp.window.resize({ x = 10, y = 0, relative = true }), { repeating = true })
    hl.bind("escape", hl.dsp.submap("reset"))
end)
hl.on("hyprland.start", function() hl.exec_cmd("never") end)
hl.config({ general = { gaps_in = 2 } })
local layers = #hl.get_layers({ namespace = "anyrun" })
"#;

const GAMING: &str = "local M = {}\nfunction M.toggle() end\nreturn M\n";

fn write_config(dir: &TempDir, main: &str) -> PathBuf {
    let path = dir.path().join("hyprland.lua");
    fs::write(&path, main).unwrap();
    fs::write(dir.path().join("gaming.lua"), GAMING).unwrap();
    path
}

fn binding(mods: Vec<Modifier>, key: &str, dispatcher: &str, args: Option<&str>) -> Keybinding {
    Keybinding {
        key_combo: KeyCombo::new(mods, key),
        bind_type: BindType::Bind,
        dispatcher: dispatcher.to_string(),
        args: args.map(str::to_string),
        description: None,
        submap: None,
    }
}

#[test]
fn test_parse_lua_config_records_every_bind() {
    let dir = TempDir::new().unwrap();
    let path = write_config(&dir, CONFIG);

    let parsed = parse_lua_config(&path).unwrap();
    let keys: Vec<String> = parsed
        .bindings
        .iter()
        .map(|b| b.binding.key_combo.to_string())
        .collect();
    assert_eq!(
        keys,
        [
            "SUPER+Q",
            "SUPER+C",
            "XF86AudioMute",
            "SUPER+1",
            "SUPER+2",
            "SUPER+3",
            "SUPER+G",
            "SUPER+H",
            "SUPER+mouse:272",
            "SUPER+D",
            "SUPER+R",
            "right",
            "escape"
        ],
        "unbound SUPER+Z must be gone"
    );
    assert_eq!(parsed.files.len(), 2, "gaming.lua was required");
    assert!(parsed.files[1].ends_with("gaming.lua"));

    let q = &parsed.bindings[0];
    assert_eq!(q.binding.dispatcher, "exec");
    assert_eq!(q.binding.args.as_deref(), Some("kitty"));
    assert_eq!(q.binding.description.as_deref(), Some("Terminal"));
    assert!(q.binding.bind_type.has_description());
    assert_eq!(q.line, 5);
    assert_eq!(q.override_reason, None);

    let close = &parsed.bindings[1];
    assert_eq!(close.binding.dispatcher, "window.close");
    assert_eq!(close.binding.args, None);

    let mute = &parsed.bindings[2];
    assert_eq!(mute.binding.bind_type.flags(), "el");
    assert_eq!(mute.binding.key_combo.modifiers, Vec::<Modifier>::new());

    let workspace = &parsed.bindings[3];
    assert_eq!(workspace.binding.dispatcher, "focus");
    assert_eq!(workspace.binding.args.as_deref(), Some("{ workspace = 1 }"));
    assert!(
        workspace
            .override_reason
            .as_deref()
            .unwrap()
            .contains("loop")
    );

    assert!(
        parsed.bindings[6]
            .override_reason
            .as_deref()
            .unwrap()
            .contains("Lua function")
    );
    assert!(
        parsed.bindings[7]
            .override_reason
            .as_deref()
            .unwrap()
            .contains("larger statement")
    );
    assert!(parsed.bindings[8].binding.bind_type.has('m'));
    assert!(
        parsed.bindings[9]
            .override_reason
            .as_deref()
            .unwrap()
            .contains("device")
    );

    let resize = &parsed.bindings[11];
    assert_eq!(resize.binding.submap.as_deref(), Some("resize"));
    assert_eq!(resize.binding.dispatcher, "window.resize");
    assert_eq!(
        resize.binding.args.as_deref(),
        Some("{ relative = true, x = 10, y = 0 }")
    );
    assert_eq!(
        parsed.bindings[12].binding.submap.as_deref(),
        Some("resize")
    );
    assert_eq!(parsed.bindings[10].binding.submap, None);
}

#[test]
fn test_lua_sandbox_blocks_escapes() {
    let dir = TempDir::new().unwrap();
    for (source, expect) in [
        ("io.open(\"/etc/passwd\")", "nil"),
        ("os.execute(\"true\")", "nil"),
        ("load(\"return 1\")", "nil"),
        ("require(\"../../etc/passwd\")", "not found"),
        ("while true do end", "instructions"),
    ] {
        let path = write_config(&dir, source);
        let err = parse_lua_config(&path).unwrap_err();
        assert!(err.contains(expect), "{source}: {err}");
    }
}

#[test]
fn test_render_lua_bind_escapes_and_validates() {
    let mut exec = binding(
        vec![Modifier::Shift, Modifier::Super],
        "E",
        "exec",
        Some("notify-send \"hi\""),
    );
    exec.bind_type = BindType::BindL;
    exec.description = Some("Say hi".to_string());
    assert_eq!(
        render_lua_bind(&exec).unwrap(),
        "hl.bind(\"SUPER + SHIFT + E\", hl.dsp.exec_cmd(\"notify-send \\\"hi\\\"\"), { locked = true, description = \"Say hi\" })"
    );

    let mv = binding(
        vec![Modifier::Super],
        "2",
        "window.move",
        Some("{ workspace = 2 }"),
    );
    assert_eq!(
        render_lua_bind(&mv).unwrap(),
        "hl.bind(\"SUPER + 2\", hl.dsp.window.move({ workspace = 2 }))"
    );

    let injected = binding(vec![], "X", "window.move", Some("os.execute(\"rm\")"));
    assert!(render_lua_bind(&injected).is_err());
    let bad_path = binding(vec![], "X", "window.close()); os.exit(", None);
    assert!(render_lua_bind(&bad_path).is_err());
    let function = binding(vec![], "X", "function", None);
    assert!(render_lua_bind(&function).is_err());
}

#[test]
fn test_rewrite_lua_files_edits_lines_in_place() {
    let dir = TempDir::new().unwrap();
    let path = write_config(&dir, CONFIG);
    let parsed = parse_lua_config(&path).unwrap();

    let mut wanted: Vec<Keybinding> = parsed.bindings.iter().map(|b| b.binding.clone()).collect();
    wanted[0].args = Some("alacritty".to_string()); // edit SUPER+Q
    wanted.remove(1); // delete SUPER+C
    wanted.push(binding(vec![Modifier::Super], "T", "exec", Some("thunar")));

    let outputs = rewrite_lua_files(&parsed, &wanted).unwrap();
    assert_eq!(outputs.len(), 1);
    let (file, text) = &outputs[0];
    assert_eq!(file, &path);

    // The edited line is rewritten in place; the new bind takes the deleted line's slot
    let expected = CONFIG.replace(
        "hl.bind(mainMod .. \" + Q\", hl.dsp.exec_cmd(terminal), { description = \"Terminal\" })\nhl.bind(mainMod .. \" + C\", hl.dsp.window.close())\n",
        "hl.bind(mainMod .. \" + Q\", hl.dsp.exec_cmd(\"alacritty\"), { description = \"Terminal\" })\nhl.bind(mainMod .. \" + T\", hl.dsp.exec_cmd(\"thunar\"))\n",
    );
    assert_eq!(text, &expected);

    // With nothing deleted, a new bind is appended under a header
    let mut appended: Vec<Keybinding> = parsed.bindings.iter().map(|b| b.binding.clone()).collect();
    appended.push(binding(vec![Modifier::Super], "Y", "exec", Some("yazi")));
    let (_, text) = rewrite_lua_files(&parsed, &appended).unwrap().remove(0);
    assert_eq!(
        text,
        format!("{CONFIG}\n{ADDED_HEADER}\nhl.bind(\"SUPER + Y\", hl.dsp.exec_cmd(\"yazi\"))\n")
    );

    // Loop-generated and function binds are changed through overrides at the end
    let mut overridden: Vec<Keybinding> =
        parsed.bindings.iter().map(|b| b.binding.clone()).collect();
    overridden[3].args = Some("{ workspace = 9 }".to_string()); // edit SUPER+1 (loop)
    overridden.remove(6); // delete SUPER+G (function)
    let (_, text) = rewrite_lua_files(&parsed, &overridden).unwrap().remove(0);
    assert_eq!(
        text,
        format!(
            "{CONFIG}\n{ADDED_HEADER}\nhl.unbind(\"SUPER + 1\")\nhl.bind(\"SUPER + 1\", hl.dsp.focus({{ workspace = 9 }}))\nhl.unbind(\"SUPER + G\")\n"
        )
    );
    fs::write(&path, &text).unwrap();
    let reparsed = parse_lua_config(&path).unwrap();
    let ones: Vec<&Keybinding> = reparsed
        .bindings
        .iter()
        .map(|b| &b.binding)
        .filter(|b| b.key_combo.to_string() == "SUPER+1")
        .collect();
    assert_eq!(ones.len(), 1, "the override replaces the loop bind");
    assert_eq!(ones[0].args.as_deref(), Some("{ workspace = 9 }"));
    assert!(
        !reparsed
            .bindings
            .iter()
            .any(|b| b.binding.key_combo.to_string() == "SUPER+G"),
        "unbind hides the function bind"
    );
    assert!(
        reparsed
            .bindings
            .iter()
            .find(|b| b.binding.key_combo.to_string() == "SUPER+1")
            .unwrap()
            .override_reason
            .is_none(),
        "the appended override is a plain line, editable in place"
    );
    fs::write(&path, CONFIG).unwrap();

    // New binds inside a submap are refused
    let mut with_submap: Vec<Keybinding> =
        parsed.bindings.iter().map(|b| b.binding.clone()).collect();
    let mut inner = binding(vec![], "left", "window.resize", Some("{ x = -10, y = 0 }"));
    inner.submap = Some("resize".to_string());
    with_submap.push(inner);
    assert!(
        rewrite_lua_files(&parsed, &with_submap)
            .unwrap_err()
            .contains("submap")
    );

    // Nothing changed means nothing to write
    let same: Vec<Keybinding> = parsed.bindings.iter().map(|b| b.binding.clone()).collect();
    assert!(rewrite_lua_files(&parsed, &same).unwrap().is_empty());
}
