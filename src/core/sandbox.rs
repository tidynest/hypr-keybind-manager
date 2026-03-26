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

//! Bubblewrap sandbox helpers for exec bindings.

const BWRAP_PREFIX: &[&str] = &[
    "bwrap",
    "--die-with-parent",
    "--new-session",
    "--unshare-net",
    "--ro-bind",
    "/usr",
    "/usr",
    "--ro-bind",
    "/bin",
    "/bin",
    "--proc",
    "/proc",
    "--dev",
    "/dev",
    "--tmpfs",
    "/tmp",
    "--",
];

pub fn wrap_command(command_line: &str) -> Result<String, String> {
    let trimmed = command_line.trim();
    if trimmed.is_empty() {
        return Err("Sandboxed exec bindings need a command to run".to_string());
    }

    let mut tokens = BWRAP_PREFIX
        .iter()
        .map(|token| token.to_string())
        .collect::<Vec<_>>();
    tokens.extend(trimmed.split_whitespace().map(str::to_string));

    Ok(tokens.join(" "))
}

pub fn unwrap_command(command_line: &str) -> Option<String> {
    let tokens = command_line.split_whitespace().collect::<Vec<_>>();
    if tokens.len() <= BWRAP_PREFIX.len() || !tokens.starts_with(BWRAP_PREFIX) {
        return None;
    }

    Some(tokens[BWRAP_PREFIX.len()..].join(" "))
}

pub fn is_wrapped(command_line: &str) -> bool {
    unwrap_command(command_line).is_some()
}
