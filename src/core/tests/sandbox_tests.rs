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

use crate::core::sandbox;

#[test]
fn test_wrap_command_adds_bubblewrap_prefix() {
    let wrapped = sandbox::wrap_command("firefox --private-window").unwrap();

    assert!(wrapped.starts_with("bwrap --die-with-parent --new-session --unshare-net"));
    assert!(wrapped.ends_with("-- firefox --private-window"));
}

#[test]
fn test_unwrap_command_recovers_original_command() {
    let wrapped = sandbox::wrap_command("kitty").unwrap();

    assert_eq!(sandbox::unwrap_command(&wrapped).as_deref(), Some("kitty"));
}

#[test]
fn test_is_wrapped_rejects_plain_exec_command() {
    assert!(!sandbox::is_wrapped("firefox"));
}
