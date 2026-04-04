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

    /// Compile a .t27 file and write generated code to a file
    Compile {
        /// Input file path
        input: String,
        /// Backend: zig, verilog, or c
        #[arg(long, default_value = "zig")]
        backend: String,
        /// Output file path (default: input with backend extension)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Compile all .t27 files from specs/ and compiler/ into an output directory
    CompileAll {
        /// Backend: zig, verilog, or c
        #[arg(long, default_value = "zig")]
        backend: String,
        /// Output directory
        #[arg(short, long, default_value = "build")]
        output: String,
        /// Path to directory containing specs/ and compiler/ (auto-detected if omitted)
        #[arg(long)]
        specs_dir: Option<String>,
    },

    /// Show repository statistics
    Stats,

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
            let name = rest.trim_end_matches(';').trim_end_matches('{').trim();
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
// Compile Commands
// ============================================================================

fn backend_extension(backend: &str) -> &str {
    match backend {
        "verilog" => ".v",
        "c" => ".c",
        _ => ".zig",
    }
}

fn compile_source(source: &str, backend: &str) -> Result<String, String> {
    match backend {
        "verilog" => compiler::Compiler::compile_verilog(source),
        "c" => compiler::Compiler::compile_c(source),
        _ => compiler::Compiler::compile(source),
    }
}

fn run_compile(input_path: &str, backend: &str, output: Option<&str>) -> anyhow::Result<()> {
    let path = Path::new(input_path);
    let source = fs::read_to_string(path)?;

    let code = compile_source(&source, backend)
        .map_err(|e| anyhow::anyhow!("Compile error: {}", e))?;

    let out_path = match output {
        Some(p) => std::path::PathBuf::from(p),
        None => {
            let stem = path.file_stem().unwrap_or_default();
            let ext = backend_extension(backend);
            path.with_file_name(format!("{}{}", stem.to_string_lossy(), ext))
        }
    };

    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&out_path, &code)?;
    println!("wrote {}", out_path.display());
    Ok(())
}

/// Auto-detect the repository root by looking for a directory containing specs/.
/// Searches CWD first, then up to 3 parent directories.
fn find_repo_root() -> Option<std::path::PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let mut dir = cwd.as_path();
    for _ in 0..4 {
        if dir.join("specs").is_dir() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
    None
}

fn run_compile_all(backend: &str, output_dir: &str, specs_dir: Option<&str>) -> anyhow::Result<()> {
    let root = match specs_dir {
        Some(d) => std::path::PathBuf::from(d),
        None => find_repo_root()
            .ok_or_else(|| anyhow::anyhow!(
                "Could not find specs/ directory. Run from the repo root or use --specs-dir"
            ))?,
    };

    let ext = backend_extension(backend);
    let out_base = Path::new(output_dir);
    let mut count = 0u32;

    // Count total .t27 files first for the progress message
    let dirs = ["specs", "compiler"];
    let mut total = 0u32;
    for dir in &dirs {
        let base = root.join(dir);
        if !base.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&base)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().extension().and_then(|e| e.to_str()) == Some("t27") {
                total += 1;
            }
        }
    }

    println!("Compiling {} files from {} to {}/", total, root.display(), output_dir);

    for dir in &dirs {
        let base = root.join(dir);
        if !base.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&base)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("t27") {
                continue;
            }
            let source = fs::read_to_string(p)?;
            let code = match compile_source(&source, backend) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("skip {}: {}", p.display(), e);
                    continue;
                }
            };
            // Preserve directory structure: specs/base/types.t27 -> build/specs/base/types.zig
            let rel = p.strip_prefix(&root).unwrap_or(p);
            let dest = out_base.join(rel).with_extension(&ext[1..]);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&dest, &code)?;
            println!("wrote {}", dest.display());
            count += 1;
        }
    }

    println!("\ncompiled {} files to {}/", count, output_dir);
    Ok(())
}

// ============================================================================
// Stats Command
// ============================================================================

fn count_pattern_in_dir(root: &Path, dirs: &[&str], pattern: &str) -> u32 {
    let mut count = 0u32;
    for dir in dirs {
        let base = root.join(dir);
        if !base.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&base)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("t27") {
                continue;
            }
            if let Ok(contents) = fs::read_to_string(p) {
                for line in contents.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with(pattern) {
                        count += 1;
                    }
                }
            }
        }
    }
    count
}

fn count_t27_files(root: &Path, dir: &str) -> u32 {
    let base = root.join(dir);
    if !base.exists() {
        return 0;
    }
    let mut count = 0u32;
    for entry in walkdir::WalkDir::new(&base)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.path().extension().and_then(|e| e.to_str()) == Some("t27") {
            count += 1;
        }
    }
    count
}

fn count_lines(path: &Path) -> u32 {
    if let Ok(contents) = fs::read_to_string(path) {
        contents.lines().count() as u32
    } else {
        0
    }
}

fn count_files_in_dir(dir: &Path, ext: &str) -> u32 {
    if !dir.exists() {
        return 0;
    }
    let mut count = 0u32;
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.path().extension().and_then(|e| e.to_str()) == Some(ext) {
            count += 1;
        }
    }
    count
}

fn run_stats() -> anyhow::Result<()> {
    let root = find_repo_root()
        .ok_or_else(|| anyhow::anyhow!(
            "Could not find specs/ directory. Run from the repo root or use --specs-dir with compile-all"
        ))?;

    let dirs = &["specs", "compiler"];

    let specs_count = count_t27_files(&root, "specs");
    let compiler_count = count_t27_files(&root, "compiler");
    let total_specs = specs_count + compiler_count;

    let functions = count_pattern_in_dir(&root, dirs, "fn ");
    let tests = count_pattern_in_dir(&root, dirs, "test ");
    let invariants = count_pattern_in_dir(&root, dirs, "invariant ");
    let benchmarks = count_pattern_in_dir(&root, dirs, "bench ");

    let conformance_count = count_files_in_dir(&root.join("conformance"), "json");

    let seals_dir = root.join(".trinity").join("seals");
    let seals_count = count_files_in_dir(&seals_dir, "json");

    let compiler_loc = count_lines(&root.join("bootstrap").join("src").join("compiler.rs"));

    // Count CLI commands by reading the Commands enum variants
    // Variants are lines like "    Parse {" or "    Stats," at exactly 4-space indent
    let cli_commands = {
        let main_rs = root.join("bootstrap").join("src").join("main.rs");
        if let Ok(contents) = fs::read_to_string(&main_rs) {
            let mut in_enum = false;
            let mut count = 0u32;
            for line in contents.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("enum Commands") {
                    in_enum = true;
                    continue;
                }
                if in_enum {
                    if trimmed == "}" {
                        break;
                    }
                    // Variant lines start with an uppercase letter
                    if let Some(first) = trimmed.chars().next() {
                        if first.is_uppercase() && (trimmed.contains('{') || trimmed.contains(',') || trimmed.ends_with('{')) {
                            count += 1;
                        }
                    }
                }
            }
            count
        } else {
            0
        }
    };

    // Detect latest ring from experience episodes.jsonl and seal files
    let fixed_point_ring = {
        let mut max_ring = 0u32;

        // Check .trinity/experience/episodes.jsonl (each line is a JSON object with "metadata.ring" or top-level "ring")
        let episodes_jsonl = root.join(".trinity").join("experience").join("episodes.jsonl");
        if episodes_jsonl.exists() {
            if let Ok(contents) = fs::read_to_string(&episodes_jsonl) {
                for line in contents.lines() {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                        // Check metadata.ring first, then top-level ring
                        let ring = json.get("metadata")
                            .and_then(|m| m.get("ring"))
                            .and_then(|r| r.as_u64())
                            .or_else(|| json.get("ring").and_then(|r| r.as_u64()));
                        if let Some(r) = ring {
                            if r as u32 > max_ring {
                                max_ring = r as u32;
                            }
                        }
                    }
                }
            }
        }

        // Also check seal files for ring values
        let seals_dir = root.join(".trinity").join("seals");
        if seals_dir.exists() {
            for entry in walkdir::WalkDir::new(&seals_dir)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.path().extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Ok(contents) = fs::read_to_string(entry.path()) {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&contents) {
                            if let Some(ring) = json.get("ring").and_then(|r| r.as_u64()) {
                                if ring as u32 > max_ring {
                                    max_ring = ring as u32;
                                }
                            }
                        }
                    }
                }
            }
        }

        max_ring
    };

    println!("T27 Repository Statistics");
    println!("========================");
    println!("Spec files:     {} ({} in specs/, {} in compiler/)", total_specs, specs_count, compiler_count);
    println!("Functions:      {}", functions);
    println!("Tests:          {}", tests);
    println!("Invariants:     {}", invariants);
    println!("Benchmarks:     {}", benchmarks);
    println!("Conformance:    {} JSON files", conformance_count);
    println!("Seals:          {} saved", seals_count);
    println!("Backends:       3 (Zig, Verilog, C)");
    println!("CLI commands:   {}", cli_commands);
    println!("Compiler LOC:   {}", compiler_loc);
    if fixed_point_ring > 0 {
        println!("Fixed point:    REACHED (ring-{})", fixed_point_ring);
    } else {
        println!("Fixed point:    NOT REACHED");
    }
    println!("phi^2 + 1/phi^2 = 3 | TRINITY");

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
        Commands::Compile { input, backend, output } => {
            run_compile(&input, &backend, output.as_deref())?
        }
        Commands::CompileAll { backend, output, specs_dir } => {
            run_compile_all(&backend, &output, specs_dir.as_deref())?
        }
        Commands::Stats => run_stats()?,
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
        Commands::Compile { input, backend, output } => {
            run_compile(&input, &backend, output.as_deref())?
        }
        Commands::CompileAll { backend, output, specs_dir } => {
            run_compile_all(&backend, &output, specs_dir.as_deref())?
        }
        Commands::Stats => run_stats()?,
        Commands::Serve { .. } => {
            eprintln!("Error: 'serve' command requires 'server' feature");
            eprintln!("Build with: cargo build --release --features server");
            std::process::exit(1);
        }
    }

    Ok(())
}
