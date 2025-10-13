//! Manual verification of entropy detection

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