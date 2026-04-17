// Verify Ring-001 spec
use std::fs;

fn main() {
    let spec_path = "specs/core_trinity.tri";
    println!("Verifying spec: {}", spec_path);

    match std::fs::read_to_string(spec_path) {
        Ok(content) => {
            println!("SUCCESS - Spec file found");
            println!("Lines: {}", content.lines().count());
            println!("\nFirst 200 characters:");
            for (i, line) in content.lines().take(200).enumerate() {
                println!("  {:3}: {}", i, line.trim());
            }
            println!("\nVerifying spec sections:");
            if content.contains("module core_trinity") { println!("  [OK] Module declaration"); }
            if content.contains("PHI:") { println!("  [OK] PHI constant"); }
            if content.contains("TRIT_NEG:") { println!("  [OK] Trit constant"); }
            if content.contains("Trit = enum") { println!("  [OK] Trit type"); }
            if content.contains("vsa_ops") { println!("  [OK] VSA operations section"); }
            if content.contains("permute(") { println!("  [OK] permute operation"); }
            if content.contains("hybrid_add(") { println!("  [OK] hybrid_add operation"); }
            if content.contains("test trit") { println!("  [OK] trit test"); }
            if content.contains("bench permute") { println!("  [OK] permute benchmark"); }
        }
        Err(e) => {
            println!("ERROR - Spec file not found: {:?}", e);
            std::process::exit(1);
        }
    }
}
