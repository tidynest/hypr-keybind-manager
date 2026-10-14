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

/// Starts the sandbox: no network, read-only system dirs, no home directory
const BWRAP_PREFIX: &str = concat!(
    "bwrap --die-with-parent --new-session --unshare-net ",
    "--ro-bind /usr /usr --ro-bind /bin /bin --ro-bind-try /lib /lib --ro-bind-try /lib64 /lib64 ",
    "--proc /proc --dev /dev --tmpfs /tmp --",
);

/// The prefix written before 1.4.2. It left out the loader in /lib64, so
/// nothing could start, and it handed the command to bwrap directly, so in
/// `a; b` the shell Hyprland starts ran `b` outside the sandbox.
const LEGACY_PREFIX: &str = concat!(
    "bwrap --die-with-parent --new-session --unshare-net ",
    "--ro-bind /usr /usr --ro-bind /bin /bin --proc /proc --dev /dev --tmpfs /tmp --",
);

/// Hyprland runs exec text with `sh -c`, so the whole command line goes,
/// quoted as one word, to a second shell that starts inside the sandbox
pub fn wrap_command(command_line: &str) -> Result<String, String> {
    let trimmed = command_line.trim();
    if trimmed.is_empty() {
        return Err("Sandboxed exec bindings need a command to run".to_string());
    }

    Ok(format!(
        "{BWRAP_PREFIX} sh -c '{}'",
        trimmed.replace('\'', r"'\''")
    ))
}

/// The command inside a wrapper written by `wrap_command`, or by an older version
pub fn unwrap_command(command_line: &str) -> Option<String> {
    let line = command_line.trim();
    if let Some(rest) = after_prefix(line, BWRAP_PREFIX) {
        let quoted = rest.strip_prefix("sh -c '")?.strip_suffix('\'')?;
        let escaped_quote = r"'\''";
        return (!quoted.replace(escaped_quote, "").contains('\''))
            .then(|| quoted.replace(escaped_quote, "'"));
    }
    // Saving the binding again wraps it in the current form
    after_prefix(line, LEGACY_PREFIX).map(str::to_string)
}

pub fn is_wrapped(command_line: &str) -> bool {
    unwrap_command(command_line).is_some()
}

/// The non-empty text after `prefix` and the space that follows it
fn after_prefix<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    let rest = line.strip_prefix(prefix)?.strip_prefix(' ')?.trim_start();
    (!rest.is_empty()).then_some(rest)
}
