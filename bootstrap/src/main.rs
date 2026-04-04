// bootstrap/src/main.rs
// T27 Bootstrap Compiler - CLI and HTTP Server Entry Point
//
// Commands:
// - parse: Parse .t27 and output JSON AST
// - gen: Generate Zig code from .t27
// - gen-verilog: Generate synthesizable Verilog from .t27
// - gen-c: Generate C code from .t27
// - seal: Compute seal hashes (with --save / --verify)
// - serve: Start HTTP server (requires 'server' feature)

mod compiler;

use clap::{Parser, Subcommand};
use sha2::{Sha256, Digest};
#[cfg(feature = "server")]
use std::env;
use std::fs;
use std::path::Path;

// ============================================================================
// CLI Definition (clap)
// ============================================================================

#[derive(Parser)]
#[command(name = "t27c")]
#[command(about = "T27 Bootstrap Compiler for Trinity S³AI Framework", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse .t27 file and output AST
    Parse {
        /// Input file path
        input: String,
    },

    /// Generate Zig code from .t27 file
    Gen {
        /// Input file path
        input: String,
    },

    /// Generate synthesizable Verilog from .t27 file
    GenVerilog {
        /// Input file path
        input: String,
    },

    /// Generate C code (.c/.h style) from .t27 file
    GenC {
        /// Input file path
        input: String,
    },

    /// Compute deterministic test_vector_hash from conformance JSON
    Conformance {
        /// Input conformance JSON file path
        input: String,
    },

    /// Compute seal hashes for a .t27 spec file
    Seal {
        /// Input .t27 spec file path
        input: String,

        /// Save computed hashes to .trinity/seals/<module>.json
        #[arg(long)]
        save: bool,

        /// Verify current hashes match previously saved seals
        #[arg(long)]
        verify: bool,
    },

    /// Start HTTP server on Railway
    Serve {
        /// Port to listen on (default: uses Railway PORT env var)
        #[arg(short, long, default_value = "8080")]
        port: String,
    },
}

// ============================================================================
// HTTP Server (Axum - optional feature)
// ============================================================================

#[cfg(feature = "server")]
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
#[cfg(feature = "server")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use tokio::net::TcpListener;

#[cfg(feature = "server")]
#[derive(Debug, Deserialize)]
struct CompileRequest {
    source: String,
}

#[cfg(feature = "server")]
#[derive(Debug, Serialize)]
struct CompileResponse {
    success: bool,
    zig_code: Option<String>,
    error: Option<String>,
}

#[cfg(feature = "server")]
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    version: &'static str,
}

#[cfg(feature = "server")]
async fn health_handler() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION"),
    })
}

#[cfg(feature = "server")]
async fn compile_handler(
    Json(req): Json<CompileRequest>,
) -> impl IntoResponse {
    match compiler::Compiler::compile(&req.source) {
        Ok(zig_code) => (
            StatusCode::OK,
            Json(CompileResponse {
                success: true,
                zig_code: Some(zig_code),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(CompileResponse {
                success: false,
                zig_code: None,
                error: Some(e),
            }),
        ),
    }
}

#[cfg(feature = "server")]
async fn run_server(port_arg: &str) -> anyhow::Result<()> {
    // Support Railway's $PORT environment variable
    let port = env::var("PORT")
        .unwrap_or_else(|_| port_arg.to_string())
        .parse::<u16>()?;

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/compile", post(compile_handler));

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    println!("t27c server listening on {}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

// ============================================================================
// Command Handlers
// ============================================================================

fn run_parse(input_path: &str) -> anyhow::Result<()> {
    let path = Path::new(input_path);
    let source = fs::read_to_string(path)?;

    match compiler::Compiler::parse_ast(&source) {
        Ok(ast) => println!("{:#?}", ast),
        Err(e) => anyhow::bail!("Parse error: {}", e),
    }
    Ok(())
}

fn run_gen(input_path: &str) -> anyhow::Result<()> {
    let path = Path::new(input_path);
    let source = fs::read_to_string(path)?;

    match compiler::Compiler::compile(&source) {
        Ok(zig_code) => print!("{}", zig_code),
        Err(e) => anyhow::bail!("Compile error: {}", e),
    }
    Ok(())
}

fn run_gen_verilog(input_path: &str) -> anyhow::Result<()> {
    let path = Path::new(input_path);
    let source = fs::read_to_string(path)?;

    match compiler::Compiler::compile_verilog(&source) {
        Ok(verilog_code) => print!("{}", verilog_code),
        Err(e) => anyhow::bail!("Compile error: {}", e),
    }
    Ok(())
}

fn run_gen_c(input_path: &str) -> anyhow::Result<()> {
    let path = Path::new(input_path);
    let source = fs::read_to_string(path)?;

    match compiler::Compiler::compile_c(&source) {
        Ok(c_code) => print!("{}", c_code),
        Err(e) => anyhow::bail!("Compile error: {}", e),
    }
    Ok(())
}

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn run_conformance(input_path: &str) -> anyhow::Result<()> {
    let path = Path::new(input_path);
    let source = fs::read_to_string(path)?;

    let json: serde_json::Value = serde_json::from_str(&source)?;

    // Extract test_vectors array and sort entries by name for determinism
    let mut entries: Vec<String> = Vec::new();

    if let Some(vectors) = json.get("test_vectors").and_then(|v| v.as_array()) {
        let mut sorted_vectors: Vec<&serde_json::Value> = vectors.iter().collect();
        sorted_vectors.sort_by(|a, b| {
            let name_a = a.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let name_b = b.get("name").and_then(|n| n.as_str()).unwrap_or("");
            name_a.cmp(name_b)
        });
        for v in sorted_vectors {
            entries.push(serde_json::to_string(v)?);
        }
    } else {
        // Fallback: sort top-level keys for non-vector JSON files
        if let Some(obj) = json.as_object() {
            let mut keys: Vec<&String> = obj.keys().collect();
            keys.sort();
            for k in keys {
                entries.push(format!("{}:{}", k, serde_json::to_string(&obj[k])?));
            }
        } else {
            entries.push(serde_json::to_string(&json)?);
        }
    }

    let canonical = entries.join("\n");
    let hash = sha256_hex(canonical.as_bytes());
    println!("test_vector_hash=sha256:{}", hash);
    Ok(())
}

/// Extract module name from .t27 source (first `module <name>;` declaration)
fn extract_module_name(source: &str) -> Option<String> {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("module ") {
            let rest = trimmed.strip_prefix("module ").unwrap().trim();
            let name = rest.trim_end_matches(';').trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
}

/// Collected seal hashes for a spec file
struct SealHashes {
    module: String,
    spec_path: String,
    spec_hash: String,
    gen_hash_zig: String,
    gen_hash_verilog: String,
    gen_hash_c: String,
}

/// Compute all seal hashes for a .t27 spec file
fn compute_seal_hashes(input_path: &str) -> anyhow::Result<SealHashes> {
    let path = Path::new(input_path);
    let source = fs::read_to_string(path)?;

    let module = extract_module_name(&source)
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string()
        });

    let spec_hash = format!("sha256:{}", sha256_hex(source.as_bytes()));

    let gen_hash_zig = match compiler::Compiler::compile(&source) {
        Ok(zig_code) => format!("sha256:{}", sha256_hex(zig_code.as_bytes())),
        Err(_) => "none".to_string(),
    };

    let gen_hash_verilog = match compiler::Compiler::compile_verilog(&source) {
        Ok(verilog_code) => format!("sha256:{}", sha256_hex(verilog_code.as_bytes())),
        Err(_) => "none".to_string(),
    };

    let gen_hash_c = match compiler::Compiler::compile_c(&source) {
        Ok(c_code) => format!("sha256:{}", sha256_hex(c_code.as_bytes())),
        Err(_) => "none".to_string(),
    };

    Ok(SealHashes {
        module,
        spec_path: input_path.to_string(),
        spec_hash,
        gen_hash_zig,
        gen_hash_verilog,
        gen_hash_c,
    })
}

/// Path to the seal JSON file for a given module
fn seal_file_path(module: &str) -> std::path::PathBuf {
    Path::new(".trinity").join("seals").join(format!("{}.json", module))
}

fn run_seal(input_path: &str, save: bool, verify: bool) -> anyhow::Result<()> {
    let hashes = compute_seal_hashes(input_path)?;

    if verify {
        // --verify: load saved seal and compare
        let seal_path = seal_file_path(&hashes.module);
        if !seal_path.exists() {
            anyhow::bail!(
                "No saved seal found at {}. Run with --save first.",
                seal_path.display()
            );
        }
        let saved_json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&seal_path)?)?;

        let mut all_match = true;
        let checks = [
            ("spec_hash", &hashes.spec_hash),
            ("gen_hash_zig", &hashes.gen_hash_zig),
            ("gen_hash_verilog", &hashes.gen_hash_verilog),
            ("gen_hash_c", &hashes.gen_hash_c),
        ];

        for (field, current) in &checks {
            let saved = saved_json
                .get(field)
                .and_then(|v| v.as_str())
                .unwrap_or("missing");
            if *current == saved {
                println!("{}: MATCH", field);
            } else {
                println!("{}: MISMATCH (saved={}, current={})", field, saved, current);
                all_match = false;
            }
        }

        if all_match {
            println!("\nall hashes MATCH");
        } else {
            println!("\nVERIFICATION FAILED — hashes differ from saved seal");
            std::process::exit(1);
        }
    } else if save {
        // --save: compute hashes and write to .trinity/seals/<module>.json
        let seals_dir = Path::new(".trinity").join("seals");
        fs::create_dir_all(&seals_dir)?;

        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        let seal_obj = serde_json::json!({
            "module": hashes.module,
            "spec_path": hashes.spec_path,
            "spec_hash": hashes.spec_hash,
            "gen_hash_zig": hashes.gen_hash_zig,
            "gen_hash_verilog": hashes.gen_hash_verilog,
            "gen_hash_c": hashes.gen_hash_c,
            "sealed_at": now,
            "ring": 12
        });

        let seal_path = seal_file_path(&hashes.module);
        let pretty = serde_json::to_string_pretty(&seal_obj)?;
        fs::write(&seal_path, &pretty)?;

        // Also print hashes to stdout
        println!("spec_hash={}", hashes.spec_hash);
        println!("gen_hash_zig={}", hashes.gen_hash_zig);
        println!("gen_hash_verilog={}", hashes.gen_hash_verilog);
        println!("gen_hash_c={}", hashes.gen_hash_c);
        println!("\nSeal saved to {}", seal_path.display());
    } else {
        // Default: just print hashes (existing behavior, enhanced with all backends)
        println!("spec_hash={}", hashes.spec_hash);
        println!("gen_hash_zig={}", hashes.gen_hash_zig);
        println!("gen_hash_verilog={}", hashes.gen_hash_verilog);
        println!("gen_hash_c={}", hashes.gen_hash_c);
    }

    Ok(())
}

// ============================================================================
// Main Entry Point
// ============================================================================

#[cfg(feature = "server")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse { input } => run_parse(&input)?,
        Commands::Gen { input } => run_gen(&input)?,
        Commands::GenVerilog { input } => run_gen_verilog(&input)?,
        Commands::GenC { input } => run_gen_c(&input)?,
        Commands::Conformance { input } => run_conformance(&input)?,
        Commands::Seal { input, save, verify } => run_seal(&input, save, verify)?,
        Commands::Serve { port } => run_server(&port).await?,
    }

    Ok(())
}

#[cfg(not(feature = "server"))]
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse { input } => run_parse(&input)?,
        Commands::Gen { input } => run_gen(&input)?,
        Commands::GenVerilog { input } => run_gen_verilog(&input)?,
        Commands::GenC { input } => run_gen_c(&input)?,
        Commands::Conformance { input } => run_conformance(&input)?,
        Commands::Seal { input, save, verify } => run_seal(&input, save, verify)?,
        Commands::Serve { .. } => {
            eprintln!("Error: 'serve' command requires 'server' feature");
            eprintln!("Build with: cargo build --release --features server");
            std::process::exit(1);
        }
    }

    Ok(())
}
