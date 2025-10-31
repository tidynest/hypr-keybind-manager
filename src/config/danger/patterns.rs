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

//! Dangerous command pattern definitions
//!
//! This module contains the pattern lists used by DangerDetector
//! to categorise commands by danger level

use std::collections::HashSet;

use regex::Regex;

/// Builds regex patterns for critical system-destroying commands (Round 1)
///
/// These patterns detect immediate, irreversible system destruction:
/// - Pattern 0-1: `rm -rf /` variants (filesystem destruction)
/// - Pattern 2: `dd` to disk devices (partition table destruction)
/// - Pattern 3: Fork bomb (resource exhaustion)
pub fn build_critical_patterns() -> Vec<Regex> {
    vec![
        // Pattern 0a: rm -rf / (r before f)
        Regex::new(r"rm\s+.*[rR].*[fF].*\s+/\s*$")
            .expect("rm -rf / pattern (r before f) should be valid regex"),
        // Pattern 0b: rm -fr / (f before r)
        Regex::new(r"rm\s+.*[fF].*[rR].*\s+/\s*$")
            .expect("rm -fr / pattern (f before r) should be valid regex"),
        // Pattern 1: dd to disk devices
        Regex::new(r"dd\s+.*of=/dev/(sd[a-z]|nvme\d+n\d+)")
            .expect("dd to disk pattern should be valid regex"),
        // Pattern 2: Fork bomb
        Regex::new(r":\(\)\s*\{\s*:\s*\|\s*:\s*&\s*}\s*;\s*:")
            .expect("fork bomb pattern should be valid regex"),
    ]
}

/// Builds HashSet of dangerous commands (Round 2)
///
/// These commands can cause **serious security issues** but aren't
/// instant system destruction. They may have legitimate uses but
/// require careful review.
///
/// # Categories
/// - **File destruction**: shred, srm, wipe
/// - **Permission changes**: chmod, chown (checked in check_dangerous_arguments)
/// - **Privilege escalation**: sudo, doas, su, pkexec
/// - **Disk operations**: mkfs, fdisk, parted, wipefs
/// - **Network exposure**: ufw, firewalld (pattern matched for specific danger)
///
/// # Design Choice
/// Static HashSet for O(1) lookup performance. User cannot disable
/// these checks (security cannot be bypassed).
pub fn build_dangerous_commands() -> HashSet<String> {
    vec![
        // File destruction (not root, but still bad)
        "shred",
        "srm",
        "wipe",
        // Permission and ownership changes
        "chmod",
        "chown",
        // Privilege escalation
        "sudo",
        "doas",
        "su",
        "pkexec",
        // Disk operations
        "mkfs",
        "fdisk",
        "parted",
        "wipefs",
        // Firewall manipulation
        "iptables",
        "ufw",
        "firewalld",
        // System service control (can disable security services)
        "systemctl",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

/// Builds HashSet of suspicious commands (Round 2)
///
/// These commands are **often used maliciously** but have legitimate uses.
/// They warrant a warning but won't block the binding.
///
/// # Categories
/// - **Encoding**: base64, xxd, openssl enc (used to obfuscate payloads)
/// - **Download**: wget, curl, fetch (can download malware)
/// - **Background execution**: nohup, disown, screen, tmux (persistence)
/// - **Obfuscation**: eval, exec, source (dynamic code execution)
///
/// # Common Attack Pattern
/// ```bash
/// echo "cm0gLXJmIC8=" | base64 -d | bash  # Decodes to "rm -rf /"
/// ```
pub fn build_suspicious_commands() -> HashSet<String> {
    vec![
        // Encoding tools (used to obfuscate payloads)
        "base64", "xxd", "uuencode", // Download tools (often download malware)
        "wget", "curl", "fetch", "aria2c",
        // Background execution (malware persistence)
        "nohup", "disown", "screen", "tmux", // Obfuscation and dynamic execution
        "eval", "exec", "source", // Network tools (used in reverse shells)
        "nc", "netcat", "ncat",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

/// Builds HashSet of safe commands (Round 2)
///
/// Whitelisted commands that are known to be safe. This provides:
/// 1. **Performance optimisation**: Fast path for common commands (O(1))
/// 2. **False positive reduction**: Don't flag normal user actions
///
/// # Categories
/// - **Browsers**: firefox, chromium, brave
/// - **Terminals**: kitty, alacritty, foot
/// - **Editors**: nvim, code, emacs
/// - **System tools**: pavucontrol, nm-applet
/// - **Media**: mpv, vlc, spotify
///
/// # Design Choice
/// Common user-facing applications that are safe to execute.
/// Does NOT include interpreters (python, node) - those stay neutral
/// since they can run arbitrary code via arguments.
pub fn build_safe_commands() -> HashSet<String> {
    vec![
        // Browsers
        "firefox",
        "chromium",
        "brave",
        "vivaldi",
        "opera",
        "qutebrowser",
        // Terminals
        "kitty",
        "alacritty",
        "foot",
        "wezterm",
        "terminator",
        "st",
        // Editors
        "nvim",
        "vim",
        "emacs",
        "code",
        "nano",
        "gedit",
        "kate",
        // File managers
        "nautilus",
        "thunar",
        "dolphin",
        "pcmanfm",
        // System tools
        "pavucontrol",
        "nm-applet",
        "blueman",
        // Media
        "mpv",
        "vlc",
        "spotify",
        "obs",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}
