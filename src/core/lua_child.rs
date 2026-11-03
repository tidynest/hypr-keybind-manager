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

//! Runs Lua configs in a child process with time and memory limits
//!
//! The sandbox in `lua_config` stops endless loops, but some code runs where
//! its hook cannot reach: a `__gc` finaliser, a string pattern that
//! backtracks inside C, a loop that carries on after memory ran out. An
//! imported file may come from anyone, so every Lua file is run by a second
//! copy of this program, and the app stops it when it takes too long.
//!
//! The child is `/proc/self/exe evaluate-lua-config -- <file>`, started
//! without a shell, with stdin closed and only the variables in
//! `PASSED_ENV`. Before any Lua runs it caps its own address space, CPU time
//! and core dumps. It writes one JSON value to stdout, `{"Ok": LuaConfig}`
//! or `{"Err": "message"}`, of which the app reads at most `MAX_REPLY` bytes.

use nix::{
    libc::SIGXCPU,
    sys::resource::{Resource, getrlimit, setrlimit},
};
use std::{
    env,
    io::{self, Read, Write},
    os::unix::process::ExitStatusExt,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    sync::{OnceLock, mpsc},
    thread,
    time::Duration,
};

use crate::core::lua_config::{LuaConfig, parse_lua_config};

/// The hidden subcommand that makes this program evaluate one file
pub const CHILD_COMMAND: &str = "evaluate-lua-config";

/// Wall-clock time a file gets. A real config takes a few milliseconds and
/// the instruction limit stops a plain endless loop in a tenth of a second.
pub const TIME_LIMIT: Duration = Duration::from_secs(2);

/// CPU seconds the child may use. The app stops it sooner; this ends it
/// when the app is gone.
const CPU_LIMIT: u64 = TIME_LIMIT.as_secs() + 1;

/// The child's address space. The program and its libraries take about
/// 128 MiB of it and Lua stops at 64 MiB; this catches what Lua does not
/// count, such as reading a file that never ends.
const ADDRESS_SPACE_LIMIT: u64 = 512 << 20;

/// Longest reply read from the child. A config with a hundred binds gives
/// about 30 KB.
const MAX_REPLY: usize = 4 << 20;

/// What the child keeps of the environment. Configs build paths from these;
/// anything else, such as a token, stays out of reach of a file from anyone.
const PASSED_ENV: [&str; 10] = [
    "HOME",
    "USER",
    "XDG_CONFIG_HOME",
    "XDG_DATA_HOME",
    "XDG_STATE_HOME",
    "XDG_CACHE_HOME",
    "XDG_RUNTIME_DIR",
    "TERMINAL",
    "EDITOR",
    "BROWSER",
];

static PROGRAM: OnceLock<PathBuf> = OnceLock::new();

/// Runs the child from `program` instead of this executable
///
/// For integration tests, whose own executable is the test harness. Only the
/// first call has an effect.
pub fn set_program(program: PathBuf) {
    let _ = PROGRAM.set(program);
}

/// Runs the Lua config at `path` in a child process and returns its binds
///
/// Blocks for at most `TIME_LIMIT`.
pub fn evaluate(path: &Path) -> Result<LuaConfig, String> {
    // Not current_exe(): once an upgrade replaces the binary, its path names
    // the new version or nothing, while /proc/self/exe is still this one
    let program = PROGRAM
        .get()
        .map_or(Path::new("/proc/self/exe"), PathBuf::as_path);
    let mut child = Command::new(program)
        .arg(CHILD_COMMAND)
        .arg("--")
        .arg(path)
        .env_clear()
        .envs(
            PASSED_ENV
                .iter()
                .filter_map(|name| Some((name, env::var_os(name)?))),
        )
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("cannot start the Lua evaluator: {e}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or("the Lua evaluator has no output")?;
    let (tx, rx) = mpsc::channel();
    // Reads until the child exits; killing it below ends this thread too
    thread::spawn(move || {
        let mut reply = Vec::new();
        let read = stdout.take(MAX_REPLY as u64 + 1).read_to_end(&mut reply);
        let _ = tx.send(read.map(|_| reply));
    });
    let reply = rx.recv_timeout(TIME_LIMIT);
    // Stops a child that is still running and does nothing to one that exited
    let _ = child.kill();
    let status = child
        .wait()
        .map_err(|e| format!("lost the Lua evaluator: {e}"))?;

    let reply = reply
        .map_err(|_| too_long())?
        .map_err(|e| format!("cannot read the Lua evaluator's result: {e}"))?;
    if reply.len() > MAX_REPLY {
        return Err(format!(
            "the binds read from the Lua file take more than {} MiB",
            MAX_REPLY >> 20
        ));
    }
    serde_json::from_slice(&reply).unwrap_or_else(|_| Err(stopped(status)))
}

/// The child's side: caps itself, runs the file and writes the result
///
/// Returns the exit code, which is non-zero only when the result could not
/// be written. A file that fails is a result too.
pub fn serve(path: &Path) -> i32 {
    let result = limit_self().and_then(|()| parse_lua_config(path));
    let mut stdout = io::stdout().lock();
    let written = serde_json::to_writer(&mut stdout, &result).is_ok() && stdout.flush().is_ok();
    i32::from(!written)
}

/// Lowers this process's limits; raising one needs privileges, so a limit
/// already below ours stays
fn limit_self() -> Result<(), String> {
    for (resource, soft, hard) in [
        (
            Resource::RLIMIT_AS,
            ADDRESS_SPACE_LIMIT,
            ADDRESS_SPACE_LIMIT,
        ),
        // SIGXCPU at the soft limit, SIGKILL at the hard one
        (Resource::RLIMIT_CPU, CPU_LIMIT, CPU_LIMIT + 1),
        (Resource::RLIMIT_CORE, 0, 0),
    ] {
        let cannot = |e| format!("cannot limit the Lua evaluator: {e}");
        let (_, current) = getrlimit(resource).map_err(cannot)?;
        setrlimit(resource, soft.min(current), hard.min(current)).map_err(cannot)?;
    }
    Ok(())
}

fn too_long() -> String {
    format!(
        "the Lua file took longer than {} seconds to evaluate and was stopped; \
         it may contain an endless loop",
        TIME_LIMIT.as_secs()
    )
}

/// Why a child ended without a readable result
fn stopped(status: ExitStatus) -> String {
    if status.signal() == Some(SIGXCPU) {
        return too_long();
    }
    format!(
        "the Lua evaluator stopped without a result ({status}); the file may need more \
         memory than it is allowed"
    )
}
