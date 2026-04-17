// Verify Bootstrap Parser Spec (Ring-002)
use std::fs;

fn main() {
    let spec_path = "specs/02-bootstrap-parser.tri";
    println!("Verifying spec: {}", spec_path);

    match std::fs::read_to_string(spec_path) {
        Ok(content) => {
            let lines: Vec<&str> = content.lines().collect();
            println!("SUCCESS - Spec file found");
            println!("Lines: {}", lines.len());
            println!("\nFirst 50 lines:");
            for (i, line) in lines.iter().take(50).enumerate() {
                println!("  {:3}: {}", i, line.trim());
            }
            println!("\nVerifying key sections:");
            if content.contains("module bootstrap_parser") { println!("  [OK] Module declaration"); }
            if content.contains("SECTION 1: ADT Types") { println!("  [OK] ADT Types section"); }
            if content.contains("NumericTowerVariant") { println!("  [OK] NumericTowerVariant enum"); }
            if content.contains("ExperienceLevel") { println!("  [OK] ExperienceLevel enum"); }
            if content.contains("types {") { println!("  [OK] Types section"); }
            if content.contains("vsa_ops {") { println!("  [OK] VSA operations section"); }
            if content.contains("PHI:") { println!("  [OK] PHI constant"); }
            if content.contains("VSA_N_BITS:") { println!("  [OK] VSA_N_BITS constant"); }
        }
        Err(e) => {
            eprintln!("ERROR - Could not read spec: {:?}", e);
            std::process::exit(1);
        }
    }
}
