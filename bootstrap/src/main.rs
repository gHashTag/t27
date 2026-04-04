// bootstrap/src/main.rs
// T27 Bootstrap Compiler - CLI and HTTP Server Entry Point
//
// Commands:
// - parse: Parse .t27 and output JSON AST
// - gen: Generate Zig code from .t27
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

    /// Compute deterministic test_vector_hash from conformance JSON
    Conformance {
        /// Input conformance JSON file path
        input: String,
    },

    /// Compute seal hashes for a .t27 spec file
    Seal {
        /// Input .t27 spec file path
        input: String,
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

fn run_seal(input_path: &str) -> anyhow::Result<()> {
    let path = Path::new(input_path);
    let source = fs::read_to_string(path)?;

    // spec_hash: SHA256 of the .t27 input file
    let spec_hash = sha256_hex(source.as_bytes());
    println!("spec_hash=sha256:{}", spec_hash);

    // gen_hash: SHA256 of the generated Zig output
    match compiler::Compiler::compile(&source) {
        Ok(zig_code) => {
            let gen_hash = sha256_hex(zig_code.as_bytes());
            println!("gen_hash=sha256:{}", gen_hash);
        }
        Err(e) => {
            eprintln!("gen_hash=error: {}", e);
        }
    }

    // test_vector_hash: look for matching conformance JSON
    let spec_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let conformance_dir = path.parent().unwrap_or(Path::new(".")).join("../conformance");
    if conformance_dir.is_dir() {
        let mut found = false;
        if let Ok(dir_entries) = fs::read_dir(&conformance_dir) {
            for entry in dir_entries.flatten() {
                let entry_path = entry.path();
                if entry_path.extension().and_then(|e| e.to_str()) == Some("json") {
                    let fname = entry_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                    if fname.contains(spec_stem) {
                        // Found a matching conformance file — compute its hash
                        let json_source = fs::read_to_string(&entry_path)?;
                        let json: serde_json::Value = serde_json::from_str(&json_source)?;
                        if let Some(vectors) = json.get("test_vectors").and_then(|v| v.as_array()) {
                            let mut sorted: Vec<&serde_json::Value> = vectors.iter().collect();
                            sorted.sort_by(|a, b| {
                                let na = a.get("name").and_then(|n| n.as_str()).unwrap_or("");
                                let nb = b.get("name").and_then(|n| n.as_str()).unwrap_or("");
                                na.cmp(nb)
                            });
                            let entries: Vec<String> = sorted
                                .iter()
                                .map(|v| serde_json::to_string(v).unwrap_or_default())
                                .collect();
                            let canonical = entries.join("\n");
                            let hash = sha256_hex(canonical.as_bytes());
                            println!("test_vector_hash=sha256:{}", hash);
                            found = true;
                        }
                    }
                }
            }
        }
        if !found {
            println!("test_vector_hash=none");
        }
    } else {
        println!("test_vector_hash=none");
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
        Commands::Conformance { input } => run_conformance(&input)?,
        Commands::Seal { input } => run_seal(&input)?,
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
        Commands::Conformance { input } => run_conformance(&input)?,
        Commands::Seal { input } => run_seal(&input)?,
        Commands::Serve { .. } => {
            eprintln!("Error: 'serve' command requires 'server' feature");
            eprintln!("Build with: cargo build --release --features server");
            std::process::exit(1);
        }
    }

    Ok(())
}
