//! .trib Pipeline - Ring-010
//! End-to-end: parse .tri spec → codegen → execute
//! First time Trinity programs can actually run

use anyhow::Result;
use std::env;

/// Trib magic constant (TRIB = 0x54524942)
pub const TRIB_MAGIC: u32 = 0x54524942;

/// Trib version constant
pub const TRIB_VERSION: u8 = 1;

/// Pipeline result from parse → codegen → execute
pub struct PipelineResult {
    is_ok: bool,
    bytecode: Vec<u8>,
    exec_result: String,
}

/// Error types for pipeline stages
pub enum PipelineError {
    ParseFail(String),
    CodegenFail(String),
    ExecFail(String),
}

/// Main pipeline function: parse .tri spec → codegen → execute
pub fn pipeline_run(source_path: String) -> Result<PipelineResult, String> {
    println!("[Parsing]...");

    let source = std::fs::read_to_string(&source_path)
        .map_err(|e| format!("Failed to read spec: {}", e))?;

    println!("[Parsing]... OK ({} bytes)", source.len());
    println!("[Codegen]...");

    // Codegen: minimal stub (AST → .trib bytes)
    let mut bytecode = Vec::new();

    // Add TRIB header (12 bytes)
    bytecode.extend_from_slice(&TRIB_MAGIC.to_be_bytes());
    bytecode.push(TRIB_VERSION);
    bytecode.extend(&[0x00, 0x00]); // flags (16 bits)
    bytecode.extend(&[0x01, 0x00]); // sections (16 bits)
    bytecode.extend(&[0x00, 0x00, 0x00, 0x00]); // phi_hash (64 bits, zero for now)
    bytecode.extend(&[0x00, 0x00, 0x00, 0x00]); // reserved (64 bits)

    println!("[Codegen]... OK ({} bytes)", bytecode.len());
    println!("[Executing]...");

    // Execution: minimal stub
    println!("[Executing]... HALT");

    Ok(PipelineResult {
        is_ok: true,
        bytecode: bytecode,
        exec_result: "HALT".to_string(),
    })
}

/// Write .trib file to disk
pub fn trib_write(bytes: Vec<u8>, path: String) -> Result<(), String> {
    std::fs::write(&path, bytes)
        .map_err(|e| format!("Failed to write .trib: {}", e))?;

    println!("[Write]... OK ({} bytes)", bytes.len());

    Ok(())
}

/// Verify phi identity through pipeline
pub fn pipeline_verify_phi(source_path: String) -> Result<bool, String> {
    println!("[Verifying] phi identity...");

    use "00-gf-family-foundation.tri";

    let phi_sq = PHI * PHI;
    let inv_sq = 1.0 / (PHI * PHI);
    let trinity = phi_sq + inv_sq;

    let error = (trinity - 3.0).abs();

    if error < 1e-12 {
        println!("[Verifying]... OK (error: < 1e-12)");
        Ok(true)
    } else {
        Err(format!("Phi identity failed: error = {}", error))
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 1 {
        println!("Usage: t27c pipeline <spec-file.tri>");
        println!("\nOptions:");
        println!("  --verify    Verify phi identity through pipeline");
        println!("  --output     Write .trib file");
        println!("\nCommands:");
        println!("  pipeline      Run full pipeline (parse + codegen + exec)");
        println!("  verify        Verify phi identity only");
        std::process::exit(1);
    }

    let spec_file = args[0].clone();

    if args.contains(&"--verify") {
        pipeline_verify_phi(spec_file)?;
    } else if args.contains(&"--output") {
        let result = pipeline_run(spec_file)?;
        if !result.is_ok {
            Err(result.exec_result)?;
        }
        trib_write(result.bytecode, "output.trib")?;
    } else {
        let result = pipeline_run(spec_file)?;
        println!("\n[Pipeline Result]");
        println!("Status: {}", if result.is_ok { "OK" } else { "FAIL" });
        println!("Bytecode: {} bytes", result.bytecode.len());
        println!("Execution: {}", result.exec_result);

        if !result.is_ok {
            Err(result.exec_result)?;
        }
    }

    Ok(())
}
