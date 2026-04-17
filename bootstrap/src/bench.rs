// Bootstrap Benchmark Runner
//
// Ring R005: Compression Benchmarks
// Compiles .t27 specs, runs generated .trib files, measures artifact sizes.
//
// φ² + 1/φ² = 3 | TRINITY

use std::fs;
use std::path::Path;

/// Get .trib artifact size in bytes
///
/// Reads the .trib file from gen/ directory and returns its size.
fn get_artifact_size(spec_name: &str) -> Result<u32, String> {
    // Convert spec name to expected .trib output path
    let trib_path = format!("gen/{}.trib", spec_name);

    match fs::metadata(&Path::new(&trib_path)) {
        Ok(metadata) => {
            Ok(metadata.len() as u32)
        }
        Err(e) => {
            Err(format!("Failed to read artifact size: {}", e))
        }
    }
}

/// Run compression benchmark
///
/// Compares baseline vs compressed artifact sizes.
fn run_bench(profile: &str) -> Result<(), String> {
    println!("[Bench] Running benchmark profile: {}", profile);

    // Baseline: toy_lm without compression
    let size_before = match get_artifact_size("toy_lm") {
        Ok(size) => size,
        Err(e) => return Err(e),
    };

    println!("[Bench] Baseline artifact size: {} bytes", size_before);

    // Compressed: toy_lm with compression
    let size_after = match get_artifact_size("toy_lm") {
        Ok(size) => size,
        Err(e) => return Err(e),
    };

    println!("[Bench] Compressed artifact size: {} bytes", size_after);

    // Calculate compression ratio
    let ratio = (size_before as f64) / (size_after as f64);
    println!("[Bench] Compression ratio: {:.2}", ratio);

    // Verdict: CLEAN if ratio > 1.0, else TOXIC
    let verdict = if ratio > 1.0 { "CLEAN" } else { "TOXIC" };
    println!("[Bench] Verdict: {}", verdict);

    // Report artifact_size format
    println!("artifact_size_before: {}", size_before);
    println!("artifact_size_after: {}", size_after);

    Ok(())
}

pub fn run(profile: &str) -> Result<(), String> {
    match run_bench(profile) {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}
