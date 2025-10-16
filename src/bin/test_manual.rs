// Copyright 2025 bakri (tidynest@proton.me)
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

//! Manual verification of entropy detection
//!
//! This binary provides manual testing for the entropy-based danger detection
//! system. It runs several test cases including:
//!
//! - Base64-encoded malicious commands
//! - Hex-encoded malicious commands
//! - Perl packed commands
//! - Normal safe commands (to verify no false positives)
//!
//! # Usage
//!
//! ```bash
//! cargo run --bin test_manual
//! ```
//!
//! The output shows danger level assessments for each test case,
//! verifying that the entropy detector correctly identifies obfuscated
//! commands while not flagging normal commands.

use hypr_keybind_manager::config::danger::DangerDetector;

fn main() {
    let detector = DangerDetector::new();

    println!("=== Manual Entropy Detection Tests ===\n");

    // Test 1: Base64-encoded "rm -rf /"
    let test1 = "echo cm0gLXJmIC8= | base64 -d | bash";
    let assess1 = detector.assess_command(test1);
    println!("Test 1 - Base64 attack:");
    println!("  Command: {}", test1);
    println!("  Level: {:?}", assess1.danger_level);
    println!("  Reason: {}", assess1.reason);
    println!("  Recommendation: {}\n", assess1.recommendation);

    // Test 2: Hex-encoded "rm -rf /"
    let test2 = "echo 726d202d7266202f | xxd -r -p | bash";
    let assess2 = detector.assess_command(test2);
    println!("Test 2 - Hex attack:");
    println!("  Command: {}", test2);
    println!("  Level: {:?}", assess2.danger_level);
    println!("  Reason: {}", assess2.reason);
    println!("  Recommendation: {}\n", assess2.recommendation);

    // Test 3: Perl packed hex
    let test3 = "perl -e 'print pack(\"H*\", \"726d202d7266202f\")'";
    let assess3 = detector.assess_command(test3);
    println!("Test 3 - Perl packed command:");
    println!("  Command: {}", test3);
    println!("  Level: {:?}", assess3.danger_level);
    println!("  Reason: {}", assess3.reason);
    println!("  Recommendation: {}\n", assess3.recommendation);

    // Test 4: Normal command (should be safe)
    let test4 = "firefox https://rust-lang.org";
    let assess4 = detector.assess_command(test4);
    println!("Test 4 - Normal safe command:");
    println!("  Command: {}", test4);
    println!("  Level: {:?}", assess4.danger_level);
    println!("  Reason: {}\n", assess4.reason);

    // Test 5: Command name that looks like base64 (should NOT false positive)
    let test5 = "uuencode file output";
    let assess5 = detector.assess_command(test5);
    println!("Test 5 - Suspicious tool (not encoded data):");
    println!("  Command: {}", test5);
    println!("  Level: {:?}", assess5.danger_level);
    println!("  Reason: {}\n", assess5.reason);

    println!("=== All manual tests complete! ===");
}