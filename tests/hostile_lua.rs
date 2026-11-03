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

//! Lua files read through the app, which evaluates them in a child process
//!
//! The evaluator is the app binary started with a hidden command, so tests
//! that load a Lua config live here, where cargo builds that binary. Most
//! files below used to freeze the app. Every run is bounded by `BOUND`: one
//! still going is stopped and fails the test, so a regression never hangs
//! `cargo test`.

#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "clippy.toml allows these in #[test] functions only, not in the helpers here"
)]

use hypr_keybind_manager::{
    config::{ConfigManager, load_bindings_from},
    core::lua_child,
    ui::{Controller, controller::ImportMode},
};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use tempfile::TempDir;

const APP: &str = env!("CARGO_BIN_EXE_hypr-keybind-manager");
const BOUND: Duration = Duration::from_secs(15);

/// A pattern that backtracks inside C, where no instruction hook runs
const BACKTRACKING: &str =
    r#"string.find(string.rep("a", 40), string.rep("a?", 40) .. string.rep("a", 40) .. "b")"#;

fn write(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).unwrap();
    path
}

/// Makes the library start the app binary as its evaluator; this test's own
/// executable is the test harness
fn use_app_binary() {
    lua_child::set_program(APP.into());
}

/// Runs `hypr-keybind-manager list` on `config` with `extra` arguments and
/// variables, and returns whether it succeeded, its output and its errors
fn list(config: &Path, extra: &[&str], env: &[(&str, &str)]) -> (bool, String, String) {
    let dir = TempDir::new().unwrap();
    let (out, err) = (dir.path().join("out"), dir.path().join("err"));
    let mut child = Command::new(APP)
        .arg("list")
        .arg("--config")
        .arg(config)
        .args(extra)
        .envs(env.iter().copied())
        .stdin(Stdio::null())
        .stdout(File::create(&out).unwrap())
        .stderr(File::create(&err).unwrap())
        .spawn()
        .unwrap();
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if start.elapsed() > BOUND {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("still running after {BOUND:?}");
        }
        thread::sleep(Duration::from_millis(20));
    };
    let read = |path| fs::read_to_string(path).unwrap();
    (status.success(), read(&out), read(&err))
}

/// Runs `list` on a file holding `source` and returns its errors
fn list_fails(source: &str) -> String {
    let dir = TempDir::new().unwrap();
    list_fails_at(&write(&dir, "hostile.lua", source))
}

fn list_fails_at(config: &Path) -> String {
    let (ok, _, errors) = list(config, &[], &[]);
    assert!(!ok, "{errors}");
    errors
}

/// Runs `work` on its own thread and fails the test when it outlasts `BOUND`
fn bounded<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || tx.send(work()));
    rx.recv_timeout(BOUND)
        .unwrap_or_else(|_| panic!("still running after {BOUND:?}"))
}

/// Imports `import` into a controller for `config` and returns the result
fn import(config: PathBuf, import: PathBuf) -> Result<(), String> {
    use_app_binary();
    bounded(move || {
        let controller = Controller::new(config).map_err(|e| e.to_string())?;
        controller.load_keybindings().map_err(|e| e.to_string())?;
        controller.import_from(&import, ImportMode::Merge)
    })
}

#[test]
fn test_finaliser_that_never_returns() {
    // Lua runs finalisers with hooks off, so the instruction limit never fires
    let errors = list_fails("setmetatable({}, { __gc = function() while true do end end })\n");
    assert!(errors.contains("took longer than"), "{errors}");
}

#[test]
fn test_pattern_that_backtracks() {
    let errors = list_fails(BACKTRACKING);
    assert!(errors.contains("took longer than"), "{errors}");
}

#[test]
fn test_endless_loop_after_memory_runs_out() {
    // Fills memory in ever smaller pieces until not even a small table fits.
    // The hook then needs an allocation before it can run, and the "not
    // enough memory" error raised instead is caught like any other.
    let errors = list_fails(concat!(
        "local hog, n = nil, 1 << 20\n",
        "local function add() hog = { hog, string.rep(\"x\", n) } end\n",
        "local function spin() while true do end end\n",
        "pcall(function()\n",
        "    while n > 0 do\n",
        "        if not pcall(add) then n = n // 2 end\n",
        "    end\n",
        "end)\n",
        "while true do pcall(spin) end\n",
    ));
    assert!(errors.contains("took longer than"), "{errors}");
}

#[test]
fn test_file_that_never_ends() {
    // Reading it in the app allocated until the machine ran out of memory
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("hostile.lua");
    std::os::unix::fs::symlink("/dev/zero", &path).unwrap();
    let errors = list_fails_at(&path);
    assert!(errors.contains("memory"), "{errors}");
}

#[test]
fn test_evaluator_sees_home_but_no_other_variables() {
    let dir = TempDir::new().unwrap();
    let config = write(
        &dir,
        "hyprland.lua",
        "hl.bind(\"SUPER + E\", hl.dsp.exec_cmd(os.getenv(\"HOME\") .. \" \" .. tostring(os.getenv(\"HKM_TEST_SECRET\"))))\n",
    );
    let env = [("HOME", "/home/tester"), ("HKM_TEST_SECRET", "hunter2")];
    let (ok, out, errors) = list(&config, &["--json"], &env);
    assert!(ok, "{errors}");
    assert!(out.contains("\"/home/tester nil\""), "{out}");
}

#[test]
fn test_evaluator_returns_binds_and_required_files() {
    use_app_binary();
    let dir = TempDir::new().unwrap();
    write(&dir, "gaming.lua", "return { key = \"G\" }\n");
    let config = write(
        &dir,
        "hyprland.lua",
        "local gaming = require(\"gaming\")\nhl.bind(\"SUPER + \" .. gaming.key, hl.dsp.window.close())\n",
    );

    let loaded = load_bindings_from(&config).unwrap();

    assert_eq!(loaded.bindings.len(), 1);
    assert_eq!(loaded.bindings[0].key_combo.key, "G");
    assert_eq!(loaded.bindings[0].dispatcher, "window.close");
    assert_eq!(loaded.files.len(), 2);
}

#[test]
fn test_import_of_hostile_lua_changes_nothing() {
    let dir = TempDir::new().unwrap();
    let config = write(&dir, "hyprland.conf", "bind = SUPER, Q, exec, kitty\n");
    let before = fs::read_to_string(&config).unwrap();

    let err = import(config.clone(), write(&dir, "shared.lua", BACKTRACKING)).unwrap_err();

    assert!(err.contains("took longer than"), "{err}");
    assert_eq!(fs::read_to_string(&config).unwrap(), before);
}

#[test]
fn test_imported_arguments_cannot_backtrack() {
    // A hyprlang file imported into a Lua config has its arguments evaluated
    // in the app, where the string methods are out of reach
    let dir = TempDir::new().unwrap();
    let config = write(
        &dir,
        "hyprland.lua",
        "hl.bind(\"SUPER + Q\", hl.dsp.exec_cmd(\"kitty\"))\n",
    );
    let before = fs::read_to_string(&config).unwrap();
    let shared = write(
        &dir,
        "shared.conf",
        r#"bind = SUPER, M, movewindow, ("a"):rep(40):find(("a?"):rep(40) .. ("a"):rep(40) .. "b")"#,
    );

    let err = import(config.clone(), shared).unwrap_err();

    assert!(err.contains("string value"), "{err}");
    assert_eq!(fs::read_to_string(&config).unwrap(), before);
}

#[test]
fn test_import_from_lua_cannot_add_config_lines() {
    let dir = TempDir::new().unwrap();
    let config = write(&dir, "hyprland.conf", "bind = SUPER, Q, exec, kitty\n");
    let before = fs::read_to_string(&config).unwrap();
    // A Lua string may hold a line break; in a .conf it would start a new directive
    let shared = write(
        &dir,
        "shared.lua",
        "hl.bind(\"SUPER + F12\", hl.dsp.exec_cmd(\"true\\nexec-once = touch /tmp/pwned\"))\n",
    );

    assert!(import(config.clone(), shared).is_err());
    assert_eq!(fs::read_to_string(&config).unwrap(), before);
}

#[test]
fn test_lua_export_keeps_unrenderable_binds_inside_comments() {
    use_app_binary();
    let dir = TempDir::new().unwrap();
    let config = write(
        &dir,
        "hyprland.lua",
        "hl.bind(\"SUPER + K\", function() end, { description = \"a\\nos.exit()\" })\n",
    );
    let manager = ConfigManager::new(config.clone()).unwrap();
    let bindings = load_bindings_from(&config).unwrap().bindings;
    let export_path = dir.path().join("export.lua");

    manager.export_to(&export_path, &bindings).unwrap();

    let exported = fs::read_to_string(&export_path).unwrap();
    assert!(
        exported
            .lines()
            .all(|line| line.is_empty() || line.starts_with("-- ")),
        "{exported}"
    );
}
