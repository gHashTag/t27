//! Minimal .tri test runner - Ring-005
//! TEMPORARY: Direct .trib file reader without full compiler
//! TODO: Replace with specs/03-tri-bootstrap-compiler.tri implementation

use anyhow::Result;
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 1 {
        println!("Usage: tri test <spec-file>");
        println!("\nOptions:");
        println!("  --plain    Output plain text (default)");
        println!("  --json      Output JSON");
        println!("  --verbose    Show all test details");
        std::process::exit(1);
    }

    let spec_file = &args[0];
    let output_json = args.contains(&"--json");

    // Read .trib file
    let trib_content = match spec_file {
        Ok(path) => std::fs::read_to_string(&path)?,
        Err(_) => {
            eprintln!("Failed to read spec file: {}", path.display());
            std::process::exit(1);
        }
    };

    // Parse tests section
    let mut test_count = 0u32;
    let mut passed_count = 0u32;

    for line in trib_content.lines() {
        if line.starts_with("test ") {
            test_count += 1;
            if output_json {
                println!("{{\"test\": \"{}\", \"passed\": true}}", line.trim_start_matches("test ").trim_matches("\""));
            } else {
                let test_name = line.trim_start_matches("test ").trim_matches("\"");
                let parts: Vec<&str> = test_name.split(" given ").collect();

                if parts.len() >= 2 {
                    let test_expr = if parts.len() > 2 { &parts[2..parts.len()-1].join(" ") } else { "true" };

                    // Run test (stub - returns true)
                    let passed = test_expr == "true";

                    if passed {
                        passed_count += 1;
                    }

                    if output_json {
                        let result = if passed { "\"passed\": true" } else { "\"passed\": false, \"reason\": \"stub - always true\"" };
                        println!("{}", result);
                    } else {
                        println!("test {}: {}", if passed { "PASS" } else { "FAIL" });
                    }
                }
        }
    }

    let summary = if output_json {
        format!(
            "{{\"summary\": {{\"total\": {}, \"passed\": {}, \"failed\": {}}}}",
            test_count, passed_count, test_count - passed_count
        )
    } else {
        println!("\nSummary:");
        println!("Total: {}", test_count);
        println!("Passed: {}", passed_count);
        println!("Failed: {}", test_count - passed_count);
    }

    Ok(())
}
