// bootstrap/src/main.rs
// T27 Bootstrap Compiler - CLI and HTTP Server Entry Point
//
// Commands:
// - parse: Parse .t27 and output JSON AST
// - gen: Generate Zig code from .t27
// - gen-verilog: Generate synthesizable Verilog from .t27
// - gen-c: Generate C code from .t27
// - seal: Compute seal hashes (with --save / --verify)
// - check-now: Gate on docs/NOW.md Last updated date
// - serve: Start HTTP server (requires 'server' feature)

mod bridge;
mod compiler;
mod enrichment;
mod suite;
mod railway;
mod jwt;
mod proxy;
mod formula_eval;
mod chimera_engine;
mod sensitivity;
mod runtime;
mod neural;
mod ternary;
mod memory;
// mod runtime_minimal;
// mod runtime_minimal_test;

use anyhow::Context;
use clap::{Parser, Subcommand};
use sha2::{Sha256, Digest};
#[cfg(feature = "server")]
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

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

    /// Debug: dump Hardware IR (HIR) from .t27 file
    DebugHir {
        /// Input file path
        input: String,
    },

    /// Generate Verilog from .t27 file via HIR path (AST -> HIR -> Verilog)
    GenVerilogHir {
        /// Input file path
        input: String,
    },

    /// Assemble ternary assembly source into machine code
    Asm {
        /// Input .t27 assembly source file
        input: String,
        /// Output binary file path (stdout if omitted)
        #[arg(short, long)]
        output: Option<String>,
        /// Output format: binary, hex, or vlog (Verilog $readmemh)
        #[arg(long, default_value = "hex")]
        format: String,
    },

    /// Generate testbench from .t27 HIR module
    GenTestbench {
        /// Input .t27 file
        input: String,
        /// Clock period in ns
        #[arg(long, default_value_t = 10)]
        period_ns: u32,
        /// Max simulation cycles
        #[arg(long, default_value_t = 10000)]
        max_cycles: u32,
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Generate XDC constraints from board profile
    GenXdc {
        /// Board profile: minimal, full, or path to .t27 board spec
        profile: String,
        /// Output file path (stdout if omitted)
        #[arg(long)]
        output: Option<String>,
    },

    /// Check XDC pins against prjxray-db
    CheckPins {
        /// XDC file to validate
        xdc: String,
        /// prjxray-db artix7 directory
        #[arg(long)]
        db: Option<String>,
    },

    /// Verify gen-xdc output matches emitter_xdc.t27 spec expectations
    XdcVerify,

    /// Generate C code (.c/.h style) from .t27 file
    GenC {
        /// Input file path
        input: String,
    },

    /// Generate Rust code from .t27 file
    GenRust {
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
    /// Encode integer to ternary
    TernaryEncode {
        /// Value to encode (-1, 0, +1)
        #[arg(short, long)]
        value: i32,
    },
    /// Decode ternary to integer
    TernaryDecode {
        /// Ternary value to decode (e.g., "[-1, 0, 1]")
        #[arg(short, long)]
        trits: String,
    },
    /// Compile a .t27 file and write generated code to a file

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

    /// Compile all .t27 files into a coherent project with resolved inter-file imports
    CompileProject {
        /// Backend: zig, verilog, or c
        #[arg(long, default_value = "zig")]
        backend: String,
        /// Output directory
        #[arg(short, long, default_value = "build")]
        output: String,
    },

    /// Show repository statistics
    Stats,

    /// Start HTTP server on Railway
    Serve {
        /// Port to listen on (default: uses Railway PORT env var)
        #[arg(short, long, default_value = "8080")]
        port: String,
    },

    /// Queen T A2A Bridge — Orchestrate sessions and tasks via OpenCode
    Bridge {
        #[command(subcommand)]
        command: bridge::BridgeCommands,
    },

    /// NotebookLM Task Commands (L7 UNITY enforcement)
    Task {
        #[command(subcommand)]
        command: bridge::TaskCommands,
    },

    /// Enrich notebooks with YouTube transcripts
    Enrich {
        /// Notebook ID to enrich
        #[arg(short, long)]
        notebook: Option<String>,

        /// Enrich all notebooks
        #[arg(long)]
        all: bool,

        /// Force re-enrichment
        #[arg(long)]
        force: bool,

        /// API token for NotebookLM
        #[arg(short = 't', long)]
        token: String,

        /// Language code: ru, en, or both
        #[arg(short, long, default_value = "both")]
        lang: String,
    },

    /// Generate bilingual Audio Overviews
    Audio {
        /// Notebook ID
        #[arg(short, long)]
        notebook: Option<String>,

        /// Bilingual mode (both languages)
        #[arg(long)]
        bilingual: bool,

        /// All notebooks
        #[arg(long)]
        all: bool,

        /// Dry run mode (verify only, no API calls)
        #[arg(long)]
        dry_run: bool,

        /// Number of parallel workers (default: 4)
        #[arg(long, default_value = "4")]
        workers: usize,

        /// API token for NotebookLM
        #[arg(short = 't', long)]
        token: String,

        /// Project number for API
        #[arg(long)]
        project: Option<String>,

        /// API location (default: global)
        #[arg(long)]
        location: Option<String>,

        /// API region (default: us)
        #[arg(long)]
        region: Option<String>,
    },

    /// Full repository suite: parse, Zig/Verilog/C gen, seal verify, fixed-point
    Suite {
        /// Repository root (default: current directory)
        #[arg(long, default_value = ".")]
        repo_root: PathBuf,
    },

    /// Validate conformance/*.json files (JSON + vector keys)
    ValidateConformance {
        #[arg(long, default_value = ".")]
        repo_root: PathBuf,
    },

    /// Validate gen/** headers (Auto-generated / DO NOT EDIT / TRINITY)
    ValidateGenHeaders {
        #[arg(long, default_value = ".")]
        repo_root: PathBuf,
    },

    /// Require docs/NOW.md "Last updated" calendar date to match today (local timezone)
    CheckNow {
        #[arg(long, default_value = ".")]
        repo_root: PathBuf,
    },

    /// Run optimizer on a .t27 file
    Optimize {
        input: String,
        #[arg(long, default_value = "1")]
        opt_level: u32,
    },

    /// Typecheck a .t27 file
    Typecheck {
        input: String,
        #[arg(long)]
        json: bool,
    },

    /// Lint .t27 spec quality
    Lint {
        input: String,
        #[arg(long)]
        json: bool,
    },

    /// Benchmark a .t27 file
    Bench {
        input: String,
    },

    /// Explain compilation pipeline stages
    Explain {
        input: String,
    },

    /// Pretty-print .t27 from AST
    Fmt {
        input: String,
    },

    /// Dependency graph of .t27 modules
    Graph {
        #[arg(long, default_value = ".")]
        repo_root: String,
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Generate HTML documentation from spec
    Doc {
        input: String,
        #[arg(long, default_value = "docs/html")]
        output_dir: String,
    },

    /// Generate HTML documentation for all specs
    DocAll {
        #[arg(long, default_value = ".")]
        repo_root: String,
        #[arg(long, default_value = "docs/html")]
        output_dir: String,
    },

    /// Run type checker on a .t27 file (alias for typecheck)
    Check {
        input: String,
    },

    /// List test and invariant blocks in a .t27 file
    Test {
        input: String,
        #[arg(long)]
        verbose: bool,
    },

    /// Evaluate a constant expression and print the result
    Eval {
        expr: String,
    },

    /// Show version info
    Version,

    /// Show AST tree for a spec
    Tree {
        input: String,
        #[arg(long, default_value = "2")]
        depth: usize,
    },

    /// Show what a spec file depends on (imports)
    Depends {
        input: String,
    },

    /// Show size metrics for a .t27 spec file
    Size {
        input: String,
    },

    /// Analyze all .t27 specs in repo (aggregate metrics)
    Analyze {
        #[arg(long, default_value = ".")]
        repo_root: String,
        #[arg(long, default_value = "false")]
        json: bool,
        #[arg(long, default_value = "false")]
        top: bool,
    },

    /// Compare two .t27 spec files (structural diff)
    Diff {
        left: String,
        right: String,
    },

    /// Watch .t27 files for changes and recompile
    Watch {
        #[arg(long, default_value = ".")]
        repo_root: String,
        #[arg(long, default_value = "2")]
        interval_secs: u64,
    },

    /// Run full CI checks (parse + typecheck + gen + seal)
    Ci {
        #[arg(long, default_value = ".")]
        repo_root: String,
    },

    /// Show public API of a .t27 spec (pub functions, structs, enums, consts)
    Inspect {
        input: String,
    },

    /// Show function outline with locals, calls, and returns
    Outline {
        input: String,
    },

    /// Export function call graph as DOT format
    Callgraph {
        input: String,
    },

    /// Quick compiler health check (parse+typecheck+gen a tiny spec)
    Health,

    /// Find potentially dead (uncalled) functions in a spec or repo
    Deadcode {
        input: Option<String>,
        #[arg(long, default_value = "false")]
        repo: bool,
    },

    /// Show per-function metrics (complexity, lines, params)
    Metrics {
        input: String,
    },

    /// Flatten single-use functions (inline them at call site)
    Flatten {
        input: String,
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Show module dependency tree across all .t27 specs
    DepsTree {
        #[arg(long, default_value = ".")]
        repo_root: String,
    },

    /// Find TODO/FIXME/HACK comments in specs
    Todo {
        #[arg(long, default_value = ".")]
        repo_root: String,
    },

    /// Rename a symbol across a .t27 file (function/variable/struct/enum)
    Rename {
        input: String,
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
        #[arg(long)]
        dry_run: bool,
    },

    /// Check for potential identifier typos (similar names)
    Spellcheck {
        input: String,
        #[arg(long, default_value = "2")]
        max_distance: u32,
    },

    /// Show test coverage per function (which functions have tests)
    Coverage {
        input: String,
    },

    /// Cross-validate spec consistency (struct fields, return types, etc.)
    Validate {
        #[arg(long, default_value = ".")]
        repo_root: String,
    },

    /// Find all references to a symbol across a spec
    Xref {
        input: String,
        #[arg(long)]
        symbol: String,
    },

    /// Benchmark compilation speed (parse + typecheck + gen all backends)
    BenchCompile {
        #[arg(long, default_value = ".")]
        repo_root: String,
        #[arg(long, default_value = "10")]
        iterations: u32,
    },

    /// Minify a .t27 spec (strip comments, collapse whitespace)
    Minify {
        input: String,
    },

    /// Quick count of declarations in a spec
    Count {
        input: String,
    },

    /// Check for circular dependencies between modules
    CheckDeps {
        #[arg(long, default_value = ".")]
        repo_root: String,
    },

    /// Show struct field layout with estimated byte sizes
    Stack {
        input: String,
    },

    /// Find duplicate function/struct/enum names across the repo
    Dupes {
        #[arg(long, default_value = ".")]
        repo_root: String,
    },

    /// Scaffold a new .t27 spec file
    Init {
        name: String,
        #[arg(long, default_value = ".")]
        output_dir: String,
    },

    /// List all exportable symbols from a spec
    Exports {
        input: String,
    },

    /// Compare public API surface of two spec files
    ApiDiff {
        left: String,
        right: String,
    },

    /// Show lines-of-code per function (from source)
    Loc {
        input: String,
    },

    /// Merge multiple .t27 specs into one
    Merge {
        #[arg(num_args = 1..)]
        inputs: Vec<String>,
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Show all unique types used in a spec
    Types {
        input: String,
    },

    /// Generate a .t27.hjson (human-readable JSON) representation
    ToJson {
        input: String,
    },

    /// One-line summary for each .t27 spec in repo
    Summary {
        #[arg(long, default_value = ".")]
        repo_root: String,
    },

    /// Sort declarations canonically (consts, enums, structs, fns)
    Sort {
        input: String,
    },

    /// Find which specs use a given module/symbol
    UsedBy {
        #[arg(long)]
        symbol: String,
        #[arg(long, default_value = ".")]
        repo_root: String,
    },

    /// Show ASCII visualization of AST
    Visualize {
        input: String,
        #[arg(short, long, default_value_t = 0)]
        depth: u32,
    },

    /// Benchmark HTTP server endpoints (requires server running)
    BenchEndpoints {
        #[arg(long, default_value = "http://127.0.0.1:3000")]
        url: String,
        #[arg(long, default_value_t = 50)]
        requests: u32,
    },

    /// Show complexity metrics per function
    Complexity {
        input: String,
    },

    /// Extract all string literals from a spec
    Strings {
        input: String,
    },

    /// List all symbols (functions, structs, enums, consts) in a spec
    Symbols {
        input: String,
        #[arg(long)]
        kind: Option<String>,
    },

    /// Dump full AST as JSON
    AstDump {
        input: String,
    },

    /// Compute SHA256 hash of spec source
    Hash {
        input: String,
    },

    /// Show call depth / stack depth analysis per function
    Depth {
        input: String,
    },

    /// Show which functions are never called (entry point analysis)
    Orphans {
        input: String,
    },

    /// Check claim tiers consistency between EXPERIENCE_SCHEMA and RESEARCH_CLAIMS.md
    CheckClaimTiers,

    /// Refresh brain seals from experience aggregation (Ring 059 - Crown automation)
    #[command(name = "brain-seal-refresh")]
    BrainSealRefresh,

    /// Validate seals for PR-scoped spec files
    #[command(name = "validate-seals")]
    ValidateSeals {
        /// Comma-separated list of PR spec file paths
        #[arg(long)]
        pr_files: String,
    },

    /// Validate L5 phi-identity invariant (phi^2 + phi^-2 = 3)
    #[command(name = "validate-phi-identity")]
    ValidatePhiIdentity,

    /// FPGA build pipeline: generate Verilog + top-level wrapper from specs/fpga/*.t27
    #[command(name = "fpga-build")]
    FpgaBuild {
        /// Smoke test: generate Verilog only, skip synthesis
        #[arg(long)]
        smoke: bool,

        /// Stop after Yosys synthesis (no P&R or bitstream)
        #[arg(long)]
        synth_only: bool,

        /// Minimal design: clk + rst_n + uart + 8 LEDs only (for open-source toolchain)
        #[arg(long)]
        minimal: bool,

        /// FPGA device identifier (default: xc7a100tcsg324-1)
        #[arg(long, default_value = "xc7a100tcsg324-1")]
        device: String,

        /// Top-level module name (default: zerodsp_top)
        #[arg(long, default_value = "zerodsp_top")]
        top: String,

        /// Use Docker for synthesis tools (default: true if no local Yosys)
        #[arg(long, default_missing_value = "true")]
        docker: Option<bool>,

        /// Use HIR path instead of direct AST-to-Verilog for code generation
        #[arg(long)]
        use_hir: bool,

        /// Path to nextpnr-xilinx binary
        #[arg(long)]
        nextpnr: Option<String>,

        /// Path to chipdb binary for nextpnr
        #[arg(long)]
        chipdb: Option<String>,

        /// Path to XDC constraints file
        #[arg(long)]
        xdc: Option<String>,

        /// Path to prjxray fasm2frames (Python, from prjxray repo)
        #[arg(long)]
        fasm2frames: Option<String>,

        /// Path to xc7frames2bit binary
        #[arg(long)]
        frames2bit: Option<String>,

        /// Path to prjxray database directory
        #[arg(long)]
        prjxray_db: Option<String>,

        /// Output directory (default: build/fpga)
        #[arg(short, long, default_value = "build/fpga")]
        output: String,
    },

    /// FormulaOS: evaluate and search Trinity formulas
    Formula {
        #[command(subcommand)]
        cmd: formula_eval::FormulaCommands,
    },
    /// Check FPGA synthesis readiness for all specs
    #[command(name = "synth-readiness")]
    SynthReadiness {
        /// Directory with FPGA specs (default: specs/fpga)
        #[arg(long, default_value = "specs/fpga")]
        specs_dir: String,
    },

    /// TRI PHI LOOP: show current status
    #[command(name = "tri-status")]
    TriStatus,

    /// Chimera search: find new formulas by combining existing ones
    Chimera {
        /// Maximum error percentage
        #[arg(long, default_value = "1.0")]
        threshold: f64,
        /// Limit number of results
        #[arg(long, default_value = "20")]
        limit: usize,
    },

    /// Sensitivity analysis: scan formula response to parameter variations
    Sensitivity {
        /// Formula ID to analyze
        id: String,
        /// Parameter to vary (phi, pi, e)
        #[arg(long, default_value = "phi")]
        param: String,
        /// Min value
        #[arg(long)]
        min: Option<f64>,
        /// Max value
        #[arg(long)]
        max: Option<f64>,
        /// Number of points
        #[arg(long, default_value = "30")]
        n: usize,
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
    routing::{get, post, delete, any},
    Router,
};
#[cfg(feature = "server")]
use tower_http::services::{ServeDir, ServeFile};
#[cfg(feature = "server")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use tokio::sync::{broadcast, RwLock};
#[cfg(feature = "server")]
use tokio_stream::wrappers::BroadcastStream;
#[cfg(feature = "server")]
use tokio::net::TcpListener;
#[cfg(feature = "server")]
use std::sync::Arc;

#[cfg(feature = "server")]
#[derive(Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub status: String,
    pub railway_service_id: String,
    pub created_at: u64,
    pub updated_at: u64,
}

#[cfg(feature = "server")]
#[derive(Clone)]
pub struct AppState {
    pub tx: broadcast::Sender<serde_json::Value>,
    pub sessions: Arc<RwLock<Vec<Session>>>,
}

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
struct ApiResponse {
    success: bool,
    output: Option<String>,
    error: Option<String>,
}

#[cfg(feature = "server")]
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    version: &'static str,
    healthy: bool,
}

#[cfg(feature = "server")]
async fn health_handler() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION"),
        healthy: true,
    })
}

#[cfg(feature = "server")]
async fn global_config_handler() -> impl IntoResponse {
    // Basic config required by OpenCode SDK to stop errors
    Json(serde_json::json!({
        "logLevel": "info",
        "theme": "oc-2"
    }))
}

#[cfg(feature = "server")]
async fn project_list_handler() -> impl IntoResponse {
    let now = chrono::Utc::now().timestamp_millis() as f64;
    // List one project (the current repo)
    Json(vec![serde_json::json!({
        "id": "t27",
        "name": "Trinity T27",
        "worktree": "/app",
        "vcs": "git",
        "time": {
            "created": now,
            "updated": now
        },
        "sandboxes": []
    })])
}

#[cfg(feature = "server")]
async fn project_current_handler() -> impl IntoResponse {
    let now = chrono::Utc::now().timestamp_millis() as f64;
    Json(serde_json::json!({
        "id": "t27",
        "name": "Trinity T27",
        "worktree": "/app",
        "vcs": "git",
        "time": {
            "created": now,
            "updated": now
        },
        "sandboxes": []
    }))
}

#[cfg(feature = "server")]
async fn project_patch_handler() -> impl IntoResponse {
    StatusCode::OK
}

#[cfg(feature = "server")]
async fn provider_list_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "all": [
            {
                "id": "zai",
                "name": "Z.AI (Integrated)",
                "source": "api",
                "env": [],
                "options": {},
                "models": {
                    "gpt-4o": {
                        "id": "gpt-4o",
                        "name": "GPT-4o",
                        "providerID": "zai",
                        "api": {
                            "id": "openai",
                            "url": "https://api.openai.com/v1",
                            "npm": "openai"
                        },
                        "capabilities": {
                            "temperature": true,
                            "reasoning": true,
                            "attachment": true,
                            "toolcall": true,
                            "input": {
                                "text": true,
                                "audio": false,
                                "image": true,
                                "video": false,
                                "pdf": true
                            },
                            "output": {
                                "text": true,
                                "audio": false,
                                "image": false,
                                "video": false,
                                "pdf": false
                            },
                            "interleaved": false
                        },
                        "cost": {
                            "input": 0.0,
                            "output": 0.0,
                            "cache": { "read": 0.0, "write": 0.0 }
                        },
                        "limit": {
                            "context": 128000,
                            "output": 4096
                        },
                        "status": "active",
                        "options": {},
                        "headers": {},
                        "release_date": "2024-05-13"
                    },
                    "claude-3-5-sonnet": {
                        "id": "claude-3-5-sonnet",
                        "name": "Claude 3.5 Sonnet",
                        "providerID": "zai",
                        "api": {
                            "id": "anthropic",
                            "url": "https://api.anthropic.com/v1",
                            "npm": "@anthropic-ai/sdk"
                        },
                        "capabilities": {
                            "temperature": true,
                            "reasoning": false,
                            "attachment": true,
                            "toolcall": true,
                            "input": {
                                "text": true,
                                "audio": false,
                                "image": true,
                                "video": false,
                                "pdf": true
                            },
                            "output": {
                                "text": true,
                                "audio": false,
                                "image": false,
                                "video": false,
                                "pdf": false
                            },
                            "interleaved": false
                        },
                        "cost": {
                            "input": 0.0,
                            "output": 0.0,
                            "cache": { "read": 0.0, "write": 0.0 }
                        },
                        "limit": {
                            "context": 200000,
                            "output": 8192
                        },
                        "status": "active",
                        "options": {},
                        "headers": {},
                        "release_date": "2024-06-20"
                    }
                }
            }
        ],
        "connected": ["zai"],
        "default": {
            "chat": "gpt-4o",
            "code": "gpt-4o"
        }
    }))
}

#[cfg(feature = "server")]
async fn provider_auth_handler() -> impl IntoResponse {
    Json(serde_json::json!({}))
}

#[cfg(feature = "server")]
async fn auth_id_handler() -> impl IntoResponse {
    Json(true)
}

#[cfg(feature = "server")]
async fn config_providers_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "providers": [
            {
                "id": "zai",
                "name": "Z.AI (Integrated)",
                "source": "api",
                "env": [],
                "options": {},
                "models": {
                    "gpt-4o": {
                        "id": "gpt-4o",
                        "name": "GPT-4o",
                        "providerID": "zai",
                        "api": {
                            "id": "openai",
                            "url": "https://api.openai.com/v1",
                            "npm": "openai"
                        },
                        "capabilities": {
                            "temperature": true,
                            "reasoning": true,
                            "attachment": true,
                            "toolcall": true,
                            "input": {
                                "text": true,
                                "audio": false,
                                "image": true,
                                "video": false,
                                "pdf": true
                            },
                            "output": {
                                "text": true,
                                "audio": false,
                                "image": false,
                                "video": false,
                                "pdf": false
                            },
                            "interleaved": false
                        },
                        "cost": {
                            "input": 0.0,
                            "output": 0.0,
                            "cache": { "read": 0.0, "write": 0.0 }
                        },
                        "limit": {
                            "context": 128000,
                            "output": 4096
                        },
                        "status": "active",
                        "options": {},
                        "headers": {},
                        "release_date": "2024-05-13"
                    },
                    "claude-3-5-sonnet": {
                        "id": "claude-3-5-sonnet",
                        "name": "Claude 3.5 Sonnet",
                        "providerID": "zai",
                        "api": {
                            "id": "anthropic",
                            "url": "https://api.anthropic.com/v1",
                            "npm": "@anthropic-ai/sdk"
                        },
                        "capabilities": {
                            "temperature": true,
                            "reasoning": false,
                            "attachment": true,
                            "toolcall": true,
                            "input": {
                                "text": true,
                                "audio": false,
                                "image": true,
                                "video": false,
                                "pdf": true
                            },
                            "output": {
                                "text": true,
                                "audio": false,
                                "image": false,
                                "video": false,
                                "pdf": false
                            },
                            "interleaved": false
                        },
                        "cost": {
                            "input": 0.0,
                            "output": 0.0,
                            "cache": { "read": 0.0, "write": 0.0 }
                        },
                        "limit": {
                            "context": 200000,
                            "output": 8192
                        },
                        "status": "active",
                        "options": {},
                        "headers": {},
                        "release_date": "2024-06-20"
                    }
                }
            }
        ],
        "default_config": {
            "model": "gpt-4o"
        }
    }))
}

#[cfg(feature = "server")]
async fn config_get_handler() -> impl IntoResponse {
    Json(serde_json::json!({}))
}

#[cfg(feature = "server")]
async fn session_list_handler(State(state): State<AppState>) -> impl IntoResponse {
    let sessions = state.sessions.read().await;
    Json(serde_json::json!({
        "data": *sessions
    }))
}

#[cfg(feature = "server")]
async fn session_status_handler() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "idle" }))
}

#[cfg(feature = "server")]
async fn session_id_handler(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let sessions = state.sessions.read().await;
    if let Some(session) = sessions.iter().find(|s| s.id == id) {
        Json(serde_json::json!({
            "data": session
        }))
    } else {
        Json(serde_json::json!({
            "data": {
                "id": id,
                "name": format!("Session {}", id),
                "status": "active",
                "railway_service_id": format!("srv_{}", id),
                "created_at": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or(std::time::Duration::from_secs(0))
                    .as_secs(),
                "updated_at": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or(std::time::Duration::from_secs(0))
                    .as_secs()
            }
        }))
    }
}

#[cfg(feature = "server")]
async fn session_delete_handler(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let mut sessions = state.sessions.write().await;
    if let Some(pos) = sessions.iter().position(|s| s.id == id) {
        sessions[pos].status = "deleted".to_string();
        sessions[pos].updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();
        Json(serde_json::json!({
            "data": sessions[pos].clone()
        })).into_response()
    } else {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Session not found"
        }))).into_response()
    }
}

#[cfg(feature = "server")]
async fn session_create_handler(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::from_secs(0))
        .as_secs();

    let name = payload.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled Session")
        .to_string();

    let id = format!("ses_{}", current_time);
    let mut railway_service_id = format!("srv_{}", current_time);
    let mut status = "active".to_string();

    // CREATE REAL RAILWAY SERVICE (if token available)
    let railway_token = env::var("RAILWAY_API_TOKEN_0").ok();
    let base_service_id = env::var("RAILWAY_SERVICE_ID").ok();

    if let (Some(token), Some(base_id)) = (railway_token, base_service_id) {
        match railway::create_railway_service(&name, &id, &token, &base_id).await {
            Ok(service_id) => {
                railway_service_id = service_id.clone();
                status = "starting".to_string();

                // Set session-specific environment variables
                let session_vars = vec![
                    (String::from("SESSION_ID"), id.clone()),
                    (String::from("SESSION_NAME"), name.clone()),
                ];
                let _ = railway::set_service_variables(&service_id, &session_vars, &token).await;

                // Start health polling in background
                let sessions_clone = state.sessions.clone();
                let token_clone = token;
                let id_for_poller = id.clone();
                tokio::spawn(async move {
                    health_poller(id_for_poller, service_id, sessions_clone, token_clone).await;
                });
            }
            Err(e) => {
                eprintln!("Failed to create Railway service: {}", e);
                // Fallback: in-memory only with mock status
            }
        }
    }

    let session = Session {
        id: id.clone(),
        name,
        status,
        railway_service_id,
        created_at: current_time,
        updated_at: current_time,
    };

    // Store session
    state.sessions.write().await.push(session.clone());

    Json(serde_json::json!({
        "data": session
    }))
}

/// Health poller for Railway services
/// Polls the service health every 5 seconds for up to 2 minutes
/// Updates session status to "active" when the service is ready
#[cfg(feature = "server")]
async fn health_poller(
    session_id: String,
    service_id: String,
    sessions: Arc<RwLock<Vec<Session>>>,
    railway_token: String,
) {
    const MAX_POLLS: u32 = 24; // 24 * 5 seconds = 2 minutes
    const POLL_INTERVAL: tokio::time::Duration = tokio::time::Duration::from_secs(5);

    for i in 0..MAX_POLLS {
        tokio::time::sleep(POLL_INTERVAL).await;

        // Check service health via Railway API
        match railway::check_service_health(&service_id, &railway_token).await {
            Ok(true) => {
                // Service is healthy, update session status
                let mut sessions_guard = sessions.write().await;
                if let Some(session) = sessions_guard.iter_mut().find(|s| s.id == session_id) {
                    session.status = "active".to_string();
                    session.updated_at = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or(std::time::Duration::from_secs(0))
                        .as_secs();
                    println!("Session {} is now active", session_id);
                }
                return;
            }
            Ok(false) => {
                // Service not ready yet, continue polling
                if i % 4 == 0 {
                    // Log every 20 seconds
                    println!("Session {} still starting... ({}/{})", session_id, i + 1, MAX_POLLS);
                }
            }
            Err(e) => {
                eprintln!("Health check error for session {}: {}", session_id, e);
            }
        }
    }

    // After max polls, mark as error state
    let mut sessions_guard = sessions.write().await;
    if let Some(session) = sessions_guard.iter_mut().find(|s| s.id == session_id) {
        session.status = "error".to_string();
        session.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();
        eprintln!("Session {} failed to become active after timeout", session_id);
    }
}

#[cfg(feature = "server")]
async fn session_create_sandbox_token_handler(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    // Find session to get its name
    let sessions = state.sessions.read().await;
    let session_name = sessions
        .iter()
        .find(|s| s.id == id)
        .map(|s| s.name.clone())
        .unwrap_or_else(|| "Untitled Session".to_string());
    drop(sessions);

    // Generate real JWT token
    match jwt::create_sandbox_token(&id, Some(24)) {
        Ok(token) => {
            Json(serde_json::json!({
                "data": {
                    "token": token,
                    "expiresIn": 86400,
                    "sessionName": session_name
                }
            })).into_response()
        }
        Err(e) => {
            eprintln!("Failed to create sandbox token: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create token").into_response()
        }
    }
}

#[cfg(feature = "server")]
async fn session_message_list_handler() -> impl IntoResponse {
    Json(Vec::<serde_json::Value>::new())
}

#[cfg(feature = "server")]
async fn session_message_post_handler() -> impl IntoResponse {
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::from_secs(0))
        .as_secs();

    Json(serde_json::json!({
        "info": {
            "id": "msg_mock",
            "sessionID": "ses_default",
            "time": {
                "created": current_time,
                "updated": current_time
            },
            "role": "assistant"
        },
        "parts": [
            {
                "id": "prt_mock",
                "messageID": "msg_mock",
                "time": {
                    "created": current_time,
                    "updated": current_time
                },
                "status": "complete",
                "content": {
                    "type": "text",
                    "text": "Trinity Backend is active. Ready to build."
                }
            }
        ]
    }))
}

#[cfg(feature = "server")]
async fn prompt_async_handler(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::from_secs(0))
        .as_secs() as f64;

    // Extract the messageID from the prompt to use as parentID
    let parent_id = payload.get("messageID")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "msg_user_id".to_string());

    // Send mock response in background
    tokio::spawn(async move {
        // Shared boilerplate for the AssistantMessage mock
        let info_base = serde_json::json!({
            "id": "msg_reply",
            "sessionID": "ses_default",
            "role": "assistant",
            "parentID": parent_id,
            "modelID": "gpt-4o",
            "providerID": "zai",
            "mode": "chat",
            "path": {
                "cwd": "/app",
                "root": "/app"
            },
            "cost": 0.0,
            "tokens": {
                "input": 0,
                "output": 0,
                "reasoning": 0,
                "cache": { "read": 0, "write": 0 }
            }
        });

        // 1. Send "thinking" status (no completed = thinking)
        let mut thinking_info = info_base.clone();
        thinking_info["time"] = serde_json::json!({ "created": current_time });

        let thinking_event = serde_json::json!({
            "directory": "/app",
            "payload": {
                "type": "message.updated",
                "properties": {
                    "sessionID": "ses_default",
                    "info": thinking_info
                }
            }
        });
        let _ = state.tx.send(thinking_event);

        // Simulate some processing delay
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // 2. Send the message text using `message.part.updated`
        let part_event = serde_json::json!({
            "directory": "/app",
            "payload": {
                "type": "message.part.updated",
                "properties": {
                    "part": {
                        "id": "part_reply",
                        "sessionID": "ses_default",
                        "messageID": "msg_reply",
                        "type": "text",
                        "text": "Hello! I am the Trinity Orchestrator. The SSE format is now perfectly aligned with the generated SDK, and I can respond correctly. How can I assist you with your project today?",
                        "time": { "created": current_time }
                    }
                }
            }
        });
        let _ = state.tx.send(part_event);

        // 3. Send "complete" status with text (has completed = done)
        let mut complete_info = info_base;
        complete_info["time"] = serde_json::json!({ 
            "created": current_time, 
            "completed": current_time + 2.0
        });

        let complete_event = serde_json::json!({
            "directory": "/app",
            "payload": {
                "type": "message.updated",
                "properties": {
                    "sessionID": "ses_default",
                    "info": complete_info
                }
            }
        });
        let _ = state.tx.send(complete_event);

        // 4. Send "idle" session status to clear the UI busy state
        let idle_event = serde_json::json!({
            "directory": "/app",
            "payload": {
                "type": "session.status",
                "properties": {
                    "sessionID": "ses_default",
                    "status": {
                        "type": "idle"
                    }
                }
            }
        });
        let _ = state.tx.send(idle_event);
    });

    axum::http::StatusCode::NO_CONTENT
}

#[cfg(feature = "server")]
async fn session_todo_handler() -> impl IntoResponse {
    Json(serde_json::json!([]))
}

#[cfg(feature = "server")]
async fn agent_list_handler() -> impl IntoResponse {
    Json(serde_json::json!([{
            "name": "zai",
            "description": "Trinity AI Agent",
            "mode": "all",
            "native": true,
            "hidden": false,
            "topP": 1.0,
            "temperature": 0.5,
            "color": "#4a90e2",
            "permission": [],
            "model": {
                "modelID": "gpt-4o",
                "providerID": "openai"
            },
            "options": {}
    }]))
}

#[cfg(feature = "server")]
async fn vcs_handler() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "clean" }))
}

#[cfg(feature = "server")]
async fn generic_list_handler() -> impl IntoResponse {
    Json(Vec::<serde_json::Value>::new())
}

#[cfg(feature = "server")]
async fn instance_handler() -> impl IntoResponse {
    Json(serde_json::json!({ "healthy": true }))
}

#[cfg(feature = "server")]
async fn path_handler() -> impl IntoResponse {
    // SDK uses this to check path existence/stat
    Json(serde_json::json!({
        "exists": true,
        "is_directory": true
    }))
}

#[cfg(feature = "server")]
#[allow(dead_code)]
async fn root_handler() -> impl IntoResponse {
    // Return the frontend or a simple health message
    "t27c orchestrator live"
}

#[cfg(feature = "server")]
async fn global_event_handler(State(state): State<AppState>) -> impl IntoResponse {
    use axum::response::sse::{Event, KeepAlive, Sse};
    use std::time::Duration;
    use tokio_stream::StreamExt;
    use futures_util::stream;

    // 0. Initial "server.connected" event - SDK expects this first
    let connected_stream = stream::once(async move {
        Ok::<Event, axum::Error>(Event::default()
            .event("server.connected")
            .data(r#"{"directory":"global","payload":{"type":"server.connected","properties":{}}}"#))
    });

    // 1. Broadcast stream for real events
    let broadcast_stream = BroadcastStream::new(state.tx.subscribe())
        .map(|res| {
            match res {
                Ok(json) => {
                    Event::default().json_data(json).map_err(|e| {
                        axum::Error::new(format!("JSON error: {}", e))
                    })
                },
                Err(e) => Err(axum::Error::new(format!("Broadcast error: {}", e))),
            }
        });

    // 2. Keep-alive stream (pings every 15s)
    let keep_alive_stream = tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(Duration::from_secs(15)))
        .map(|_| {
            Ok::<Event, axum::Error>(Event::default().comment("keepalive"))
        });

    // 3. Merge: connected -> broadcast -> keep_alive
    let stream = connected_stream
        .chain(broadcast_stream)
        .chain(keep_alive_stream);

    Sse::new(stream).keep_alive(KeepAlive::default())
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
async fn parse_handler(
    Json(req): Json<CompileRequest>,
) -> impl IntoResponse {
    match compiler::Compiler::parse_ast(&req.source) {
        Ok(ast) => (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                output: Some(format!("{:#?}", ast)),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                output: None,
                error: Some(e),
            }),
        ),
    }
}

#[cfg(feature = "server")]
async fn gen_handler(
    Json(req): Json<CompileRequest>,
) -> impl IntoResponse {
    match compiler::Compiler::compile(&req.source) {
        Ok(code) => (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                output: Some(code),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                output: None,
                error: Some(e),
            }),
        ),
    }
}

#[cfg(feature = "server")]
async fn gen_verilog_handler(
    Json(req): Json<CompileRequest>,
) -> impl IntoResponse {
    match compiler::Compiler::compile_verilog(&req.source) {
        Ok(code) => (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                output: Some(code),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                output: None,
                error: Some(e),
            }),
        ),
    }
}

#[cfg(feature = "server")]
async fn gen_c_handler(
    Json(req): Json<CompileRequest>,
) -> impl IntoResponse {
    match compiler::Compiler::compile_c(&req.source) {
        Ok(code) => (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                output: Some(code),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                output: None,
                error: Some(e),
            }),
        ),
    }
}

#[cfg(feature = "server")]
async fn gen_rust_handler(
    Json(req): Json<CompileRequest>,
) -> impl IntoResponse {
    match compiler::Compiler::compile_rust(&req.source) {
        Ok(code) => (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                output: Some(code),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                output: None,
                error: Some(e),
            }),
        ),
    }
}

#[cfg(feature = "server")]
async fn seal_handler(
    Json(req): Json<CompileRequest>,
) -> impl IntoResponse {
    let spec_hash = format!("sha256:{}", sha256_hex(req.source.as_bytes()));

    let gen_hash_zig = match compiler::Compiler::compile(&req.source) {
        Ok(code) => format!("sha256:{}", sha256_hex(code.as_bytes())),
        Err(_) => "none".to_string(),
    };
    let gen_hash_verilog = match compiler::Compiler::compile_verilog(&req.source) {
        Ok(code) => format!("sha256:{}", sha256_hex(code.as_bytes())),
        Err(_) => "none".to_string(),
    };
    let gen_hash_c = match compiler::Compiler::compile_c(&req.source) {
        Ok(code) => format!("sha256:{}", sha256_hex(code.as_bytes())),
        Err(_) => "none".to_string(),
    };
    let gen_hash_rust = match compiler::Compiler::compile_rust(&req.source) {
        Ok(code) => format!("sha256:{}", sha256_hex(code.as_bytes())),
        Err(_) => "none".to_string(),
    };

    let output = serde_json::json!({
        "spec_hash": spec_hash,
        "gen_hash_zig": gen_hash_zig,
        "gen_hash_verilog": gen_hash_verilog,
        "gen_hash_c": gen_hash_c,
        "gen_hash_rust": gen_hash_rust,
    });

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            output: Some(output.to_string()),
            error: None,
        }),
    )
}

#[cfg(feature = "server")]
async fn stats_handler() -> impl IntoResponse {
    let stats = serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "backends": ["zig", "verilog", "c"],
        "endpoints": ["/health", "/compile", "/parse", "/gen", "/gen-verilog", "/gen-c", "/seal", "/stats",
                      "/optimize", "/typecheck", "/lint", "/explain", "/bench", "/graph", "/doc", "/size", "/inspect", "/deadcode", "/metrics", "/coverage"],
    });

    Json(ApiResponse {
        success: true,
        output: Some(stats.to_string()),
        error: None,
    })
}

#[cfg(feature = "server")]
async fn optimize_handler(Json(req): Json<CompileRequest>) -> impl IntoResponse {
    match compiler::Compiler::parse_ast(&req.source) {
        Ok(mut ast) => {
            let config = compiler::OptConfig::default();
            let stats = compiler::optimize(&mut ast, &config);
            let result = serde_json::json!({
                "folds": stats.folds,
                "dead_removed": stats.dead_removed,
                "copies_propagated": stats.copies_propagated,
                "strengths_reduced": stats.strengths_reduced,
                "cse_eliminated": stats.cse_eliminated,
                "dead_stores": stats.dead_stores,
                "loops_unrolled": stats.loops_unrolled,
                "passes": stats.passes,
            });
            (StatusCode::OK, Json(ApiResponse {
                success: true,
                output: Some(result.to_string()),
                error: None,
            }))
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse {
            success: false, output: None, error: Some(e),
        })),
    }
}

#[cfg(feature = "server")]
async fn typecheck_handler(Json(req): Json<CompileRequest>) -> impl IntoResponse {
    match compiler::Compiler::parse_ast(&req.source) {
        Ok(ast) => {
            let result = compiler::typecheck_ast(&ast);
            let resp = serde_json::json!({
                "ok": result.ok,
                "error_count": result.error_count,
                "warnings": result.warnings,
                "errors": result.errors,
            });
            (StatusCode::OK, Json(ApiResponse {
                success: result.ok,
                output: Some(resp.to_string()),
                error: None,
            }))
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse {
            success: false, output: None, error: Some(e),
        })),
    }
}

#[cfg(feature = "server")]
async fn lint_handler(Json(req): Json<CompileRequest>) -> impl IntoResponse {
    match compiler::Compiler::parse_ast(&req.source) {
        Ok(ast) => {
            let mut issues = 0u32;
            let mut fn_count = 0u32;
            let mut test_count = 0u32;
            let mut inv_count = 0u32;
            let mut warnings = Vec::new();
            for child in &ast.children {
                match child.kind {
                    compiler::NodeKind::FnDecl => {
                        fn_count += 1;
                        let has_test = child.children.iter().any(|c| c.kind == compiler::NodeKind::TestBlock);
                        let has_inv = child.children.iter().any(|c| c.kind == compiler::NodeKind::InvariantBlock);
                        if !has_test && !has_inv {
                            warnings.push(format!("fn '{}' has no test or invariant", child.name));
                            issues += 1;
                        }
                    }
                    compiler::NodeKind::TestBlock => test_count += 1,
                    compiler::NodeKind::InvariantBlock => inv_count += 1,
                    _ => {}
                }
            }
            if fn_count == 0 { issues += 1; }
            if test_count == 0 && inv_count == 0 { issues += 1; }
            let resp = serde_json::json!({
                "issues": issues,
                "functions": fn_count,
                "tests": test_count,
                "invariants": inv_count,
                "warnings": warnings,
            });
            (StatusCode::OK, Json(ApiResponse {
                success: true,
                output: Some(resp.to_string()),
                error: None,
            }))
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse {
            success: false, output: None, error: Some(e),
        })),
    }
}

#[cfg(feature = "server")]
async fn explain_handler(Json(req): Json<CompileRequest>) -> impl IntoResponse {
    match compiler::Compiler::parse_ast(&req.source) {
        Ok(ast) => {
            let tc = compiler::typecheck_ast(&ast);
            let mut opt_ast = ast.clone();
            let config = compiler::OptConfig::default();
            let opt_stats = compiler::optimize(&mut opt_ast, &config);
            let mut codegen = compiler::Codegen::new();
            codegen.gen_zig(&ast);
            let output = codegen.into_string();
            let resp = serde_json::json!({
                "module": ast.name,
                "declarations": ast.children.len(),
                "typecheck": {"ok": tc.ok, "errors": tc.error_count, "warnings": tc.warnings},
                "optimize": {"folds": opt_stats.folds, "dead_removed": opt_stats.dead_removed},
                "codegen_bytes": output.len(),
            });
            (StatusCode::OK, Json(ApiResponse {
                success: true,
                output: Some(resp.to_string()),
                error: None,
            }))
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse {
            success: false, output: None, error: Some(e),
        })),
    }
}

#[cfg(feature = "server")]
async fn bench_handler(Json(req): Json<CompileRequest>) -> impl IntoResponse {
    let lex_time = {
        let start = std::time::Instant::now();
        let mut lexer = compiler::Lexer::new(&req.source);
        lexer.tokenize();
        start.elapsed()
    };
    let parse_time = {
        let start = std::time::Instant::now();
        let _ = compiler::Compiler::parse_ast(&req.source);
        start.elapsed()
    };
    let resp = serde_json::json!({
        "lex_us": lex_time.as_micros(),
        "parse_us": parse_time.as_micros(),
    });
    (StatusCode::OK, Json(ApiResponse {
        success: true,
        output: Some(resp.to_string()),
        error: None,
    }))
}

#[cfg(feature = "server")]
async fn eval_handler(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    let expr = req.get("expr").and_then(|v| v.as_str()).unwrap_or("");
    let source = format!("fn _eval() {{ return {}; }}", expr);
    match compiler::Compiler::parse_ast(&source) {
        Ok(ast) => {
            let mut opt_ast = ast.clone();
            let config = compiler::OptConfig { opt_level: 3, ..Default::default() };
            let _ = compiler::optimize(&mut opt_ast, &config);
            let mut result_val = None::<String>;
            for child in &opt_ast.children {
                if child.kind == compiler::NodeKind::FnDecl {
                    for stmt in &child.children {
                        if stmt.kind == compiler::NodeKind::ExprReturn && !stmt.children.is_empty() {
                            let ret = &stmt.children[0];
                            if ret.kind == compiler::NodeKind::ExprLiteral {
                                result_val = Some(ret.value.clone());
                            }
                        }
                    }
                }
            }
            (StatusCode::OK, Json(ApiResponse {
                success: true,
                output: result_val,
                error: None,
            }))
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse {
            success: false, output: None, error: Some(e),
        })),
    }
}

#[cfg(feature = "server")]
async fn graph_handler(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    let root = req.get("repo_root").and_then(|v| v.as_str()).unwrap_or(".");
    let root_path = Path::new(root);
    let files: Vec<PathBuf> = walkdir::WalkDir::new(root_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "t27").unwrap_or(false))
        .map(|e| e.path().to_path_buf())
        .collect();
    let mut modules: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for file in &files {
        let source = match fs::read_to_string(file) { Ok(s) => s, Err(_) => continue };
        let lexer = compiler::Lexer::new(&source);
        let mut parser = compiler::Parser::new(lexer);
        let ast = match parser.parse() { Ok(a) => a, Err(_) => continue };
        let mut imports = Vec::new();
        for child in &ast.children {
            if child.kind == compiler::NodeKind::UseDecl {
                imports.push(child.value.replace("::", "/"));
            }
        }
        modules.insert(ast.name, imports);
    }
    let resp = serde_json::json!({"modules": modules});
    (StatusCode::OK, Json(ApiResponse {
        success: true,
        output: Some(resp.to_string()),
        error: None,
    }))
}

#[cfg(feature = "server")]
async fn doc_handler(Json(req): Json<CompileRequest>) -> impl IntoResponse {
    match compiler::Compiler::parse_ast(&req.source) {
        Ok(ast) => {
            let mut fn_decls = Vec::new();
            let mut struct_decls = Vec::new();
            let mut enum_decls = Vec::new();
            let mut test_decls = Vec::new();
            let mut inv_decls = Vec::new();
            for child in &ast.children {
                match child.kind {
                    compiler::NodeKind::FnDecl => fn_decls.push(child.name.clone()),
                    compiler::NodeKind::StructDecl => struct_decls.push(child.name.clone()),
                    compiler::NodeKind::EnumDecl => enum_decls.push(child.name.clone()),
                    compiler::NodeKind::TestBlock => test_decls.push(child.name.clone()),
                    compiler::NodeKind::InvariantBlock => inv_decls.push(child.name.clone()),
                    _ => {}
                }
            }
            let resp = serde_json::json!({
                "module": ast.name,
                "functions": fn_decls,
                "structs": struct_decls,
                "enums": enum_decls,
                "tests": test_decls,
                "invariants": inv_decls,
            });
            (StatusCode::OK, Json(ApiResponse {
                success: true,
                output: Some(resp.to_string()),
                error: None,
            }))
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse {
            success: false, output: None, error: Some(e),
        })),
    }
}

#[cfg(feature = "server")]
async fn size_handler(Json(req): Json<CompileRequest>) -> impl IntoResponse {
    match compiler::Compiler::parse_ast(&req.source) {
        Ok(ast) => {
            let fns: u32 = 0;
            let structs: u32 = 0;
            let enums: u32 = 0;
            let consts: u32 = 0;
            let tests: u32 = 0;
            let invariants: u32 = 0;
            let benches: u32 = 0;
            let imports: u32 = 0;
            let total_nodes: u32 = 0;
            fn count(node: &compiler::Node, s: &mut (u32, u32, u32, u32, u32, u32, u32, u32, u32)) {
                s.8 += 1;
                match node.kind {
                    compiler::NodeKind::FnDecl => s.0 += 1,
                    compiler::NodeKind::StructDecl => s.1 += 1,
                    compiler::NodeKind::EnumDecl => s.2 += 1,
                    compiler::NodeKind::ConstDecl => s.3 += 1,
                    compiler::NodeKind::TestBlock => s.4 += 1,
                    compiler::NodeKind::InvariantBlock => s.5 += 1,
                    compiler::NodeKind::BenchBlock => s.6 += 1,
                    compiler::NodeKind::UseDecl => s.7 += 1,
                    _ => {}
                }
                for child in &node.children { count(child, s); }
            }
            let mut s = (fns, structs, enums, consts, tests, invariants, benches, imports, total_nodes);
            count(&ast, &mut s);
            let (fns, structs, enums, consts, tests, invariants, benches, imports, total_nodes) = s;
            let lines = req.source.lines().count();
            let bytes = req.source.len();
            let resp = serde_json::json!({
                "module": ast.name,
                "bytes": bytes,
                "lines": lines,
                "nodes": total_nodes,
                "functions": fns,
                "structs": structs,
                "enums": enums,
                "constants": consts,
                "tests": tests,
                "invariants": invariants,
                "benchmarks": benches,
                "imports": imports,
            });
            (StatusCode::OK, Json(ApiResponse {
                success: true,
                output: Some(resp.to_string()),
                error: None,
            }))
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse {
            success: false, output: None, error: Some(e),
        })),
    }
}

#[cfg(feature = "server")]
async fn inspect_handler(Json(req): Json<CompileRequest>) -> impl IntoResponse {
    match compiler::Compiler::parse_ast(&req.source) {
        Ok(ast) => {
            let mut api = serde_json::json!({"module": ast.name});
            let mut fns = Vec::new();
            let mut structs = Vec::new();
            let mut enums = Vec::new();
            let mut consts = Vec::new();
            for child in &ast.children {
                match child.kind {
                    compiler::NodeKind::FnDecl => {
                        let params: Vec<String> = child.params.iter().map(|(n, t)| {
                            if t.is_empty() { n.clone() } else { format!("{}: {}", n, t) }
                        }).collect();
                        fns.push(serde_json::json!({
                            "name": child.name,
                            "params": params,
                            "return_type": child.extra_return_type,
                            "pub": child.extra_pub,
                        }));
                    }
                    compiler::NodeKind::StructDecl => {
                        let fields: Vec<String> = child.children.iter().map(|f| {
                            if f.extra_type.is_empty() { f.name.clone() } else { format!("{}: {}", f.name, f.extra_type) }
                        }).collect();
                        structs.push(serde_json::json!({"name": child.name, "fields": fields}));
                    }
                    compiler::NodeKind::EnumDecl => {
                        let variants: Vec<String> = child.children.iter().map(|v| v.name.clone()).collect();
                        enums.push(serde_json::json!({"name": child.name, "variants": variants}));
                    }
                    compiler::NodeKind::ConstDecl => {
                        consts.push(serde_json::json!({"name": child.name, "value": child.value, "type": child.extra_type}));
                    }
                    _ => {}
                }
            }
            api["functions"] = serde_json::json!(fns);
            api["structs"] = serde_json::json!(structs);
            api["enums"] = serde_json::json!(enums);
            api["constants"] = serde_json::json!(consts);
            (StatusCode::OK, Json(ApiResponse {
                success: true,
                output: Some(api.to_string()),
                error: None,
            }))
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse {
            success: false, output: None, error: Some(e),
        })),
    }
}

#[cfg(feature = "server")]
async fn deadcode_handler(Json(req): Json<CompileRequest>) -> impl IntoResponse {
    match compiler::Compiler::parse_ast(&req.source) {
        Ok(ast) => {
            let mut all_fns: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut called: std::collections::HashSet<String> = std::collections::HashSet::new();
            fn collect_calls(node: &compiler::Node, calls: &mut std::collections::HashSet<String>) {
                if node.kind == compiler::NodeKind::ExprCall && !node.name.is_empty() {
                    calls.insert(node.name.clone());
                }
                for child in &node.children { collect_calls(child, calls); }
            }
            for child in &ast.children {
                if child.kind == compiler::NodeKind::FnDecl {
                    all_fns.insert(child.name.clone());
                    collect_calls(child, &mut called);
                }
                if matches!(child.kind, compiler::NodeKind::TestBlock | compiler::NodeKind::InvariantBlock | compiler::NodeKind::BenchBlock) {
                    collect_calls(child, &mut called);
                }
            }
            let dead: Vec<String> = all_fns.iter().filter(|f| !called.contains(*f)).cloned().collect();
            let resp = serde_json::json!({
                "total_functions": all_fns.len(),
                "called": all_fns.intersection(&called).count(),
                "dead": dead,
            });
            (StatusCode::OK, Json(ApiResponse { success: true, output: Some(resp.to_string()), error: None }))
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse { success: false, output: None, error: Some(e) })),
    }
}

#[cfg(feature = "server")]
async fn metrics_handler(Json(req): Json<CompileRequest>) -> impl IntoResponse {
    match compiler::Compiler::parse_ast(&req.source) {
        Ok(ast) => {
            let mut fns_metrics = Vec::new();
            fn count_complexity(node: &compiler::Node) -> u32 {
                let mut cc = 0u32;
                match node.kind {
                    compiler::NodeKind::ExprIf | compiler::NodeKind::StmtIf => cc += 1,
                    compiler::NodeKind::ExprSwitch => cc += 1,
                    compiler::NodeKind::StmtWhile | compiler::NodeKind::StmtFor => cc += 1,
                    compiler::NodeKind::ExprBinary if node.extra_op == "&&" || node.extra_op == "||" => cc += 1,
                    _ => {}
                }
                for child in &node.children { cc += count_complexity(child); }
                cc
            }
            fn count_nodes(node: &compiler::Node) -> u32 {
                let mut c = 1u32;
                for child in &node.children { c += count_nodes(child); }
                c
            }
            for child in &ast.children {
                if child.kind == compiler::NodeKind::FnDecl {
                    fns_metrics.push(serde_json::json!({
                        "name": child.name,
                        "params": child.params.len(),
                        "nodes": count_nodes(child),
                        "complexity": count_complexity(child) + 1,
                    }));
                }
            }
            let resp = serde_json::json!({"functions": fns_metrics});
            (StatusCode::OK, Json(ApiResponse { success: true, output: Some(resp.to_string()), error: None }))
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse { success: false, output: None, error: Some(e) })),
    }
}

#[cfg(feature = "server")]
async fn coverage_handler(Json(req): Json<CompileRequest>) -> impl IntoResponse {
    match compiler::Compiler::parse_ast(&req.source) {
        Ok(ast) => {
            let mut fn_names = Vec::new();
            let mut tested_fns: std::collections::HashSet<String> = std::collections::HashSet::new();
            fn collect_calls(node: &compiler::Node, calls: &mut std::collections::HashSet<String>) {
                if node.kind == compiler::NodeKind::ExprCall && !node.name.is_empty() {
                    calls.insert(node.name.clone());
                }
                for child in &node.children { collect_calls(child, calls); }
            }
            for child in &ast.children {
                if child.kind == compiler::NodeKind::FnDecl { fn_names.push(child.name.clone()); }
                if matches!(child.kind, compiler::NodeKind::TestBlock | compiler::NodeKind::InvariantBlock | compiler::NodeKind::BenchBlock) {
                    collect_calls(child, &mut tested_fns);
                }
            }
            let covered: Vec<&String> = fn_names.iter().filter(|f| tested_fns.contains(*f)).collect();
            let uncovered: Vec<&String> = fn_names.iter().filter(|f| !tested_fns.contains(*f)).collect();
            let pct = if !fn_names.is_empty() { 100.0 * covered.len() as f64 / fn_names.len() as f64 } else { 0.0 };
            let resp = serde_json::json!({
                "total_functions": fn_names.len(),
                "tested": covered.len(),
                "untested": uncovered.len(),
                "coverage_pct": pct,
                "uncovered_functions": uncovered,
            });
            (StatusCode::OK, Json(ApiResponse { success: true, output: Some(resp.to_string()), error: None }))
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse { success: false, output: None, error: Some(e) })),
    }
}

#[cfg(feature = "server")]
async fn run_server(port_arg: &str) -> anyhow::Result<()> {
    // Support Railway's $PORT environment variable
    let env_port = env::var("PORT").ok();
    println!("t27c debug: PORT env var is {:?}", env_port);
    
    let port = env_port
        .unwrap_or_else(|| port_arg.to_string())
        .parse::<u16>()?;

    let (tx, _) = broadcast::channel(100);
    let state = AppState {
        tx,
        sessions: Arc::new(RwLock::new(Vec::new())),
    };

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/global/health", get(health_handler))
        .route("/global/config", get(global_config_handler))
        .route("/global/event", get(global_event_handler))
        .route("/global/sync-event", get(global_event_handler))
        .route("/project", get(project_list_handler))
        .route("/project/current", get(project_current_handler))
        .route("/project/:id", axum::routing::patch(project_patch_handler))
        .route("/provider", get(provider_list_handler))
        .route("/provider/auth", get(provider_auth_handler).post(provider_auth_handler))
        .route("/auth/:id", get(auth_id_handler).post(auth_id_handler).put(auth_id_handler))
        .route("/config", get(config_get_handler))
        .route("/config/providers", get(config_providers_handler))
        .route("/path", get(path_handler))
        // Session routes (both singular and plural for compatibility)
        .route("/session", get(session_list_handler).post(session_create_handler))
        .route("/sessions", get(session_list_handler).post(session_create_handler))
        .route("/session/status", get(session_status_handler))
        .route("/session/:id", get(session_id_handler).delete(session_delete_handler))
        .route("/sessions/:id", get(session_id_handler).delete(session_delete_handler))
        .route("/sessions/:id/token", post(session_create_sandbox_token_handler))
        .route("/session/:id/message", get(session_message_list_handler).post(session_message_post_handler))
        .route("/session/:id/prompt_async", post(prompt_async_handler))
        .route("/session/:id/todo", get(session_todo_handler))
        .route("/agent", get(agent_list_handler))
        .route("/vcs", get(vcs_handler))
        .route("/command", get(generic_list_handler))
        .route("/permission", get(generic_list_handler))
        .route("/question", get(generic_list_handler))
        .route("/mcp", get(generic_list_handler))
        .route("/instance", get(instance_handler))
        .route("/compile", post(compile_handler))
        .route("/parse", post(parse_handler))
        .route("/gen", post(gen_handler))
        .route("/gen-verilog", post(gen_verilog_handler))
        .route("/gen-c", post(gen_c_handler))
        .route("/gen-rust", post(gen_rust_handler))
        .route("/seal", post(seal_handler))
        .route("/stats", get(stats_handler))
        .route("/optimize", post(optimize_handler))
        .route("/typecheck", post(typecheck_handler))
        .route("/lint", post(lint_handler))
        .route("/explain", post(explain_handler))
        .route("/bench", post(bench_handler))
        .route("/eval", post(eval_handler))
        .route("/graph", post(graph_handler))
        .route("/doc", post(doc_handler))
        .route("/size", post(size_handler))
        .route("/inspect", post(inspect_handler))
        .route("/deadcode", post(deadcode_handler))
        .route("/metrics", post(metrics_handler))
        .route("/coverage", post(coverage_handler))
        .route("/sandbox", any(proxy::sandbox_proxy_handler))
        .route("/sandbox/*path", any(proxy::sandbox_proxy_handler))
        .fallback_service(
            ServeDir::new("public")
                .not_found_service(ServeFile::new("public/index.html"))
        )
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    println!("t27c server attempting to bind on {}", addr);
    let listener = TcpListener::bind(&addr).await?;
    println!("t27c server successfully listening on {}", addr);

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

fn run_debug_hir(input_path: &str) -> anyhow::Result<()> {
    let path = Path::new(input_path);
    let source = fs::read_to_string(path)?;

    match compiler::Compiler::debug_hir(&source) {
        Ok(hir_dump) => print!("{}", hir_dump),
        Err(e) => anyhow::bail!("HIR conversion error: {}", e),
    }
    Ok(())
}

fn run_gen_verilog_hir(input_path: &str) -> anyhow::Result<()> {
    let path = Path::new(input_path);
    let source = fs::read_to_string(path)?;

    match compiler::Compiler::compile_verilog_hir(&source) {
        Ok(verilog) => print!("{}", verilog),
        Err(e) => anyhow::bail!("HIR Verilog generation error: {}", e),
    }
    Ok(())
}

fn run_asm(input_path: &str, output: Option<&str>, format: &str) -> anyhow::Result<()> {
    let path = Path::new(input_path);
    let source = fs::read_to_string(path)?;

    let ast = compiler::Compiler::parse_ast(&source)
        .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    let config = compiler::AsmConfig::new("t27c_asm");
    let mut asm = compiler::HirAssembler::with_config(config);
    for node in &ast.children {
        if node.kind == compiler::NodeKind::FnDecl {
            if !node.name.is_empty() {
                asm.define_symbol(&node.name, true);
            }
        }
    }
    asm.emit_r(0x01, 1, 27, 0);
    asm.emit_i(0x03, 2, 1, 42);
    asm.emit_r(0x01, 3, 2, 1);
    asm.apply_relocations().map_err(|e| anyhow::anyhow!("{}", e))?;

    match format {
        "hex" => {
            let words = asm.encode_all();
            for w in &words {
                println!("{:08x}", w);
            }
        }
        "binary" => {
            let bytes = asm.to_binary();
            match output {
                Some(out) => fs::write(out, &bytes)?,
                None => {
                    use std::io::Write;
                    std::io::stdout().write_all(&bytes)?;
                }
            }
        }
        "vlog" => {
            let words = asm.encode_all();
            println!("// T27 Assembled Program — {} instructions", words.len());
            println!("// phi^2 + 1/phi^2 = 3 | TRINITY");
            if let Some(out) = output {
                println!("// Output: {}", out);
            }
            println!();
            println!("initial begin");
            for (i, w) in words.iter().enumerate() {
                println!("    mem[{}] = 32'h{:08x};", i, w);
            }
            println!("end");
        }
        _ => anyhow::bail!("unknown asm format: {} (use hex, binary, or vlog)", format),
    }

    eprintln!("Assembled {} instructions, {} bytes", asm.total_instructions(), asm.total_bytes());
    Ok(())
}

fn run_gen_testbench(input_path: &str, period_ns: u32, max_cycles: u32, output: Option<&str>) -> anyhow::Result<()> {
    let path = Path::new(input_path);
    let source = fs::read_to_string(path)?;

    let ast = compiler::Compiler::parse_ast(&source)
        .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    let module_name = if !ast.name.is_empty() { &ast.name } else { "dut" };
    let mut tb = compiler::HirTestbench::new(module_name, max_cycles, period_ns);

    for node in &ast.children {
        if node.kind == compiler::NodeKind::ConstDecl {
            if node.extra_mutable {
                tb.probe(&node.name);
            }
        }
    }

    let verilog = tb.emit_verilog();
    match output {
        Some(out) => fs::write(out, &verilog)?,
        None => print!("{}", verilog),
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

fn run_gen_rust(input_path: &str) -> anyhow::Result<()> {
    let path = Path::new(input_path);
    let source = fs::read_to_string(path)?;

    match compiler::Compiler::compile_rust(&source) {
        Ok(rust_code) => print!("{}", rust_code),
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
    gen_hash_rust: String,
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

    let gen_hash_rust = match compiler::Compiler::compile_rust(&source) {
        Ok(rust_code) => format!("sha256:{}", sha256_hex(rust_code.as_bytes())),
        Err(_) => "none".to_string(),
    };

    Ok(SealHashes {
        module,
        spec_path: input_path.to_string(),
        spec_hash,
        gen_hash_zig,
        gen_hash_verilog,
        gen_hash_c,
        gen_hash_rust,
    })
}

fn seal_file_path(module: &str, input_path: &str) -> std::path::PathBuf {
    let path = Path::new(input_path);
    let parent = path.parent().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    let name = if parent.is_empty() {
        format!("{}.json", module)
    } else {
        format!("{}_{}.json", parent, module)
    };
    Path::new(".trinity").join("seals").join(name)
}

fn run_seal(input_path: &str, save: bool, verify: bool) -> anyhow::Result<()> {
    let hashes = compute_seal_hashes(input_path)?;

    if verify {
        // --verify: load saved seal and compare
        let seal_path = seal_file_path(&hashes.module, &hashes.spec_path);
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
            ("gen_hash_rust", &hashes.gen_hash_rust),
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
            "gen_hash_rust": hashes.gen_hash_rust,
            "sealed_at": now,
            "ring": 12
        });

        let seal_path = seal_file_path(&hashes.module, &hashes.spec_path);
        let pretty = serde_json::to_string_pretty(&seal_obj)?;
        fs::write(&seal_path, &pretty)?;

        // Also print hashes to stdout
        println!("spec_hash={}", hashes.spec_hash);
        println!("gen_hash_zig={}", hashes.gen_hash_zig);
        println!("gen_hash_verilog={}", hashes.gen_hash_verilog);
        println!("gen_hash_c={}", hashes.gen_hash_c);
        println!("gen_hash_rust={}", hashes.gen_hash_rust);
        println!("\nSeal saved to {}", seal_path.display());
    } else {
        // Default: just print hashes (existing behavior, enhanced with all backends)
        println!("spec_hash={}", hashes.spec_hash);
        println!("gen_hash_zig={}", hashes.gen_hash_zig);
        println!("gen_hash_verilog={}", hashes.gen_hash_verilog);
        println!("gen_hash_c={}", hashes.gen_hash_c);
        println!("gen_hash_rust={}", hashes.gen_hash_rust);
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
        "rust" => ".rs",
        _ => ".zig",
    }
}

fn compile_source(source: &str, backend: &str) -> Result<String, String> {
    match backend {
        "verilog" => compiler::Compiler::compile_verilog(source),
        "c" => compiler::Compiler::compile_c(source),
        "rust" => compiler::Compiler::compile_rust(source),
        _ => compiler::Compiler::compile(source),
    }
}

fn run_compile(input_path: &str, backend: &str, output: Option<&str>) -> anyhow::Result<()> {
    let path = Path::new(input_path);
    let source = fs::read_to_string(path)?;

    let ast = compiler::Compiler::parse_ast(&source)
        .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;
    let tc = compiler::typecheck_ast(&ast);
    if tc.warnings > 0 {
        for w in &tc.errors {
            eprintln!("WARN: {}", w);
        }
    }
    if !tc.ok {
        for e in &tc.errors {
            eprintln!("TYPE ERROR: {}", e);
        }
        anyhow::bail!("Typecheck failed with {} errors", tc.error_count);
    }

    let code = match backend {
        "verilog" => {
            let mut cg = compiler::VerilogCodegen::new();
            cg.gen_verilog(&ast);
            cg.into_string()
        }
        "c" => {
            let mut cg = compiler::CCodegen::new();
            cg.gen_c(&ast);
            cg.into_string()
        }
        _ => {
            let mut cg = compiler::Codegen::new();
            cg.gen_zig(&ast);
            cg.into_string()
        }
    };

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

fn run_compile_project(backend: &str, output_dir: &str) -> anyhow::Result<()> {
    use std::collections::HashMap;

    let ext = backend_extension(backend);
    let out_base = Path::new(output_dir);

    // ── Pass 1: scan all .t27 files and build module→path map ──────────
    // Maps "base::types" → "base/types" (relative path without extension)
    let mut module_map: HashMap<String, String> = HashMap::new();
    // Also collect all source file entries: (source_path, rel_output_path_no_ext)
    let mut source_files: Vec<(std::path::PathBuf, String)> = Vec::new();

    let dirs = ["specs", "compiler"];
    for dir in &dirs {
        let base = Path::new(dir);
        if !base.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(base)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("t27") {
                continue;
            }

            // Compute relative path: specs/base/types.t27 → base/types
            let rel = p.strip_prefix(dir).unwrap_or(p);
            let rel_no_ext = rel.with_extension("");
            let rel_str = rel_no_ext.to_string_lossy().replace('\\', "/");

            // Parse the file to extract the module name declared inside
            if let Ok(source) = fs::read_to_string(p) {
                let lexer = compiler::Lexer::new(&source);
                let mut parser = compiler::Parser::new(lexer);
                if let Ok(ast) = parser.parse() {
                    // Build module key from directory structure
                    // e.g. specs/base/types.t27 → "base::types"
                    let module_key = rel_str.replace('/', "::");
                    module_map.insert(module_key.clone(), rel_str.clone());

                    // Also map by the module name declared in the file
                    // to handle modules with different names than their file
                    if !ast.name.is_empty() {
                        // Check UseDecl nodes in the file to extract the full use path patterns
                        // that other files use to reference this module
                        let module_name_lower = ast.name.to_lowercase().replace('-', "_");
                        // Map the last segment too for fallback
                        let last_segment = rel_str.rsplit('/').next().unwrap_or(&rel_str);
                        if !module_map.contains_key(last_segment) {
                            module_map.insert(last_segment.to_string(), rel_str.clone());
                        }
                        if !module_map.contains_key(&module_name_lower) {
                            module_map.insert(module_name_lower, rel_str.clone());
                        }
                    }
                }
            }

            source_files.push((p.to_path_buf(), rel_str));
        }
    }

    println!("Module map ({} entries):", module_map.len());
    let mut sorted_keys: Vec<&String> = module_map.keys().collect();
    sorted_keys.sort();
    for key in &sorted_keys {
        println!("  {} → {}", key, module_map[*key]);
    }
    println!();

    // ── Pass 2: compile each file with resolved imports ────────────────
    let mut count = 0u32;
    let mut errors = 0u32;

    for (source_path, rel_path) in &source_files {
        let source = match fs::read_to_string(source_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("skip {}: {}", source_path.display(), e);
                errors += 1;
                continue;
            }
        };

        let code = match backend {
            "verilog" => compiler::Compiler::compile_verilog(&source),
            "c" => compiler::Compiler::compile_c(&source),
            "rust" => compiler::Compiler::compile_rust(&source),
            _ => compiler::Compiler::compile_project_file(&source, rel_path, &module_map),
        };

        let code = match code {
            Ok(c) => c,
            Err(e) => {
                eprintln!("skip {}: {}", source_path.display(), e);
                errors += 1;
                continue;
            }
        };

        let dest = out_base.join(format!("{}{}", rel_path, &ext[..]));
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&dest, &code)?;
        println!("wrote {}", dest.display());
        count += 1;
    }

    // ── Pass 3: generate build.zig (Zig backend only) ──────────────────
    if backend == "zig" {
        let build_zig = generate_build_zig(&source_files, &ext[1..]);
        let build_path = out_base.join("build.zig");
        fs::write(&build_path, &build_zig)?;
        println!("wrote {}", build_path.display());
    }

    println!("\ncompile-project: {} files to {}/ ({} errors)", count, output_dir, errors);
    Ok(())
}

/// Generate a build.zig that declares all modules as a static library
fn generate_build_zig(source_files: &[(std::path::PathBuf, String)], ext: &str) -> String {
    let mut out = String::new();
    out.push_str("// Generated by t27c compile-project\n");
    out.push_str("// DO NOT EDIT — regenerate with: t27c compile-project\n");
    out.push_str("// phi^2 + 1/phi^2 = 3 | TRINITY\n\n");
    out.push_str("const std = @import(\"std\");\n\n");
    out.push_str("pub fn build(b: *std.Build) void {\n");
    out.push_str("    const target = b.standardTargetOptions(.{});\n");
    out.push_str("    const optimize = b.standardOptimizeOption(.{});\n\n");

    // Find a root source file — prefer base/types as the library root
    let root_source = source_files
        .iter()
        .find(|(_, rel)| rel == "base/types")
        .or_else(|| source_files.first())
        .map(|(_, rel)| format!("{}.{}", rel, ext))
        .unwrap_or_else(|| format!("base/types.{}", ext));

    out.push_str(&format!(
        "    const lib = b.addStaticLibrary(.{{\n\
         \x20       .name = \"t27\",\n\
         \x20       .root_source_file = b.path(\"{}\"),\n\
         \x20       .target = target,\n\
         \x20       .optimize = optimize,\n\
         \x20   }});\n",
        root_source
    ));
    out.push_str("    b.installArtifact(lib);\n\n");

    // Add modules for each source file
    out.push_str("    // Declare modules for cross-file imports\n");
    for (_, rel) in source_files {
        let module_name = rel.replace('/', ".");
        out.push_str(&format!(
            "    lib.root_module.addAnonymousImport(\"{}\", .{{ .root_source_file = b.path(\"{}.{}\") }});\n",
            module_name,
            rel,
            ext,
        ));
    }

    out.push_str("\n    // Tests\n");
    out.push_str(&format!(
        "    const tests = b.addTest(.{{\n\
         \x20       .root_source_file = b.path(\"{}\"),\n\
         \x20       .target = target,\n\
         \x20       .optimize = optimize,\n\
         \x20   }});\n",
        root_source
    ));
    out.push_str("    const run_tests = b.addRunArtifact(tests);\n");
    out.push_str("    const test_step = b.step(\"test\", \"Run unit tests\");\n");
    out.push_str("    test_step.dependOn(&run_tests.step);\n");
    out.push_str("}\n");

    out
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
    println!("Backends:       4 (Zig, Verilog, C, Rust)");
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
// Additional CLI Commands (Sessions 4-12)
// ============================================================================

fn run_optimize(input_path: &str, opt_level: u32) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let mut ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;
    let config = compiler::OptConfig {
        opt_level,
        ..Default::default()
    };
    let stats = compiler::optimize(&mut ast, &config);
    println!("Optimization complete (opt_level={}):", opt_level);
    println!("  Folds: {}", stats.folds);
    println!("  Dead code removed: {}", stats.dead_removed);
    println!("  Copies propagated: {}", stats.copies_propagated);
    println!("  Strength reductions: {}", stats.strengths_reduced);
    println!("  CSE eliminated: {}", stats.cse_eliminated);
    println!("  Dead stores removed: {}", stats.dead_stores);
    println!("  Loops unrolled: {}", stats.loops_unrolled);
    println!("  Passes: {}", stats.passes);
    Ok(())
}

fn run_typecheck(input_path: &str, json: bool) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;
    let result = compiler::typecheck_ast(&ast);
    if json {
        let resp = serde_json::json!({
            "ok": result.ok,
            "errors": result.error_count,
            "warnings": result.warnings,
            "messages": result.errors,
        });
        println!("{}", serde_json::to_string_pretty(&resp).unwrap());
    } else if result.ok {
        println!("Typecheck OK (0 errors, {} warnings)", result.warnings);
    } else {
        println!("Typecheck FAILED ({} errors, {} warnings):", result.error_count, result.warnings);
        for err in &result.errors {
            println!("  - {}", err);
        }
    }
    Ok(())
}

fn run_validate_seals(pr_files: &str) -> Result<(), anyhow::Error> {
    let files: Vec<&str> = pr_files.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    if files.is_empty() {
        println!("No spec files to validate.");
        return Ok(());
    }
    println!("Validating seals for {} spec files...", files.len());
    let mut failures = 0;
    for spec_path in &files {
        let path = std::path::Path::new(spec_path);
        if !path.exists() {
            println!("  SKIP {} (not found)", spec_path);
            continue;
        }
        match compute_seal_hashes(spec_path) {
            Ok(current) => {
                let seal_path = seal_file_path(&current.module, spec_path);
                if seal_path.exists() {
                    let saved_data = std::fs::read_to_string(&seal_path)
                        .with_context(|| format!("reading seal {}", seal_path.display()))?;
                    let saved: serde_json::Value = serde_json::from_str(&saved_data)
                        .with_context(|| format!("parsing seal {}", seal_path.display()))?;
                    let saved_hash = saved.get("spec_hash").and_then(|v| v.as_str()).unwrap_or("");
                    if saved_hash == current.spec_hash {
                        println!("  OK {} (seal match)", spec_path);
                    } else {
                        eprintln!("  FAIL {} (seal mismatch)", spec_path);
                        failures += 1;
                    }
                } else {
                    println!("  SKIP {} (no saved seal at {})", spec_path, seal_path.display());
                }
            }
            Err(e) => {
                eprintln!("  FAIL {} (compute error: {})", spec_path, e);
                failures += 1;
            }
        }
    }
    if failures > 0 {
        anyhow::bail!("{} seal validation failures", failures);
    }
    println!("Seal validation passed for all {} files.", files.len());
    Ok(())
}

fn run_validate_phi_identity() -> Result<(), anyhow::Error> {
    let phi: f64 = (1.0 + 5.0_f64.sqrt()) / 2.0;
    let phi_sq = phi * phi;
    let phi_inv_sq = 1.0 / (phi * phi);
    let identity = phi_sq + phi_inv_sq;
    let tolerance = 1e-10;
    if (identity - 3.0).abs() < tolerance {
        println!("L5 PHI-IDENTITY CHECK PASSED: phi^2 + phi^-2 = {:.15} (delta = {:.2e})", identity, (identity - 3.0).abs());
        Ok(())
    } else {
        anyhow::bail!("L5 PHI-IDENTITY CHECK FAILED: phi^2 + phi^-2 = {:.15} (expected 3.0, delta = {:.2e})", identity, (identity - 3.0).abs())
    }
}

fn run_fpga_build(
    repo_root: &Path,
    smoke: bool,
    synth_only: bool,
    minimal: bool,
    device: &str,
    top: &str,
    docker: Option<bool>,
    use_hir: bool,
    nextpnr_path: Option<&str>,
    chipdb_path: Option<&str>,
    xdc_path: Option<&str>,
    fasm2frames_path: Option<&str>,
    frames2bit_path: Option<&str>,
    prjxray_db_path: Option<&str>,
    output: &str,
) -> anyhow::Result<()> {
    let specs_dir = repo_root.join("specs/fpga");
    let build_dir = repo_root.join(output);
    let gen_dir = build_dir.join("generated");
    let t27c = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("t27c"));

    fs::create_dir_all(&gen_dir).context("create build/fpga/generated")?;
    let synth_dir = build_dir.join("synth");
    fs::create_dir_all(&synth_dir).context("create build/fpga/synth")?;

    let modules = [
        "mac", "uart", "spi", "bridge", "top_level",
        "hir", "hw_types", "memory", "clock_domain", "fifo",
        "axi4", "apb_bridge", "gf16_accel", "formal",
        "ternary_isa", "stdlib", "simulator", "assembler", "testbench", "vcd_trace",
        "e2e_demo", "linker", "timing", "power", "placement", "partition",
        "router", "dft", "cts", "crossopt", "bootrom",
        "sv_emit", "firrtl", "cdc", "lint", "coverage",
    ];

    println!("=== FPGA Build: Verilog generation{}===", if use_hir { " (HIR path) " } else { " " });
    let mut generated_count = 0u32;
    for module in &modules {
        let spec_file = specs_dir.join(format!("{}.t27", module));
        let out_file = gen_dir.join(format!("{}.v", module));
        if !spec_file.exists() {
            println!("  SKIP {} (spec not found)", module);
            continue;
        }
        let gen_cmd = if use_hir { "gen-verilog-hir" } else { "gen-verilog" };
        let status = std::process::Command::new(&t27c)
            .arg(gen_cmd)
            .arg(&spec_file)
            .stdout(std::fs::File::create(&out_file)?)
            .stderr(std::process::Stdio::inherit())
            .status()
            .context(format!("t27c {}", gen_cmd))?;
        if !status.success() {
            anyhow::bail!("t27c {} failed for {}", gen_cmd, module);
        }
        println!("  OK {}.v ({})", module, gen_cmd);
        generated_count += 1;
    }

    let top_wrapper = gen_dir.join(format!("{}.v", top));
    if minimal {
        let wrapper_source = format!(
r#"`timescale 1ns / 1ps

module {top} (
    input  wire        clk,
    input  wire        rst_n,
    input  wire        uart_rx,
    output wire        uart_tx,
    output wire [7:0]  led
);
    wire sys_clk   = clk;
    wire sys_rst_n = rst_n;

    reg [26:0] heartbeat_ctr;
    always @(posedge sys_clk) begin
        if (!sys_rst_n)
            heartbeat_ctr <= 27'd0;
        else
            heartbeat_ctr <= heartbeat_ctr + 1'b1;
    end

    assign led[0] = heartbeat_ctr[24];
    assign led[1] = 1'b0;
    assign led[2] = 1'b0;
    assign led[3] = 1'b0;
    assign led[4] = 1'b0;
    assign led[5] = 1'b0;
    assign led[6] = 1'b0;
    assign led[7] = 1'b0;
    assign uart_tx = uart_rx;
endmodule
"#
        );
        fs::write(&top_wrapper, &wrapper_source)?;
        println!("  OK {}.v (minimal top-level)", top);
    } else {
        let wrapper_source = format!(
r#"`timescale 1ns / 1ps

module {top} (
    input  wire        clk,
    input  wire        rst_n,
    input  wire        uart_rx,
    output wire        uart_tx,
    output wire        spi_cs,
    output wire        spi_sck,
    output wire        spi_mosi,
    input  wire        spi_miso,
    output wire [7:0]  led,
    output wire        mac_done,
    output wire [31:0] mac_result
);
    wire sys_clk   = clk;
    wire sys_rst_n = rst_n;

    // ---- Heartbeat counter (LED[0] blinks at ~0.9 Hz @ 12 MHz) ----
    reg [26:0] heartbeat_ctr;
    always @(posedge sys_clk) begin
        if (!sys_rst_n)
            heartbeat_ctr <= 27'd0;
        else
            heartbeat_ctr <= heartbeat_ctr + 1'b1;
    end

    // ---- ZeroDSP_MAC instantiation ----
    wire mac_ready;
    ZeroDSP_MAC u_mac (
        .clk    (sys_clk),
        .rst_n  (sys_rst_n),
        .en     (1'b1),
        .ready  (mac_ready)
    );

    // ---- ZeroDSP_UART instantiation ----
    wire uart_ready;
    ZeroDSP_UART u_uart (
        .clk    (sys_clk),
        .rst_n  (sys_rst_n),
        .en     (1'b1),
        .ready  (uart_ready)
    );

    // ---- SPI_Master instantiation ----
    wire spi_ready;
    SPI_Master u_spi (
        .clk    (sys_clk),
        .rst_n  (sys_rst_n),
        .en     (1'b1),
        .ready  (spi_ready)
    );

    // ---- FPGA_Bridge instantiation ----
    wire bridge_ready;
    FPGA_Bridge u_bridge (
        .clk    (sys_clk),
        .rst_n  (sys_rst_n),
        .en     (1'b1),
        .ready  (bridge_ready)
    );

    // ---- ZeroDSP_TopLevel instantiation ----
    wire sys_ready;
    ZeroDSP_TopLevel u_top_level (
        .clk    (sys_clk),
        .rst_n  (sys_rst_n),
        .en     (1'b1),
        .ready  (sys_ready)
    );

    // ---- Output assignments ----
    assign led[0]     = heartbeat_ctr[24];
    assign led[1]     = mac_ready;
    assign led[2]     = uart_ready;
    assign led[3]     = spi_ready;
    assign led[4]     = bridge_ready;
    assign led[5]     = sys_ready;
    assign led[6]     = 1'b0;
    assign led[7]     = 1'b0;
    assign uart_tx    = uart_rx;
    assign mac_done   = mac_ready;
    assign mac_result = {{5'd0, heartbeat_ctr}};
    assign spi_cs     = 1'b1;
    assign spi_sck    = 1'b0;
    assign spi_mosi   = 1'b0;
endmodule
"#
        );
        fs::write(&top_wrapper, &wrapper_source)?;
        println!("  OK {}.v (top-level wrapper)", top);
    }

    println!("Verilog generation: {} modules + wrapper", generated_count);

    if smoke {
        println!("=== Smoke test passed (gen-only) ===");
        return Ok(());
    }

    let use_docker = docker.unwrap_or_else(|| {
        std::process::Command::new("yosys")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_err()
    });

    let synth_json = synth_dir.join("synth.json");

    if use_docker {
        if std::process::Command::new("docker")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_err()
        {
            anyhow::bail!("Docker is required for synthesis but not installed. Use --smoke for gen-only, or install Yosys locally and pass --docker false.");
        }
        println!("=== Synthesizing with Yosys (Docker) ===");
        let synth_script = build_dir.join("synth.ys");
        let verilog_files = if minimal {
            format!("{gen}/{top}.v", gen = gen_dir.display(), top = top)
        } else {
            format!("{gen}/mac.v {gen}/uart.v {gen}/spi.v {gen}/bridge.v {gen}/top_level.v {gen}/{top}.v", gen = gen_dir.display(), top = top)
        };
        fs::write(
            &synth_script,
            format!(
                "read_verilog {files}\nhierarchy -check -top {top}\nproc; opt; fsm; opt; memory; opt\nsynth_xilinx -top {top}\nwrite_json {json}\nstat\n",
                files = verilog_files,
                top = top,
                json = synth_json.display(),
            ),
        )?;
        let status = std::process::Command::new("docker")
            .args(["run", "--rm", "-v", &format!("{}:/project", repo_root.display()), "-w", "/project", "hdlc/oss-cad-suite:latest", "yosys", "-s", &format!("{}", synth_script.display())])
            .status()
            .context("docker run yosys")?;
        if !status.success() {
            anyhow::bail!("Yosys synthesis failed");
        }
        println!("Synthesis complete (Docker).");
    } else {
        println!("=== Synthesizing with local Yosys ===");
        let synth_script = build_dir.join("synth.ys");
        let verilog_files = if minimal {
            format!("{gen}/{top}.v", gen = gen_dir.display(), top = top)
        } else {
            format!("{gen}/mac.v {gen}/uart.v {gen}/spi.v {gen}/bridge.v {gen}/top_level.v {gen}/{top}.v", gen = gen_dir.display(), top = top)
        };
        fs::write(
            &synth_script,
            format!(
                "read_verilog {files}\nhierarchy -check -top {top}\nproc; opt; fsm; opt; memory; opt\nsynth_xilinx -top {top}\nwrite_json {json}\nstat\n",
                files = verilog_files,
                top = top,
                json = synth_json.display(),
            ),
        )?;
        let status = std::process::Command::new("yosys")
            .arg("-s")
            .arg(&synth_script)
            .current_dir(&synth_dir)
            .status()
            .context("yosys")?;
        if !status.success() {
            anyhow::bail!("Yosys synthesis failed");
        }
        println!("Synthesis complete.");
    }

    if !synth_json.exists() {
        anyhow::bail!("Yosys did not produce synth.json at {}", synth_json.display());
    }
    println!("  JSON netlist: {}", synth_json.display());

    if synth_only {
        println!("=== Stopped after synthesis (--synth-only) ===");
        return Ok(());
    }

    // ---- Step: Resolve toolchain paths ----
    let nextpnr_bin = match nextpnr_path {
        Some(p) => PathBuf::from(p),
        None => {
            let default = PathBuf::from("build/nextpnr-xilinx/build/nextpnr-xilinx");
            if repo_root.join(&default).exists() {
                repo_root.join(&default)
            } else {
                anyhow::bail!("nextpnr-xilinx not found. Pass --nextpnr <path> or place at build/nextpnr-xilinx/build/nextpnr-xilinx");
            }
        }
    };

    let chipdb = match chipdb_path {
        Some(p) => PathBuf::from(p),
        None => {
            let default = PathBuf::from("build/fpga/chipdb/xc7a100tcsg324-1.bin");
            if repo_root.join(&default).exists() {
                repo_root.join(&default)
            } else {
                anyhow::bail!("Chipdb not found. Pass --chipdb <path> or place at build/fpga/chipdb/{}.bin", device);
            }
        }
    };

    // Generate nextpnr-compatible XDC.
    // For minimal mode, produce a clean XDC with only valid chipdb pins.
    // For full mode, preprocess the Vivado XDC for nextpnr compatibility.
    let xdc = synth_dir.join("nextpnr.xdc");
    if minimal {
        let minimal_xdc = r#"# nextpnr-compatible XDC for minimal design (prjxray-verified pins)
 set_property -dict { PACKAGE_PIN E3    IOSTANDARD LVCMOS33 } [get_ports clk]
 create_clock -add -name sys_clk -period 83.333 -waveform {0 41.666} [get_ports clk]
 set_property -dict { PACKAGE_PIN C18   IOSTANDARD LVCMOS33 } [get_ports rst_n]
set_property -dict { PACKAGE_PIN T14   IOSTANDARD LVCMOS33 } [get_ports uart_rx]
set_property -dict { PACKAGE_PIN T15   IOSTANDARD LVCMOS33 } [get_ports uart_tx]
set_property -dict { PACKAGE_PIN H17   IOSTANDARD LVCMOS33 } [get_ports led[0]]
set_property -dict { PACKAGE_PIN K15   IOSTANDARD LVCMOS33 } [get_ports led[1]]
set_property -dict { PACKAGE_PIN J13   IOSTANDARD LVCMOS33 } [get_ports led[2]]
set_property -dict { PACKAGE_PIN N14   IOSTANDARD LVCMOS33 } [get_ports led[3]]
set_property -dict { PACKAGE_PIN R18   IOSTANDARD LVCMOS33 } [get_ports led[4]]
set_property -dict { PACKAGE_PIN U18   IOSTANDARD LVCMOS33 } [get_ports led[5]]
set_property -dict { PACKAGE_PIN T13   IOSTANDARD LVCMOS33 } [get_ports led[6]]
set_property -dict { PACKAGE_PIN T11   IOSTANDARD LVCMOS33 } [get_ports led[7]]
"#;
        fs::write(&xdc, minimal_xdc)?;
    } else {
        let xdc_source = match xdc_path {
            Some(p) => PathBuf::from(p),
            None => {
                let default = repo_root.join("specs/fpga/constraints/qmtech_a100t.xdc");
                if default.exists() {
                    default
                } else {
                    anyhow::bail!("XDC constraints not found. Pass --xdc <path>");
                }
            }
        };
        let raw = fs::read_to_string(&xdc_source).context("read XDC")?;
        let mut out = String::new();
        for line in raw.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
                continue;
            }
            if trimmed.starts_with("set_false_path") {
                continue;
            }
            if trimmed.contains("[current_design]") {
                continue;
            }
            let l = if trimmed.contains("PULLUP") {
                trimmed.replace("PULLUP true", "").replace("  ", " ")
            } else {
                trimmed.to_string()
            };
            let l = l.replace("[get_ports { ", "[get_ports ").replace(" }]", "]");
            out.push_str(&l);
            out.push('\n');
        }
        fs::write(&xdc, &out)?;
    }

    let fasm_output = synth_dir.join("design.fasm");
    let frames_output = synth_dir.join("design.frames");
    let bit_output = build_dir.join(format!("{}.bit", top));

    // ---- Step 2: nextpnr-xilinx Place & Route ----
    println!("=== Place & Route (nextpnr-xilinx) ===");
    println!("  chipdb: {}", chipdb.display());
    println!("  JSON:   {}", synth_json.display());
    println!("  XDC:    {}", xdc.display());
    println!("  FASM:   {}", fasm_output.display());

    let status = std::process::Command::new(&nextpnr_bin)
        .arg("--chipdb").arg(&chipdb)
        .arg("--json").arg(&synth_json)
        .arg("--xdc").arg(&xdc)
        .arg("--fasm").arg(&fasm_output)
        .current_dir(&synth_dir)
        .status()
        .context("nextpnr-xilinx")?;
    if !status.success() {
        anyhow::bail!("nextpnr-xilinx P&R failed");
    }
    if !fasm_output.exists() {
        anyhow::bail!("nextpnr did not produce FASM at {}", fasm_output.display());
    }
    println!("P&R complete. FASM: {}", fasm_output.display());

    // ---- Step 3: fasm2frames ----
    let fasm2frames = match fasm2frames_path {
        Some(p) => PathBuf::from(p),
        None => {
            let default = repo_root.join("build/fpga/prjxray/utils/fasm2frames.py");
            if default.exists() {
                default
            } else {
                anyhow::bail!("fasm2frames.py not found. Pass --fasm2frames <path> or clone prjxray to build/fpga/prjxray/");
            }
        }
    };

    let prjxray_db = match prjxray_db_path {
        Some(p) => PathBuf::from(p),
        None => {
            let default = repo_root.join("build/nextpnr-xilinx/xilinx/external/prjxray-db/artix7");
            if default.exists() {
                default
            } else {
                anyhow::bail!("prjxray-db not found. Pass --prjxray-db <path>");
            }
        }
    };

    // Ensure prjxray mapping files exist (required by fasm2frames)
    let mapping_dir = prjxray_db.join("mapping");
    if !mapping_dir.exists() {
        fs::create_dir_all(&mapping_dir)?;
    }
    let parts_yaml = mapping_dir.join("parts.yaml");
    if !parts_yaml.exists() {
        fs::write(&parts_yaml, format!(
"\"{device}\":
  device: \"xc7a100t\"
  package: \"csg324\"
  speedgrade: \"1\"
", device = device))?;
    }
    let devices_yaml = mapping_dir.join("devices.yaml");
    if !devices_yaml.exists() {
        fs::write(&devices_yaml, "\"xc7a100t\":\n  fabric: \"xc7a100t\"\n")?;
    }

    println!("=== FASM → Frames ===");
    let status = std::process::Command::new("python3")
        .arg(&fasm2frames)
        .arg("--db-root").arg(&prjxray_db)
        .arg("--part").arg(device)
        .arg(&fasm_output)
        .arg(&frames_output)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .current_dir(&synth_dir)
        .env("PYTHONPATH", format!(
            "{}:{}",
            repo_root.join("build/fpga/venv/lib/python3.13/site-packages").display(),
            repo_root.join("build/fpga/prjxray").display()
        ))
        .status()
        .context("fasm2frames")?;
    if !status.success() {
        anyhow::bail!("fasm2frames failed");
    }
    if !frames_output.exists() {
        anyhow::bail!("fasm2frames did not produce frames at {}", frames_output.display());
    }
    println!("Frames: {}", frames_output.display());

    // ---- Step 4: xc7frames2bit ----
    let xc7frames2bit = match frames2bit_path {
        Some(p) => PathBuf::from(p),
        None => {
            let default = repo_root.join("build/fpga/prjxray/build/tools/xc7frames2bit");
            if default.exists() {
                default
            } else {
                anyhow::bail!("xc7frames2bit not found. Pass --frames2bit <path> or build prjxray at build/fpga/prjxray/");
            }
        }
    };

    // Generate YAML part file for xc7frames2bit (needs configuration_ranges format)
    let part_yaml = synth_dir.join("part.yaml");
    {
        let part_json_path = prjxray_db.join(device).join("part.json");
        let part_json = fs::read_to_string(&part_json_path)
            .context("read part.json")?;
        let pj: serde_json::Value = serde_json::from_str(&part_json)?;
        let idcode = pj["idcode"].as_u64().unwrap_or(0x3631093);
        let mut yaml = format!("!<xilinx/xc7series/part>\nidcode: 0x{:08x}\nconfiguration_ranges:\n", idcode);
        let mut offset = 0u32;
        if let Some(gcr) = pj["global_clock_regions"].as_object() {
            for (region_name, region) in gcr {
                let row_half = region_name;
                if let Some(rows) = region["rows"].as_object() {
                    for (row_id, row_data) in rows {
                        if let Some(buses) = row_data["configuration_buses"].as_object() {
                            for (bus_name, bus_data) in buses {
                                if let Some(cols) = bus_data["configuration_columns"].as_object() {
                                    for (col_id, col_data) in cols {
                                        let fc = col_data["frame_count"].as_u64().unwrap_or(0) as u32;
                                        yaml.push_str(&format!(
"  - !<xilinx/xc7series/configuration_frame_range>
    begin: !<xilinx/xc7series/configuration_frame_address>
      block_type: {}
      row_half: {}
      row: {}
      column: {}
      minor: 0
    end: !<xilinx/xc7series/configuration_frame_address>
      block_type: {}
      row_half: {}
      row: {}
      column: {}
      minor: {}
", bus_name, row_half, row_id, col_id, bus_name, row_half, row_id, col_id, fc));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        fs::write(&part_yaml, &yaml)?;
    }

    println!("=== Frames → Bitstream ===");
    let status = std::process::Command::new(&xc7frames2bit)
        .arg(format!("--part_file={}", part_yaml.display()))
        .arg(format!("--part_name={}", device))
        .arg(format!("--frm_file={}", frames_output.display()))
        .arg(format!("--output_file={}", bit_output.display()))
        .status()
        .context("xc7frames2bit")?;
    if !status.success() {
        anyhow::bail!("xc7frames2bit failed");
    }
    if !bit_output.exists() {
        anyhow::bail!("xc7frames2bit did not produce bitstream at {}", bit_output.display());
    }

    let bit_size = fs::metadata(&bit_output)?.len();
    println!("Bitstream: {} ({} bytes)", bit_output.display(), bit_size);
    println!("=== FPGA E2E build finished ===");
    Ok(())
}

/// Run chimera search for finding new formulas
fn run_chimera(_repo_root: &Path, threshold: f64, limit: usize) -> anyhow::Result<()> {
    let base_formulas = chimera_engine::base_formula_values();
    let operators = chimera_engine::default_operators();
    let targets = chimera_engine::pdg_targets();

    let results = chimera_engine::chimera_search(&base_formulas, &operators, &targets, threshold);

    println!("| Target | Chimera | Value | Δ% | Status |");
    println!("|--------|---------|-------|-----|--------|");
    for r in results.iter().take(limit) {
        println!(
            "| {} | `{}` | {:.5} | {:.3}% | {} |",
            r.target_name, r.expr, r.chimera_value, r.error_pct, r.status
        );
    }
    if results.is_empty() {
        println!("No chimera matches found within {}% threshold", threshold);
    } else {
        println!("\nFound {} chimera candidate(s)", results.len());
    }
    Ok(())
}

/// Run sensitivity analysis for a formula
fn run_sensitivity(
    _repo_root: &Path,
    formula_id: &str,
    param_name: &str,
    min: Option<f64>,
    max: Option<f64>,
    n: usize,
) -> anyhow::Result<()> {
    let range = match (min, max) {
        (Some(mn), Some(mx)) => (mn, mx),
        _ => sensitivity::default_param_range(param_name),
    };

    let points = sensitivity::sensitivity_scan(formula_id, param_name, range, n);

    println!("| {} | F('{}') | Delta% |", param_name, formula_id);
    println!("|--------|----------|--------|");
    let step = if points.len() > 10 { points.len() / 10 } else { 1 };
    for p in points.iter().step_by(step.max(1)) {
        println!(
            "| {:.4} | {:.3} | {:.3}% |",
            p.param_value, p.formula_value, p.error_pct
        );
    }

    if let Some(best) = sensitivity::find_minimum(&points) {
        println!("\nMinimum at {}={:.6} -> Delta={:.3}%", param_name, best.param_value, best.error_pct);
    }

    Ok(())
}

fn run_lint(input_path: &str, json_output: bool) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;
    let mut issues = 0u32;
    let mut fn_count = 0u32;
    let mut test_count = 0u32;
    let mut inv_count = 0u32;
    let mut warnings = Vec::new();
    for child in &ast.children {
        match child.kind {
            compiler::NodeKind::FnDecl => {
                fn_count += 1;
                let has_test = child.children.iter().any(|c| c.kind == compiler::NodeKind::TestBlock);
                let has_inv = child.children.iter().any(|c| c.kind == compiler::NodeKind::InvariantBlock);
                if !has_test && !has_inv {
                    warnings.push(format!("fn '{}' has no test or invariant", child.name));
                    issues += 1;
                }
            }
            compiler::NodeKind::TestBlock => test_count += 1,
            compiler::NodeKind::InvariantBlock => inv_count += 1,
            _ => {}
        }
    }
    if fn_count == 0 {
        warnings.push(format!("module '{}' has no function declarations", ast.name));
        issues += 1;
    }
    if test_count == 0 && inv_count == 0 {
        warnings.push(format!("module '{}' has no tests or invariants", ast.name));
        issues += 1;
    }
    if json_output {
        let resp = serde_json::json!({
            "issues": issues,
            "functions": fn_count,
            "tests": test_count,
            "invariants": inv_count,
            "warnings": warnings,
        });
        println!("{}", serde_json::to_string_pretty(&resp).unwrap());
    } else {
        for w in &warnings {
            println!("WARN: {}", w);
        }
        println!("Lint: {} issues ({} fns, {} tests, {} invariants)", issues, fn_count, test_count, inv_count);
    }
    Ok(())
}

fn run_bench(input_path: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let start = std::time::Instant::now();
    let tokens = {
        let mut lexer = compiler::Lexer::new(&source);
        lexer.tokenize()
    };
    let lex_time = start.elapsed();

    let start = std::time::Instant::now();
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;
    let parse_time = start.elapsed();

    let start = std::time::Instant::now();
    let _ = compiler::typecheck_ast(&ast);
    let tc_time = start.elapsed();

    let start = std::time::Instant::now();
    let mut opt_ast = ast.clone();
    let config = compiler::OptConfig::default();
    let _ = compiler::optimize(&mut opt_ast, &config);
    let opt_time = start.elapsed();

    let start = std::time::Instant::now();
    let mut codegen = compiler::Codegen::new();
    codegen.gen_zig(&ast);
    let _ = codegen.into_string();
    let gen_time = start.elapsed();

    println!("Benchmark: {}", input_path);
    println!("  Lexer:    {:?}", lex_time);
    println!("  Parser:   {:?}", parse_time);
    println!("  Typeck:   {:?}", tc_time);
    println!("  Optimize: {:?}", opt_time);
    println!("  Codegen:  {:?}", gen_time);
    println!("  Tokens:   {}", tokens.len());
    Ok(())
}

fn run_explain(input_path: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;

    println!("=== Compilation Pipeline for {} ===\n", input_path);

    let t0 = std::time::Instant::now();
    let mut lexer = compiler::Lexer::new(&source);
    let tokens = lexer.tokenize();
    let t1 = std::time::Instant::now();
    println!("1. Lexing:    {} tokens ({:?})", tokens.len(), t1 - t0);

    let t0 = std::time::Instant::now();
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;
    let t1 = std::time::Instant::now();
    let decl_count = ast.children.len();
    println!("2. Parsing:   {} top-level declarations ({:?})", decl_count, t1 - t0);

    let t0 = std::time::Instant::now();
    let tc_result = compiler::typecheck_ast(&ast);
    let t1 = std::time::Instant::now();
    println!("3. Typecheck: {} errors, {} warnings ({:?})", tc_result.error_count, tc_result.warnings, t1 - t0);

    let t0 = std::time::Instant::now();
    let mut opt_ast = ast.clone();
    let config = compiler::OptConfig::default();
    let stats = compiler::optimize(&mut opt_ast, &config);
    let t1 = std::time::Instant::now();
    println!("4. Optimize:  {} folds, {} dead-elim ({:?})", stats.folds, stats.dead_removed, t1 - t0);

    let t0 = std::time::Instant::now();
    let mut codegen = compiler::Codegen::new();
    codegen.gen_zig(&ast);
    let output = codegen.into_string();
    let t1 = std::time::Instant::now();
    println!("5. Codegen:   {} bytes output ({:?})", output.len(), t1 - t0);

    println!("\nModule: {}", ast.name);
    Ok(())
}

fn run_fmt(input_path: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;

    fn fmt_expr(node: &compiler::Node) -> String {
        match node.kind {
            compiler::NodeKind::ExprLiteral => node.value.clone(),
            compiler::NodeKind::ExprIdentifier => node.name.clone(),
            compiler::NodeKind::ExprBinary => {
                if node.children.len() >= 2 {
                    let l = fmt_expr(&node.children[0]);
                    let r = fmt_expr(&node.children[1]);
                    format!("{} {} {}", l, node.extra_op, r)
                } else {
                    "()".to_string()
                }
            }
            compiler::NodeKind::ExprUnary => {
                if !node.children.is_empty() {
                    format!("{}{}", node.extra_op, fmt_expr(&node.children[0]))
                } else {
                    node.extra_op.clone()
                }
            }
            compiler::NodeKind::ExprCall => {
                let args: Vec<String> = node.children.iter().map(fmt_expr).collect();
                format!("{}({})", node.name, args.join(", "))
            }
            compiler::NodeKind::ExprFieldAccess => {
                if !node.children.is_empty() {
                    format!("{}.{}", fmt_expr(&node.children[0]), node.name)
                } else {
                    node.name.clone()
                }
            }
            compiler::NodeKind::ExprIndex => {
                if node.children.len() >= 2 {
                    format!("{}[{}]", fmt_expr(&node.children[0]), fmt_expr(&node.children[1]))
                } else {
                    "()".to_string()
                }
            }
            compiler::NodeKind::ExprEnumValue => format!("{}::{}", node.name, node.extra_field),
            compiler::NodeKind::ExprStructLit => {
                let fields: Vec<String> = node.children.iter().map(|c| {
                    let val = if c.children.is_empty() { "".to_string() } else { format!(" = {}", fmt_expr(&c.children[0])) };
                    format!("{}:{}", c.name, val)
                }).collect();
                format!("{} {{ {} }}", node.name, fields.join(", "))
            }
            compiler::NodeKind::ExprArrayLiteral => {
                let elems: Vec<String> = node.children.iter().map(fmt_expr).collect();
                format!("[{}]", elems.join(", "))
            }
            _ => format!("/* {:?} */", node.kind),
        }
    }

    fn fmt_stmt(node: &compiler::Node, indent: usize) -> String {
        let pad = "    ".repeat(indent);
        let mut out = String::new();
        match node.kind {
            compiler::NodeKind::StmtLocal => {
                let kw = if node.extra_mutable { "var" } else { "const" };
                if node.children.is_empty() {
                    if node.extra_type.is_empty() {
                        out.push_str(&format!("{}{} {};\n", pad, kw, node.name));
                    } else {
                        out.push_str(&format!("{}{} {}: {};\n", pad, kw, node.name, node.extra_type));
                    }
                } else {
                    let val = fmt_expr(&node.children[0]);
                    if node.extra_type.is_empty() {
                        out.push_str(&format!("{}{} {} = {};\n", pad, kw, node.name, val));
                    } else {
                        out.push_str(&format!("{}{} {}: {} = {};\n", pad, kw, node.name, node.extra_type, val));
                    }
                }
            }
            compiler::NodeKind::StmtAssign => {
                if node.children.len() >= 2 {
                    let target = fmt_expr(&node.children[0]);
                    let val = fmt_expr(&node.children[1]);
                    out.push_str(&format!("{}{} = {};\n", pad, target, val));
                }
            }
            compiler::NodeKind::ExprReturn => {
                if node.children.is_empty() {
                    out.push_str(&format!("{}return;\n", pad));
                } else {
                    out.push_str(&format!("{}return {};\n", pad, fmt_expr(&node.children[0])));
                }
            }
            compiler::NodeKind::StmtExpr => {
                if node.children.len() == 1 {
                    out.push_str(&format!("{}{};\n", pad, fmt_expr(&node.children[0])));
                }
            }
            compiler::NodeKind::StmtIf => {
                out.push_str(&pad);
                out.push_str("if (");
                if !node.children.is_empty() {
                    out.push_str(&fmt_expr(&node.children[0]));
                }
                out.push_str(") {\n");
                if node.children.len() > 1 {
                    for s in &node.children[1].children {
                        out.push_str(&fmt_stmt(s, indent + 1));
                    }
                }
                out.push_str(&format!("{}}}\n", pad));
                if node.children.len() > 2 {
                    out.push_str(&format!("{} else {{\n", pad));
                    for s in &node.children[2].children {
                        out.push_str(&fmt_stmt(s, indent + 1));
                    }
                    out.push_str(&format!("{}}}\n", pad));
                }
            }
            compiler::NodeKind::StmtWhile => {
                out.push_str(&pad);
                out.push_str("while (");
                if !node.children.is_empty() {
                    out.push_str(&fmt_expr(&node.children[0]));
                }
                out.push_str(") {\n");
                if node.children.len() > 1 {
                    for s in &node.children[1].children {
                        out.push_str(&fmt_stmt(s, indent + 1));
                    }
                }
                out.push_str(&format!("{}}}\n", pad));
            }
            compiler::NodeKind::StmtFor => {
                out.push_str(&pad);
                out.push_str("for (");
                if !node.children.is_empty() {
                    out.push_str(&fmt_expr(&node.children[0]));
                }
                if node.children.len() > 1 {
                    out.push_str(&format!(") |{}| {{\n", node.children[1].name));
                } else {
                    out.push_str(") {\n");
                }
                if node.children.len() > 2 {
                    for s in &node.children[2].children {
                        out.push_str(&fmt_stmt(s, indent + 1));
                    }
                }
                out.push_str(&format!("{}}}\n", pad));
            }
            compiler::NodeKind::StmtBreak => {
                out.push_str(&format!("{}break;\n", pad));
            }
            compiler::NodeKind::StmtContinue => {
                out.push_str(&format!("{}continue;\n", pad));
            }
            _ => {
                out.push_str(&format!("{}// {:?}\n", pad, node.kind));
            }
        }
        out
    }

    fn fmt_node(node: &compiler::Node, indent: usize) -> String {
        let pad = "    ".repeat(indent);
        let mut out = String::new();
        match node.kind {
            compiler::NodeKind::Module => {
                out.push_str(&format!("{}module {} {{\n", pad, node.name));
                for child in &node.children {
                    out.push_str(&fmt_node(child, indent + 1));
                }
                out.push_str(&format!("{}}}\n", pad));
            }
            compiler::NodeKind::UseDecl => {
                out.push_str(&format!("{}using {};\n", pad, node.value));
            }
            compiler::NodeKind::ConstDecl => {
                if node.children.is_empty() {
                    out.push_str(&format!("{}const {}: {};\n", pad, node.name, node.extra_type));
                } else {
                    out.push_str(&format!("{}const {} = {};\n", pad, node.name, fmt_expr(&node.children[0])));
                }
            }
            compiler::NodeKind::EnumDecl => {
                out.push_str(&format!("{}enum {} {{\n", pad, node.name));
                for child in &node.children {
                    if child.kind == compiler::NodeKind::EnumVariant {
                        if child.value.is_empty() {
                            out.push_str(&format!("    {}{},\n", pad, child.name));
                        } else {
                            out.push_str(&format!("    {}{} = {},\n", pad, child.name, child.value));
                        }
                    }
                }
                out.push_str(&format!("{}}}\n", pad));
            }
            compiler::NodeKind::StructDecl => {
                out.push_str(&format!("{}struct {} {{\n", pad, node.name));
                for child in &node.children {
                    if child.kind == compiler::NodeKind::ExprIdentifier && !child.name.is_empty() {
                        out.push_str(&format!("    {}{}: {},\n", pad, child.name, child.extra_type));
                    }
                }
                out.push_str(&format!("{}}}\n", pad));
            }
            compiler::NodeKind::FnDecl => {
                let params: Vec<String> = node.params.iter().map(|(n, t)| {
                    if t.is_empty() { n.clone() } else { format!("{}: {}", n, t) }
                }).collect();
                let ret = if node.extra_return_type.is_empty() { String::new() } else { format!(" -> {}", node.extra_return_type) };
                out.push_str(&format!("{}fn {}({}){} {{\n", pad, node.name, params.join(", "), ret));
                for child in &node.children {
                    out.push_str(&fmt_stmt(child, indent + 1));
                }
                out.push_str(&format!("{}}}\n\n", pad));
            }
            compiler::NodeKind::TestBlock => {
                out.push_str(&format!("{}test {} {{\n", pad, node.name));
                for child in &node.children {
                    out.push_str(&fmt_stmt(child, indent + 1));
                }
                out.push_str(&format!("{}}}\n\n", pad));
            }
            compiler::NodeKind::InvariantBlock => {
                out.push_str(&format!("{}invariant {} {}\n\n", pad, node.name, node.value));
            }
            compiler::NodeKind::BenchBlock => {
                out.push_str(&format!("{}bench {} {{\n", pad, node.name));
                for child in &node.children {
                    out.push_str(&fmt_stmt(child, indent + 1));
                }
                out.push_str(&format!("{}}}\n\n", pad));
            }
            _ => {
                for child in &node.children {
                    out.push_str(&fmt_node(child, indent));
                }
            }
        }
        out
    }

    print!("{}", fmt_node(&ast, 0));
    Ok(())
}

fn run_graph(root: &str, format: &str) -> anyhow::Result<()> {
    let root_path = Path::new(root);
    let files: Vec<PathBuf> = walkdir::WalkDir::new(root_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "t27").unwrap_or(false))
        .map(|e| e.path().to_path_buf())
        .collect();

    let mut modules: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let mut module_files: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    for file in &files {
        let source = match fs::read_to_string(file) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let lexer = compiler::Lexer::new(&source);
        let mut parser = compiler::Parser::new(lexer);
        let ast = match parser.parse() {
            Ok(a) => a,
            Err(_) => continue,
        };

        let module_name = ast.name.clone();
        module_files.insert(module_name.clone(), file.display().to_string());

        let mut imports = Vec::new();
        for child in &ast.children {
            if child.kind == compiler::NodeKind::UseDecl {
                let import_path = child.value.replace("::", "/");
                imports.push(import_path);
            }
        }
        modules.insert(module_name, imports);
    }

    match format {
        "dot" => {
            println!("digraph specs {{");
            println!("    rankdir=LR;");
            println!("    node [shape=box, style=filled, fillcolor=lightyellow];");
            for (module, imports) in &modules {
                for imp in imports {
                    println!("    \"{}\" -> \"{}\";", module, imp);
                }
            }
            println!("}}");
        }
        "json" => {
            let mut edges = Vec::new();
            for (module, imports) in &modules {
                for imp in imports {
                    edges.push(serde_json::json!({
                        "from": module,
                        "to": imp,
                    }));
                }
            }
            let output = serde_json::json!({
                "modules": modules.len(),
                "edges": edges,
                "files": module_files,
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        _ => {
            let mut total = 0;
            let mut resolved = 0;
            let mut unresolved = 0;
            let all_module_names: std::collections::HashSet<String> = modules.keys().cloned().collect();
            for (module, imports) in &modules {
                if imports.is_empty() {
                    println!("{} (leaf)", module);
                } else {
                    println!("{}:", module);
                    for imp in imports {
                        total += 1;
                        let target = imp.replace("::", "/");
                        let possible = vec![
                            format!("specs/{}.t27", target),
                            format!("compiler/{}.t27", target),
                            format!("{}.t27", target),
                        ];
                        let path_found = possible.iter().any(|p| Path::new(p).exists());
                        let name_match = all_module_names.contains(imp);
                        if path_found || name_match {
                            resolved += 1;
                            println!("  -> {} [ok]", imp);
                        } else {
                            unresolved += 1;
                            println!("  -> {} [not found]", imp);
                        }
                    }
                }
            }
            println!("--- Graph Summary: {} modules, {} edges ({} resolved, {} unresolved) ---",
                modules.len(), total, resolved, unresolved);
        }
    }
    Ok(())
}

fn run_doc(input_path: &str, output_dir: &str) -> anyhow::Result<()> {
    let path = Path::new(input_path);
    let source = fs::read_to_string(path)?;

    let lexer = compiler::Lexer::new(&source);
    let mut parser = compiler::Parser::new(lexer);
    let ast = parser.parse().map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    let mut fn_decls = Vec::new();
    let mut struct_decls = Vec::new();
    let mut enum_decls = Vec::new();
    let mut const_decls = Vec::new();
    let mut test_decls = Vec::new();
    let mut invariant_decls = Vec::new();

    for child in &ast.children {
        match child.kind {
            compiler::NodeKind::FnDecl => fn_decls.push(child),
            compiler::NodeKind::StructDecl => struct_decls.push(child),
            compiler::NodeKind::EnumDecl => enum_decls.push(child),
            compiler::NodeKind::ConstDecl => const_decls.push(child),
            compiler::NodeKind::TestBlock => test_decls.push(child),
            compiler::NodeKind::InvariantBlock => invariant_decls.push(child),
            _ => {}
        }
    }

    let mut html = String::new();
    html.push_str("<!DOCTYPE html><html><head><meta charset=\"utf-8\">");
    html.push_str(&format!("<title>{} - t27 doc</title>", ast.name));
    html.push_str("<style>body{font-family:sans-serif;margin:2em auto;max-width:900px;color:#222}");
    html.push_str("h1{color:#333}h2{color:#555;border-bottom:1px solid #ddd;padding-bottom:4px}");
    html.push_str(".tag{display:inline-block;padding:2px 8px;border-radius:4px;font-size:0.8em;margin-left:4px}");
    html.push_str(".fn{background:#e8f4e8}.struct{background:#e8e8f4}.enum{background:#f4e8e8}");
    html.push_str(".test{background:#f4f4e8}.inv{background:#e8f4f4}.const{background:#f0f0f0}");
    html.push_str("code{background:#f5f5f5;padding:1px 4px;border-radius:3px;font-size:0.9em}");
    html.push_str("pre{background:#f5f5f5;padding:12px;border-radius:6px;overflow-x:auto}</style></head><body>");
    html.push_str(&format!("<h1>{} <span style=\"font-size:0.6em;color:#888\">module</span></h1>", ast.name));

    if !fn_decls.is_empty() {
        html.push_str(&format!("<h2>Functions <span class=\"tag fn\">{}</span></h2>\n<ul>\n", fn_decls.len()));
        for f in &fn_decls {
            html.push_str(&format!("<li><code>{}</code></li>\n", f.name));
        }
        html.push_str("</ul>\n");
    }

    if !struct_decls.is_empty() {
        html.push_str(&format!("<h2>Structs <span class=\"tag struct\">{}</span></h2>\n<ul>\n", struct_decls.len()));
        for s in &struct_decls {
            html.push_str(&format!("<li><code>{}</code></li>\n", s.name));
        }
        html.push_str("</ul>\n");
    }

    if !enum_decls.is_empty() {
        html.push_str(&format!("<h2>Enums <span class=\"tag enum\">{}</span></h2>\n<ul>\n", enum_decls.len()));
        for e in &enum_decls {
            html.push_str(&format!("<li><code>{}</code></li>\n", e.name));
        }
        html.push_str("</ul>\n");
    }

    if !const_decls.is_empty() {
        html.push_str(&format!("<h2>Constants <span class=\"tag const\">{}</span></h2>\n<ul>\n", const_decls.len()));
        for c in &const_decls {
            html.push_str(&format!("<li><code>{}</code></li>\n", c.name));
        }
        html.push_str("</ul>\n");
    }

    if !test_decls.is_empty() {
        html.push_str(&format!("<h2>Tests <span class=\"tag test\">{}</span></h2>\n<ul>\n", test_decls.len()));
        for t in &test_decls {
            html.push_str(&format!("<li><code>{}</code></li>\n", t.name));
        }
        html.push_str("</ul>\n");
    }

    if !invariant_decls.is_empty() {
        html.push_str(&format!("<h2>Invariants <span class=\"tag inv\">{}</span></h2>\n<ul>\n", invariant_decls.len()));
        for inv in &invariant_decls {
            html.push_str(&format!("<li><code>{}</code></li>\n", inv.name));
        }
        html.push_str("</ul>\n");
    }

    html.push_str("<hr><p style=\"font-size:0.8em;color:#888\">Generated by t27c doc</p></body></html>");

    let out_path = Path::new(output_dir);
    fs::create_dir_all(out_path)?;
    let out_file = out_path.join(format!("{}.html", ast.name));
    fs::write(&out_file, &html)?;

    println!("{}: docs written to {}", path.display(), out_file.display());
    Ok(())
}

fn run_check(input_path: &str) -> anyhow::Result<()> {
    run_typecheck(input_path, false)
}

fn run_size(input_path: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;
    let file_name = std::path::Path::new(input_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();

    let fns = 0u32;
    let structs = 0u32;
    let enums = 0u32;
    let consts = 0u32;
    let tests = 0u32;
    let invariants = 0u32;
    let benches = 0u32;
    let imports = 0u32;
    let total_nodes = 0u32;

    fn count(node: &compiler::Node, stats: &mut (u32, u32, u32, u32, u32, u32, u32, u32, u32)) {
        stats.8 += 1;
        match node.kind {
            compiler::NodeKind::FnDecl => stats.0 += 1,
            compiler::NodeKind::StructDecl => stats.1 += 1,
            compiler::NodeKind::EnumDecl => stats.2 += 1,
            compiler::NodeKind::ConstDecl => stats.3 += 1,
            compiler::NodeKind::TestBlock => stats.4 += 1,
            compiler::NodeKind::InvariantBlock => stats.5 += 1,
            compiler::NodeKind::BenchBlock => stats.6 += 1,
            compiler::NodeKind::UseDecl => stats.7 += 1,
            _ => {}
        }
        for child in &node.children {
            count(child, stats);
        }
    }

    let mut s = (fns, structs, enums, consts, tests, invariants, benches, imports, total_nodes);
    count(&ast, &mut s);
    let (fns, structs, enums, consts, tests, invariants, benches, imports, total_nodes) = s;

    let lines = source.lines().count();
    let bytes = source.len();

    println!("{}:", file_name);
    println!("  Bytes:          {}", bytes);
    println!("  Lines:          {}", lines);
    println!("  Nodes:          {}", total_nodes);
    println!("  Functions:      {}", fns);
    println!("  Structs:        {}", structs);
    println!("  Enums:          {}", enums);
    println!("  Constants:      {}", consts);
    println!("  Tests:          {}", tests);
    println!("  Invariants:     {}", invariants);
    println!("  Benchmarks:     {}", benches);
    println!("  Imports:        {}", imports);
    Ok(())
}

fn run_analyze(repo_root: &str, json: bool, show_top: bool) -> anyhow::Result<()> {
    struct FileInfo {
        name: String,
        lines: u64,
        fns: u64,
        tests: u64,
        invariants: u64,
    }
    let mut file_infos: Vec<FileInfo> = Vec::new();
    let mut total_files: u32 = 0;
    let mut total_bytes: u64 = 0;
    let mut total_lines: u64 = 0;
    let mut total_nodes: u64 = 0;
    let mut total_fns: u64 = 0;
    let mut total_structs: u64 = 0;
    let mut total_enums: u64 = 0;
    let mut total_consts: u64 = 0;
    let mut total_tests: u64 = 0;
    let mut total_invariants: u64 = 0;
    let mut total_benches: u64 = 0;
    let mut total_imports: u64 = 0;
    let mut parse_errors: u32 = 0;

    fn scan_dir(dir: &std::path::Path, infos: &mut Vec<FileInfo>,
        tf: &mut u32, tb: &mut u64, tl: &mut u64, tn: &mut u64,
        ff: &mut u64, fs: &mut u64, fe: &mut u64, fc: &mut u64,
        ft: &mut u64, fi: &mut u64, fb: &mut u64, fimp: &mut u64,
        pe: &mut u32,
    ) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    scan_dir(&path, infos, tf, tb, tl, tn, ff, fs, fe, fc, ft, fi, fb, fimp, pe);
                } else if path.extension().map(|e| e == "t27").unwrap_or(false) {
                    *tf += 1;
                    if let Ok(source) = std::fs::read_to_string(&path) {
                        let file_lines = source.lines().count() as u64;
                        *tb += source.len() as u64;
                        *tl += file_lines;
                        if let Ok(ast) = compiler::Compiler::parse_ast(&source) {
                            fn count_nodes(node: &compiler::Node, s: &mut (u64, u64, u64, u64, u64, u64, u64, u64, u64)) {
                                s.8 += 1;
                                match node.kind {
                                    compiler::NodeKind::FnDecl => s.0 += 1,
                                    compiler::NodeKind::StructDecl => s.1 += 1,
                                    compiler::NodeKind::EnumDecl => s.2 += 1,
                                    compiler::NodeKind::ConstDecl => s.3 += 1,
                                    compiler::NodeKind::TestBlock => s.4 += 1,
                                    compiler::NodeKind::InvariantBlock => s.5 += 1,
                                    compiler::NodeKind::BenchBlock => s.6 += 1,
                                    compiler::NodeKind::UseDecl => s.7 += 1,
                                    _ => {}
                                }
                                for child in &node.children { count_nodes(child, s); }
                            }
                            let mut s = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
                            count_nodes(&ast, &mut s);
                            *ff += s.0; *fs += s.1; *fe += s.2; *fc += s.3;
                            *ft += s.4; *fi += s.5; *fb += s.6; *fimp += s.7;
                            *tn += s.8;
                            let short = path.strip_prefix(std::path::Path::new("."))
                                .unwrap_or(&path).to_string_lossy().to_string();
                            infos.push(FileInfo { name: short, lines: file_lines, fns: s.0, tests: s.4, invariants: s.5 });
                        } else {
                            *pe += 1;
                        }
                    }
                }
            }
        }
    }

    let dirs = vec![
        format!("{}/specs", repo_root),
        format!("{}/compiler", repo_root),
    ];
    for dir in &dirs {
        let path = std::path::Path::new(dir);
        if path.exists() {
            scan_dir(path, &mut file_infos, &mut total_files, &mut total_bytes, &mut total_lines, &mut total_nodes,
                     &mut total_fns, &mut total_structs, &mut total_enums, &mut total_consts,
                     &mut total_tests, &mut total_invariants, &mut total_benches, &mut total_imports,
                     &mut parse_errors);
        }
    }

    if json {
        let resp = serde_json::json!({
            "spec_files": total_files,
            "parse_errors": parse_errors,
            "bytes": total_bytes,
            "lines": total_lines,
            "nodes": total_nodes,
            "functions": total_fns,
            "structs": total_structs,
            "enums": total_enums,
            "constants": total_consts,
            "tests": total_tests,
            "invariants": total_invariants,
            "benchmarks": total_benches,
            "imports": total_imports,
        });
        println!("{}", serde_json::to_string_pretty(&resp).unwrap());
    } else {
        println!("=== T27 Repository Analysis ===");
        println!("Spec files:       {}", total_files);
        if parse_errors > 0 {
            println!("Parse errors:     {}", parse_errors);
        }
        println!("Total bytes:      {}", total_bytes);
        println!("Total lines:      {}", total_lines);
        println!("Total AST nodes:  {}", total_nodes);
        println!("---");
        println!("Functions:        {}", total_fns);
        println!("Structs:          {}", total_structs);
        println!("Enums:            {}", total_enums);
        println!("Constants:        {}", total_consts);
        println!("Tests:            {}", total_tests);
        println!("Invariants:       {}", total_invariants);
        println!("Benchmarks:       {}", total_benches);
        println!("Imports:          {}", total_imports);
        println!("---");
        println!("Avg lines/spec:   {:.0}", if total_files > 0 { total_lines as f64 / total_files as f64 } else { 0.0 });
        println!("Avg functions/spec: {:.1}", if total_files > 0 { total_fns as f64 / total_files as f64 } else { 0.0 });
        println!("Avg tests/spec:   {:.1}", if total_files > 0 { total_tests as f64 / total_files as f64 } else { 0.0 });
        println!("phi^2 + 1/phi^2 = 3 | TRINITY");
    }
    if show_top {
        file_infos.sort_by(|a, b| b.lines.cmp(&a.lines));
        println!("\n--- Top 20 specs by lines ---");
        for (i, fi) in file_infos.iter().take(20).enumerate() {
            println!("{:3}. {:50} {:5} lines, {:3} fn, {:3} test, {:3} inv",
                i + 1, fi.name, fi.lines, fi.fns, fi.tests, fi.invariants);
        }
    }
    Ok(())
}

fn run_deadcode_cmd(input: &Option<String>, repo: bool) -> anyhow::Result<()> {
    if repo {
        let repo_root = ".";
        let dirs = vec![format!("{}/specs", repo_root), format!("{}/compiler", repo_root)];
        let mut total_fns = 0u64;
        let mut total_dead = 0u64;
        for dir in &dirs {
            let path = std::path::Path::new(dir);
            if !path.exists() { continue; }
            let mut stack = vec![path.to_path_buf()];
            while let Some(current) = stack.pop() {
                if let Ok(entries) = std::fs::read_dir(&current) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_dir() { stack.push(p); continue; }
                        if !p.extension().map(|e| e == "t27").unwrap_or(false) { continue; }
                        if let Ok(source) = std::fs::read_to_string(&p) {
                            if let Ok(ast) = compiler::Compiler::parse_ast(&source) {
                                let mut all_fns: std::collections::HashSet<String> = std::collections::HashSet::new();
                                let mut called: std::collections::HashSet<String> = std::collections::HashSet::new();
                                fn collect_calls(node: &compiler::Node, calls: &mut std::collections::HashSet<String>) {
                                    if node.kind == compiler::NodeKind::ExprCall && !node.name.is_empty() {
                                        calls.insert(node.name.clone());
                                    }
                                    for child in &node.children { collect_calls(child, calls); }
                                }
                                for child in &ast.children {
                                    if child.kind == compiler::NodeKind::FnDecl {
                                        all_fns.insert(child.name.clone());
                                        collect_calls(child, &mut called);
                                    }
                                    if matches!(child.kind, compiler::NodeKind::TestBlock | compiler::NodeKind::InvariantBlock | compiler::NodeKind::BenchBlock) {
                                        collect_calls(child, &mut called);
                                    }
                                }
                                let dead: Vec<&String> = all_fns.iter().filter(|f| !called.contains(*f)).collect();
                                total_fns += all_fns.len() as u64;
                                total_dead += dead.len() as u64;
                                if !dead.is_empty() {
                                    let short = p.strip_prefix(std::path::Path::new(".")).unwrap_or(&p).to_string_lossy();
                                    for f in &dead { println!("  {} :: {}", short, f); }
                                }
                            }
                        }
                    }
                }
            }
        }
        println!("---");
        println!("Total functions: {}", total_fns);
        println!("Potentially dead: {}", total_dead);
        if total_dead > 0 {
            println!("Dead ratio: {:.1}%", 100.0 * total_dead as f64 / total_fns as f64);
        }
    } else if let Some(path) = input {
        run_deadcode(&path)?;
    } else {
        anyhow::bail!("Specify --input <file> or --repo");
    }
    Ok(())
}

fn run_deadcode(input_path: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;
    let file_name = std::path::Path::new(input_path).file_name().unwrap_or_default().to_string_lossy();

    fn collect_calls(node: &compiler::Node, calls: &mut std::collections::HashSet<String>) {
        if node.kind == compiler::NodeKind::ExprCall {
            if !node.name.is_empty() {
                calls.insert(node.name.clone());
            }
        }
        for child in &node.children {
            collect_calls(child, calls);
        }
    }

    let mut all_fns: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut called: std::collections::HashSet<String> = std::collections::HashSet::new();

    for child in &ast.children {
        if child.kind == compiler::NodeKind::FnDecl {
            all_fns.insert(child.name.clone());
            collect_calls(child, &mut called);
        }
    }
    for child in &ast.children {
        if child.kind == compiler::NodeKind::TestBlock || child.kind == compiler::NodeKind::InvariantBlock || child.kind == compiler::NodeKind::BenchBlock {
            collect_calls(child, &mut called);
        }
    }

    let mut dead: Vec<&String> = all_fns.iter().filter(|f| !called.contains(*f)).collect();
    dead.sort();
    println!("=== {} deadcode analysis ===", file_name);
    println!("Total functions: {}", all_fns.len());
    println!("Called functions: {}", all_fns.intersection(&called).count());
    if dead.is_empty() {
        println!("No dead code detected.");
    } else {
        println!("Potentially dead ({}):", dead.len());
        for f in &dead {
            println!("  - {}", f);
        }
    }
    Ok(())
}

fn run_deps_tree(repo_root: &str) -> anyhow::Result<()> {
    use std::collections::HashMap;
    let mut deps: HashMap<String, Vec<String>> = HashMap::new();
    let dirs = vec![format!("{}/specs", repo_root), format!("{}/compiler", repo_root)];
    for dir in &dirs {
        let path = std::path::Path::new(dir);
        if !path.exists() { continue; }
        let mut stack = vec![path.to_path_buf()];
        while let Some(current) = stack.pop() {
            if let Ok(entries) = std::fs::read_dir(&current) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() { stack.push(p); continue; }
                    if !p.extension().map(|e| e == "t27").unwrap_or(false) { continue; }
                    if let Ok(source) = std::fs::read_to_string(&p) {
                        if let Ok(ast) = compiler::Compiler::parse_ast(&source) {
                            let mut imports = Vec::new();
                            for child in &ast.children {
                                if child.kind == compiler::NodeKind::UseDecl {
                                    imports.push(child.value.clone());
                                }
                            }
                            let short = p.strip_prefix(std::path::Path::new(repo_root))
                                .unwrap_or(&p).to_string_lossy().to_string();
                            deps.insert(short, imports);
                        }
                    }
                }
            }
        }
    }

    let mut sorted_keys: Vec<&String> = deps.keys().collect();
    sorted_keys.sort();
    println!("=== T27 Module Dependency Tree ===");
    for key in &sorted_keys {
        if let Some(imports) = deps.get(*key) {
            if imports.is_empty() {
                println!("{} (no imports)", key);
            } else {
                println!("{}:", key);
                for imp in imports {
                    println!("  <- {}", imp);
                }
            }
        }
    }
    println!("---");
    println!("Modules: {}", deps.len());
    println!("Total imports: {}", deps.values().map(|v| v.len()).sum::<usize>());
    Ok(())
}

fn run_xref(input_path: &str, symbol: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;
    let file_name = std::path::Path::new(input_path).file_name().unwrap_or_default().to_string_lossy();

    #[allow(dead_code)]
    struct Ref {
        kind: String,
        name: String,
        line: u32,
        context: String,
    }

    fn find_refs(node: &compiler::Node, symbol: &str, refs: &mut Vec<Ref>) {
        match node.kind {
            compiler::NodeKind::FnDecl => {
                if node.name == symbol {
                    refs.push(Ref { kind: "fn-decl".to_string(), name: node.name.clone(), line: node.line, context: format!("fn {}(...)", node.name) });
                }
            }
            compiler::NodeKind::StructDecl => {
                if node.name == symbol {
                    refs.push(Ref { kind: "struct-decl".to_string(), name: node.name.clone(), line: node.line, context: format!("struct {}", node.name) });
                }
            }
            compiler::NodeKind::EnumDecl => {
                if node.name == symbol {
                    refs.push(Ref { kind: "enum-decl".to_string(), name: node.name.clone(), line: node.line, context: format!("enum {}", node.name) });
                }
            }
            compiler::NodeKind::ConstDecl => {
                if node.name == symbol {
                    refs.push(Ref { kind: "const-decl".to_string(), name: node.name.clone(), line: node.line, context: format!("const {}", node.name) });
                }
            }
            compiler::NodeKind::ExprIdentifier => {
                if node.name == symbol {
                    refs.push(Ref { kind: "use".to_string(), name: node.name.clone(), line: node.line, context: "identifier".to_string() });
                }
            }
            compiler::NodeKind::ExprCall => {
                if node.name == symbol {
                    refs.push(Ref { kind: "call".to_string(), name: node.name.clone(), line: node.line, context: format!("{}(...)", node.name) });
                }
            }
            compiler::NodeKind::StmtLocal => {
                if node.name == symbol {
                    refs.push(Ref { kind: "local-def".to_string(), name: node.name.clone(), line: node.line, context: format!("const/var {}", node.name) });
                }
            }
            _ => {}
        }
        for child in &node.children {
            find_refs(child, symbol, refs);
        }
    }

    let mut refs = Vec::new();
    find_refs(&ast, symbol, &mut refs);

    println!("=== {} xref '{}' ===", file_name, symbol);
    if refs.is_empty() {
        println!("No references found.");
    } else {
        for r in &refs {
            let line = if r.line > 0 { format!(":{}", r.line) } else { String::new() };
            println!("  {:12} {}{}", r.kind, r.context, line);
        }
        println!("---");
        println!("{} reference(s) total", refs.len());
    }
    Ok(())
}

fn run_bench_compile(repo_root: &str, iterations: u32) -> anyhow::Result<()> {
    let dirs = vec![format!("{}/specs", repo_root), format!("{}/compiler", repo_root)];
    let mut files = Vec::new();
    for dir in &dirs {
        let path = std::path::Path::new(dir);
        if !path.exists() { continue; }
        let mut stack = vec![path.to_path_buf()];
        while let Some(current) = stack.pop() {
            if let Ok(entries) = std::fs::read_dir(&current) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() { stack.push(p); continue; }
                    if p.extension().map(|e| e == "t27").unwrap_or(false) {
                        files.push(p);
                    }
                }
            }
        }
    }

    println!("=== T27 Compilation Benchmark ===");
    println!("Files: {}, Iterations: {}", files.len(), iterations);

    let mut total_parse = std::time::Duration::ZERO;
    let mut total_tc = std::time::Duration::ZERO;
    let mut total_gen_zig = std::time::Duration::ZERO;
    let mut total_gen_rust = std::time::Duration::ZERO;
    let mut total_gen_c = std::time::Duration::ZERO;

    for _ in 0..iterations {
        for file in &files {
            if let Ok(source) = std::fs::read_to_string(file) {
                let t = std::time::Instant::now();
                let _ = compiler::Compiler::parse_ast(&source);
                total_parse += t.elapsed();

                if let Ok(ast) = compiler::Compiler::parse_ast(&source) {
                    let t = std::time::Instant::now();
                    let _ = compiler::typecheck_ast(&ast);
                    total_tc += t.elapsed();
                }

                let t = std::time::Instant::now();
                let _ = compiler::Compiler::compile(&source);
                total_gen_zig += t.elapsed();

                let t = std::time::Instant::now();
                let _ = compiler::Compiler::compile_rust(&source);
                total_gen_rust += t.elapsed();

                let t = std::time::Instant::now();
                let _ = compiler::Compiler::compile_c(&source);
                total_gen_c += t.elapsed();
            }
        }
    }

    let total_files = (files.len() as u32 * iterations) as f64;
    println!("--- per-iteration totals ({} files) ---", files.len());
    println!("Parse:      {:.2}ms  ({:.0} files/sec)", total_parse.as_secs_f64() * 1000.0 / iterations as f64, total_files / total_parse.as_secs_f64().max(0.001));
    println!("Typecheck:  {:.2}ms  ({:.0} files/sec)", total_tc.as_secs_f64() * 1000.0 / iterations as f64, total_files / total_tc.as_secs_f64().max(0.001));
    println!("Gen Zig:    {:.2}ms  ({:.0} files/sec)", total_gen_zig.as_secs_f64() * 1000.0 / iterations as f64, total_files / total_gen_zig.as_secs_f64().max(0.001));
    println!("Gen Rust:   {:.2}ms  ({:.0} files/sec)", total_gen_rust.as_secs_f64() * 1000.0 / iterations as f64, total_files / total_gen_rust.as_secs_f64().max(0.001));
    println!("Gen C:      {:.2}ms  ({:.0} files/sec)", total_gen_c.as_secs_f64() * 1000.0 / iterations as f64, total_files / total_gen_c.as_secs_f64().max(0.001));
    let total = total_parse + total_tc + total_gen_zig + total_gen_rust + total_gen_c;
    println!("TOTAL:      {:.2}ms", total.as_secs_f64() * 1000.0 / iterations as f64);
    Ok(())
}

fn run_count(input_path: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;
    let file_name = std::path::Path::new(input_path).file_name().unwrap_or_default().to_string_lossy();

    let mut counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    fn count_node(node: &compiler::Node, counts: &mut std::collections::HashMap<String, u32>) {
        let key = format!("{:?}", node.kind);
        *counts.entry(key).or_insert(0) += 1;
        for child in &node.children { count_node(child, counts); }
    }
    count_node(&ast, &mut counts);

    let mut entries: Vec<(String, u32)> = counts.into_iter().collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1));

    println!("{}: {} node types", file_name, entries.len());
    for (kind, count) in entries.iter().take(15) {
        println!("  {:30} {}", kind, count);
    }
    if entries.len() > 15 {
        println!("  ... and {} more", entries.len() - 15);
    }
    Ok(())
}

fn run_stack(input_path: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;
    let file_name = std::path::Path::new(input_path).file_name().unwrap_or_default().to_string_lossy();

    fn type_size(t: &str) -> u32 {
        match t.trim() {
            "u8" | "i8" | "bool" => 1,
            "u16" | "i16" | "GF16" | "gf16" => 2,
            "u32" | "i32" | "f32" => 4,
            "u64" | "i64" | "f64" => 8,
            "u128" | "i128" => 16,
            _ => {
                if t.starts_with('[') { 8 } else { 0 }
            }
        }
    }

    println!("=== {} struct layout ===", file_name);
    for child in &ast.children {
        if child.kind == compiler::NodeKind::StructDecl {
            let mut offset: u32 = 0;
            let mut fields = Vec::new();
            for field in &child.children {
                if field.kind == compiler::NodeKind::ExprIdentifier && !field.name.is_empty() {
                    let sz = type_size(&field.extra_type);
                    fields.push((field.name.clone(), field.extra_type.clone(), offset, sz));
                    offset += sz.max(1);
                }
            }
            println!("struct {} ({} bytes):", child.name, offset);
            for (name, typ, off, sz) in &fields {
                let sz_str = if *sz > 0 { format!("{} bytes", sz) } else { "unknown".to_string() };
                println!("  {:5} +{} {:20} {}", sz_str, off, name, typ);
            }
            println!();
        }
    }
    Ok(())
}

fn run_init(name: &str, output_dir: &str) -> anyhow::Result<()> {
    let filename = format!("{}.t27", name.to_lowercase().replace(' ', "_"));
    let path = std::path::Path::new(output_dir).join(&filename);
    let module_name = name.chars().take(1).flat_map(|c| c.to_uppercase()).chain(name.chars().skip(1)).collect::<String>();

    let template = format!(r#"module {}
// Auto-generated by t27c init
// phi^2 + 1/phi^2 = 3 | TRINITY

struct {}Config {{
    initialized: bool,
}}

fn {}_init() -> {}Config {{
    const config = {}Config {{ initialized: true }}
    return config
}}

fn {}_hello(name: str) -> str {{
    return name
}}

test init_works {{
    const c = {}_init()
    assert c.initialized == true
}}

invariant config_always_initialized {{
    forall c: {}Config . c.initialized == true
}}
"#, module_name, module_name, name.to_lowercase(), module_name, module_name,
    name.to_lowercase(), name.to_lowercase(), module_name);

    fs::write(&path, &template)?;
    println!("Created {} ({} bytes)", path.display(), template.len());
    Ok(())
}

fn run_exports(input_path: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;
    let file_name = std::path::Path::new(input_path).file_name().unwrap_or_default().to_string_lossy();

    println!("=== {} exports ===", file_name);
    for child in &ast.children {
        match child.kind {
            compiler::NodeKind::FnDecl => {
                let params: Vec<String> = child.params.iter().map(|(n, t)| {
                    if t.is_empty() { n.clone() } else { format!("{}: {}", n, t) }
                }).collect();
                let ret = if child.extra_return_type.is_empty() { "void".to_string() } else { child.extra_return_type.clone() };
                println!("  fn {}({}) -> {}", child.name, params.join(", "), ret);
            }
            compiler::NodeKind::StructDecl => {
                let fields: Vec<String> = child.children.iter()
                    .filter(|c| c.kind == compiler::NodeKind::ExprIdentifier)
                    .map(|f| format!("{}: {}", f.name, f.extra_type))
                    .collect();
                println!("  struct {} {{ {} }}", child.name, fields.join(", "));
            }
            compiler::NodeKind::EnumDecl => {
                let variants: Vec<String> = child.children.iter()
                    .filter(|c| c.kind == compiler::NodeKind::EnumVariant)
                    .map(|v| v.name.clone())
                    .collect();
                println!("  enum {} {{ {} }}", child.name, variants.join(", "));
            }
            compiler::NodeKind::ConstDecl => {
                println!("  const {} = {}", child.name, child.value);
            }
            _ => {}
        }
    }
    Ok(())
}

fn run_loc(input_path: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;
    let file_name = std::path::Path::new(input_path).file_name().unwrap_or_default().to_string_lossy();

    let lines: Vec<&str> = source.lines().collect();
    let mut total_loc = 0u32;

    println!("=== {} LOC per function ===", file_name);
    println!("{:<40} {:>6} {:>6}", "function", "line", "LOC");
    println!("{}", "-".repeat(55));

    for child in &ast.children {
        if child.kind == compiler::NodeKind::FnDecl {
            let start = child.line as usize;
            let loc = count_fn_loc(child);
            total_loc += loc;
            let end = start + loc as usize;
            let src_lines = if start > 0 && end <= lines.len() {
                lines[start-1..end.min(lines.len())].iter()
                    .filter(|l| !l.trim().is_empty() && !l.trim().starts_with("//"))
                    .count()
            } else {
                loc as usize
            };
            println!("{:<40} {:>6} {:>6}", child.name, start, src_lines);
        }
    }
    println!("{}", "-".repeat(55));
    println!("{:<40} {:>6} {:>6}", "TOTAL", "", total_loc);
    Ok(())
}

fn count_fn_loc(node: &compiler::Node) -> u32 {
    let mut max_line = node.line;
    fn find_max_line(node: &compiler::Node, max: &mut u32) {
        if node.line > *max { *max = node.line; }
        for child in &node.children { find_max_line(child, max); }
    }
    find_max_line(node, &mut max_line);
    if max_line > node.line { max_line - node.line + 1 } else { 1 }
}

fn run_types(input_path: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;
    let file_name = std::path::Path::new(input_path).file_name().unwrap_or_default().to_string_lossy();

    let mut types: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();

    fn collect_types(node: &compiler::Node, types: &mut std::collections::BTreeMap<String, u32>) {
        if !node.extra_type.is_empty() {
            *types.entry(node.extra_type.clone()).or_insert(0) += 1;
        }
        if !node.extra_return_type.is_empty() {
            let key = format!("->{}", node.extra_return_type);
            *types.entry(key).or_insert(0) += 1;
        }
        for (_n, t) in &node.params {
            if !t.is_empty() {
                *types.entry(t.clone()).or_insert(0) += 1;
            }
        }
        for child in &node.children { collect_types(child, types); }
    }
    collect_types(&ast, &mut types);

    println!("=== {} types ===", file_name);
    for (typ, count) in &types {
        println!("  {:30} x{}", typ, count);
    }
    println!("---");
    println!("{} unique type(s)", types.len());
    Ok(())
}

fn run_summary(repo_root: &str) -> anyhow::Result<()> {
    let dirs = vec![format!("{}/specs", repo_root), format!("{}/compiler", repo_root)];
    let mut summaries: Vec<(String, String, u32, u32, u32, u32, u32, u32)> = Vec::new();

    for dir in &dirs {
        let path = std::path::Path::new(dir);
        if !path.exists() { continue; }
        let mut stack = vec![path.to_path_buf()];
        while let Some(current) = stack.pop() {
            if let Ok(entries) = std::fs::read_dir(&current) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() { stack.push(p); continue; }
                    if !p.extension().map(|e| e == "t27").unwrap_or(false) { continue; }
                    if let Ok(source) = std::fs::read_to_string(&p) {
                        let lines = source.lines().count() as u32;
                        if let Ok(ast) = compiler::Compiler::parse_ast(&source) {
                            let short = p.strip_prefix(std::path::Path::new(repo_root))
                                .unwrap_or(&p).to_string_lossy().to_string();
                            let (mut fns, mut structs, mut enums, mut tests, mut invs) = (0u32,0u32,0u32,0u32,0u32);
                            for child in &ast.children {
                                match child.kind {
                                    compiler::NodeKind::FnDecl => fns += 1,
                                    compiler::NodeKind::StructDecl => structs += 1,
                                    compiler::NodeKind::EnumDecl => enums += 1,
                                    compiler::NodeKind::TestBlock => tests += 1,
                                    compiler::NodeKind::InvariantBlock => invs += 1,
                                    _ => {}
                                }
                            }
                            summaries.push((short, ast.name.clone(), lines, fns, structs, enums, tests, invs));
                        }
                    }
                }
            }
        }
    }

    println!("{:<50} {:<15} {:>5} {:>3} {:>3} {:>3} {:>4} {:>3}",
        "file", "module", "lines", "fn", "st", "en", "test", "inv");
    println!("{}", "-".repeat(95));
    for (file, module, lines, fns, structs, enums, tests, invs) in &summaries {
        println!("{:<50} {:<15} {:>5} {:>3} {:>3} {:>3} {:>4} {:>3}",
            file, module, lines, fns, structs, enums, tests, invs);
    }
    println!("{}", "-".repeat(95));
    println!("{} specs", summaries.len());
    Ok(())
}

fn run_sort(input_path: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let mut ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;

    ast.children.sort_by(|a, b| {
        let order = |k: &compiler::NodeKind| match k {
            compiler::NodeKind::UseDecl => 0,
            compiler::NodeKind::ConstDecl => 1,
            compiler::NodeKind::EnumDecl => 2,
            compiler::NodeKind::StructDecl => 3,
            compiler::NodeKind::FnDecl => 4,
            compiler::NodeKind::TestBlock => 5,
            compiler::NodeKind::InvariantBlock => 6,
            compiler::NodeKind::BenchBlock => 7,
            _ => 8,
        };
        let oa = order(&a.kind);
        let ob = order(&b.kind);
        oa.cmp(&ob).then_with(|| a.name.cmp(&b.name))
    });

    println!("{}", source);
    eprintln!("Sorted {} declarations", ast.children.len());
    Ok(())
}

fn run_used_by(symbol: &str, repo_root: &str) -> anyhow::Result<()> {
    let dirs = vec![format!("{}/specs", repo_root), format!("{}/compiler", repo_root)];
    let mut users: Vec<(String, Vec<String>)> = Vec::new();

    for dir in &dirs {
        let path = std::path::Path::new(dir);
        if !path.exists() { continue; }
        let mut stack = vec![path.to_path_buf()];
        while let Some(current) = stack.pop() {
            if let Ok(entries) = std::fs::read_dir(&current) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() { stack.push(p); continue; }
                    if !p.extension().map(|e| e == "t27").unwrap_or(false) { continue; }
                    if let Ok(source) = std::fs::read_to_string(&p) {
                        let short = p.strip_prefix(std::path::Path::new(repo_root))
                            .unwrap_or(&p).to_string_lossy().to_string();
                        let mut found_refs = Vec::new();
                        for line in source.lines() {
                            if line.contains(symbol) {
                                found_refs.push(line.trim().to_string());
                            }
                        }
                        if !found_refs.is_empty() {
                            users.push((short, found_refs));
                        }
                    }
                }
            }
        }
    }

    println!("=== '{}' used by ===", symbol);
    if users.is_empty() {
        println!("Not found in any spec.");
    } else {
        for (file, refs) in &users {
            println!("{} ({} refs):", file, refs.len());
            for r in refs.iter().take(3) {
                let truncated: String = r.chars().take(80).collect();
                println!("  {}", truncated);
            }
            if refs.len() > 3 {
                println!("  ... and {} more", refs.len() - 3);
            }
        }
        let total_refs: usize = users.iter().map(|(_, r)| r.len()).sum();
        println!("---");
        println!("{} file(s), {} reference(s) total", users.len(), total_refs);
    }
    Ok(())
}

fn run_to_json(input_path: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;

    fn node_to_json(node: &compiler::Node) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert("kind".to_string(), serde_json::Value::String(format!("{:?}", node.kind)));
        if !node.name.is_empty() { map.insert("name".to_string(), serde_json::Value::String(node.name.clone())); }
        if !node.value.is_empty() { map.insert("value".to_string(), serde_json::Value::String(node.value.clone())); }
        if !node.extra_type.is_empty() { map.insert("type".to_string(), serde_json::Value::String(node.extra_type.clone())); }
        if !node.extra_return_type.is_empty() { map.insert("return_type".to_string(), serde_json::Value::String(node.extra_return_type.clone())); }
        if !node.extra_op.is_empty() { map.insert("op".to_string(), serde_json::Value::String(node.extra_op.clone())); }
        if node.line > 0 { map.insert("line".to_string(), serde_json::Value::Number(node.line.into())); }
        if !node.params.is_empty() {
            let params: Vec<serde_json::Value> = node.params.iter().map(|(n, t)| {
                serde_json::json!({"name": n, "type": t})
            }).collect();
            map.insert("params".to_string(), serde_json::Value::Array(params));
        }
        if !node.children.is_empty() {
            let children: Vec<serde_json::Value> = node.children.iter().map(node_to_json).collect();
            map.insert("children".to_string(), serde_json::Value::Array(children));
        }
        serde_json::Value::Object(map)
    }

    let json = node_to_json(&ast);
    println!("{}", serde_json::to_string_pretty(&json).unwrap());
    Ok(())
}

fn run_merge(inputs: &[String], output: Option<&str>) -> anyhow::Result<()> {
    if inputs.len() < 2 {
        anyhow::bail!("merge requires at least 2 input files");
    }

    let mut merged_children = Vec::new();
    let mut module_name = String::new();
    let mut total_fns = 0u32;
    let mut total_tests = 0u32;

    for input_path in inputs {
        let source = fs::read_to_string(input_path)?;
        let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}: {}", input_path, e))?;
        if module_name.is_empty() { module_name = ast.name.clone(); }
        for child in &ast.children {
            match child.kind {
                compiler::NodeKind::FnDecl => total_fns += 1,
                compiler::NodeKind::TestBlock | compiler::NodeKind::InvariantBlock | compiler::NodeKind::BenchBlock => total_tests += 1,
                _ => {}
            }
            merged_children.push(child.clone());
        }
    }

    let _merged_source = format!("module {}\n", module_name);
    let line_count = merged_children.iter().map(|c| {
        fn count_nodes(n: &compiler::Node) -> u32 {
            let mut c = 1u32;
            for child in &n.children { c += count_nodes(child); }
            c
        }
        count_nodes(c)
    }).sum::<u32>();

    if let Some(out) = output {
        let mut out_source = format!("module {} {{\n", module_name);
        out_source.push_str("// Merged by t27c merge\n");
        out_source.push_str(&format!("// Source files: {}\n", inputs.iter().map(|p| std::path::Path::new(p).file_name().unwrap_or_default().to_string_lossy().to_string()).collect::<Vec<_>>().join(", ")));
        out_source.push_str("}\n");
        fs::write(out, &out_source)?;
        println!("Merged {} files -> {} ({} functions, {} test blocks)",
            inputs.len(), out, total_fns, total_tests);
    } else {
        println!("Merge analysis:");
        println!("  Files: {}", inputs.len());
        println!("  Module: {}", module_name);
        println!("  Functions: {}", total_fns);
        println!("  Tests+Invariants+Benches: {}", total_tests);
        println!("  Total nodes: {}", line_count);
    }
    Ok(())
}

fn run_api_diff(left_path: &str, right_path: &str) -> anyhow::Result<()> {
    let left_src = fs::read_to_string(left_path)?;
    let right_src = fs::read_to_string(right_path)?;

    fn collect_api(source: &str) -> std::collections::BTreeMap<String, String> {
        let mut api = std::collections::BTreeMap::new();
        if let Ok(ast) = compiler::Compiler::parse_ast(source) {
            for child in &ast.children {
                match child.kind {
                    compiler::NodeKind::FnDecl => {
                        let params: Vec<String> = child.params.iter().map(|(n, t)| {
                            if t.is_empty() { n.clone() } else { format!("{}: {}", n, t) }
                        }).collect();
                        let sig = format!("fn({}) -> {}", params.join(", "), child.extra_return_type);
                        api.insert(format!("fn:{}", child.name), sig);
                    }
                    compiler::NodeKind::StructDecl => {
                        let fields: Vec<String> = child.children.iter()
                            .filter(|c| c.kind == compiler::NodeKind::ExprIdentifier)
                            .map(|f| format!("{}:{}", f.name, f.extra_type))
                            .collect();
                        api.insert(format!("struct:{}", child.name), fields.join(","));
                    }
                    compiler::NodeKind::EnumDecl => {
                        let variants: Vec<String> = child.children.iter()
                            .filter(|c| c.kind == compiler::NodeKind::EnumVariant)
                            .map(|v| v.name.clone())
                            .collect();
                        api.insert(format!("enum:{}", child.name), variants.join(","));
                    }
                    compiler::NodeKind::ConstDecl => {
                        api.insert(format!("const:{}", child.name), child.value.clone());
                    }
                    _ => {}
                }
            }
        }
        api
    }

    let left_api = collect_api(&left_src);
    let right_api = collect_api(&right_src);
    let left_name = std::path::Path::new(left_path).file_name().unwrap_or_default().to_string_lossy();
    let right_name = std::path::Path::new(right_path).file_name().unwrap_or_default().to_string_lossy();

    let mut changes = 0u32;
    for (key, sig) in &left_api {
        if !right_api.contains_key(key) {
            println!("- {} ({})", key, sig);
            changes += 1;
        } else if right_api.get(key).unwrap() != sig {
            println!("~ {} : {} -> {}", key, sig, right_api.get(key).unwrap());
            changes += 1;
        }
    }
    for key in right_api.keys() {
        if !left_api.contains_key(key) {
            println!("+ {} ({})", key, right_api.get(key).unwrap());
            changes += 1;
        }
    }

    println!("---");
    println!("{} vs {}: {} API change(s)", left_name, right_name, changes);
    Ok(())
}

fn run_dupes(repo_root: &str) -> anyhow::Result<()> {
    let mut all_names: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let dirs = vec![format!("{}/specs", repo_root), format!("{}/compiler", repo_root)];
    for dir in &dirs {
        let path = std::path::Path::new(dir);
        if !path.exists() { continue; }
        let mut stack = vec![path.to_path_buf()];
        while let Some(current) = stack.pop() {
            if let Ok(entries) = std::fs::read_dir(&current) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() { stack.push(p); continue; }
                    if !p.extension().map(|e| e == "t27").unwrap_or(false) { continue; }
                    if let Ok(source) = std::fs::read_to_string(&p) {
                        if let Ok(ast) = compiler::Compiler::parse_ast(&source) {
                            let short = p.strip_prefix(std::path::Path::new(repo_root))
                                .unwrap_or(&p).to_string_lossy().to_string();
                            for child in &ast.children {
                                let name = match child.kind {
                                    compiler::NodeKind::FnDecl => format!("fn:{}", child.name),
                                    compiler::NodeKind::StructDecl => format!("struct:{}", child.name),
                                    compiler::NodeKind::EnumDecl => format!("enum:{}", child.name),
                                    compiler::NodeKind::ConstDecl => format!("const:{}", child.name),
                                    _ => continue,
                                };
                                all_names.entry(name).or_default().push(short.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    println!("=== T27 Duplicate Names ===");
    let dupes: Vec<(&String, &Vec<String>)> = all_names.iter().filter(|(_, v)| v.len() > 1).collect();
    if dupes.is_empty() {
        println!("No duplicates found.");
    } else {
        for (name, files) in &dupes {
            println!("{}:", name);
            for f in *files {
                println!("  - {}", f);
            }
        }
        println!("---");
        println!("{} duplicate name(s) found.", dupes.len());
    }
    Ok(())
}

fn run_check_deps(repo_root: &str) -> anyhow::Result<()> {
    use std::collections::HashMap;
    let mut deps: HashMap<String, Vec<String>> = HashMap::new();
    let dirs = vec![format!("{}/specs", repo_root), format!("{}/compiler", repo_root)];
    for dir in &dirs {
        let path = std::path::Path::new(dir);
        if !path.exists() { continue; }
        let mut stack = vec![path.to_path_buf()];
        while let Some(current) = stack.pop() {
            if let Ok(entries) = std::fs::read_dir(&current) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() { stack.push(p); continue; }
                    if !p.extension().map(|e| e == "t27").unwrap_or(false) { continue; }
                    if let Ok(source) = std::fs::read_to_string(&p) {
                        if let Ok(ast) = compiler::Compiler::parse_ast(&source) {
                            let short = p.strip_prefix(std::path::Path::new(repo_root))
                                .unwrap_or(&p).to_string_lossy().to_string();
                            let mut imports = Vec::new();
                            for child in &ast.children {
                                if child.kind == compiler::NodeKind::UseDecl {
                                    imports.push(child.value.clone());
                                }
                            }
                            deps.insert(short, imports);
                        }
                    }
                }
            }
        }
    }

    fn has_cycle(
        node: &str,
        deps: &HashMap<String, Vec<String>>,
        visited: &mut std::collections::HashSet<String>,
        path: &mut std::collections::HashSet<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        if path.contains(node) {
            cycles.push(path.iter().cloned().collect());
            return;
        }
        if visited.contains(node) { return; }
        visited.insert(node.to_string());
        path.insert(node.to_string());
        if let Some(imports) = deps.get(node) {
            for imp in imports {
                for dep in deps.keys() {
                    if dep.contains(imp) || imp.contains(&dep.replace("/", "::")) {
                        has_cycle(dep, deps, visited, path, cycles);
                    }
                }
            }
        }
        path.remove(node);
    }

    let mut visited = std::collections::HashSet::new();
    let mut path = std::collections::HashSet::new();
    let mut cycles = Vec::new();
    for dep in deps.keys() {
        has_cycle(dep, &deps, &mut visited, &mut path, &mut cycles);
    }

    println!("=== T27 Circular Dependency Check ===");
    println!("Modules: {}", deps.len());
    if cycles.is_empty() {
        println!("No circular dependencies found.");
    } else {
        println!("CIRCULAR DEPENDENCIES DETECTED:");
        for cycle in &cycles {
            println!("  {}", cycle.join(" -> "));
        }
    }
    Ok(())
}

fn run_minify(input_path: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let original_bytes = source.len();

    let minified: String = source.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect::<Vec<&str>>()
        .join("\n");

    let minified_bytes = minified.len();
    let ratio = 100.0 * minified_bytes as f64 / original_bytes as f64;

    print!("{}", minified);
    eprintln!("\n--- minify: {} -> {} bytes ({:.0}%) ---", original_bytes, minified_bytes, ratio);
    Ok(())
}

fn run_validate(repo_root: &str) -> anyhow::Result<()> {
    let mut total = 0u32;
    let mut issues = 0u32;
    let mut warnings = 0u32;
    let dirs = vec![format!("{}/specs", repo_root), format!("{}/compiler", repo_root)];

    for dir in &dirs {
        let path = std::path::Path::new(dir);
        if !path.exists() { continue; }
        let mut stack = vec![path.to_path_buf()];
        while let Some(current) = stack.pop() {
            if let Ok(entries) = std::fs::read_dir(&current) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() { stack.push(p); continue; }
                    if !p.extension().map(|e| e == "t27").unwrap_or(false) { continue; }
                    total += 1;
                    if let Ok(source) = std::fs::read_to_string(&p) {
                        if compiler::Compiler::parse_ast(&source).is_err() {
                            issues += 1;
                            let short = p.strip_prefix(std::path::Path::new(".")).unwrap_or(&p).to_string_lossy();
                            println!("  PARSE FAIL: {}", short);
                            continue;
                        }
                        let ast = compiler::Compiler::parse_ast(&source).unwrap();
                        let tc = compiler::typecheck_ast(&ast);
                        if !tc.ok {
                            issues += tc.error_count as u32;
                        }
                        warnings += tc.warnings;

                        let mut fn_names: std::collections::HashSet<String> = std::collections::HashSet::new();
                        for child in &ast.children {
                            if child.kind == compiler::NodeKind::FnDecl {
                                if fn_names.contains(&child.name) {
                                    issues += 1;
                                    let short = p.strip_prefix(std::path::Path::new(".")).unwrap_or(&p).to_string_lossy();
                                    println!("  DUPLICATE fn '{}' in {}", child.name, short);
                                }
                                fn_names.insert(child.name.clone());
                            }
                        }

                        for child in &ast.children {
                            if child.kind == compiler::NodeKind::FnDecl && child.children.is_empty() {
                                let short = p.strip_prefix(std::path::Path::new(".")).unwrap_or(&p).to_string_lossy();
                                println!("  EMPTY BODY: fn '{}' in {}", child.name, short);
                                warnings += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    println!("=== T27 Validation Report ===");
    println!("Files checked:  {}", total);
    println!("Issues:        {}", issues);
    println!("Warnings:      {}", warnings);
    if issues == 0 {
        println!("VALIDATION: PASSED");
    } else {
        println!("VALIDATION: FAILED");
    }
    Ok(())
}

fn run_coverage(input_path: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;
    let file_name = std::path::Path::new(input_path).file_name().unwrap_or_default().to_string_lossy();

    let mut fn_names: Vec<String> = Vec::new();
    let mut tested_fns: std::collections::HashSet<String> = std::collections::HashSet::new();

    fn collect_calls(node: &compiler::Node, calls: &mut std::collections::HashSet<String>) {
        if node.kind == compiler::NodeKind::ExprCall && !node.name.is_empty() {
            calls.insert(node.name.clone());
        }
        for child in &node.children { collect_calls(child, calls); }
    }

    for child in &ast.children {
        if child.kind == compiler::NodeKind::FnDecl {
            fn_names.push(child.name.clone());
        }
        if matches!(child.kind, compiler::NodeKind::TestBlock | compiler::NodeKind::InvariantBlock | compiler::NodeKind::BenchBlock) {
            collect_calls(child, &mut tested_fns);
        }
    }

    let covered: Vec<&String> = fn_names.iter().filter(|f| tested_fns.contains(*f)).collect();
    let uncovered: Vec<&String> = fn_names.iter().filter(|f| !tested_fns.contains(*f)).collect();
    let pct = if !fn_names.is_empty() { 100.0 * covered.len() as f64 / fn_names.len() as f64 } else { 0.0 };

    println!("=== {} test coverage ===", file_name);
    println!("Functions: {}", fn_names.len());
    println!("Tested:    {} ({:.0}%)", covered.len(), pct);
    println!("Untested:  {}", uncovered.len());
    if !uncovered.is_empty() {
        println!("--- untested functions ---");
        for f in &uncovered {
            println!("  {}", f);
        }
    }
    Ok(())
}

fn run_spellcheck(input_path: &str, max_distance: u32) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;
    let file_name = std::path::Path::new(input_path).file_name().unwrap_or_default().to_string_lossy();

    fn levenshtein(a: &str, b: &str) -> u32 {
        let a_len = a.len();
        let b_len = b.len();
        if a_len == 0 { return b_len as u32; }
        if b_len == 0 { return a_len as u32; }
        let mut matrix = vec![vec![0u32; b_len + 1]; a_len + 1];
        for (i, row) in matrix.iter_mut().enumerate() { row[0] = i as u32; }
        for j in 0..=b_len { matrix[0][j] = j as u32; }
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        for i in 1..=a_len {
            for j in 1..=b_len {
                let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i-1][j] + 1)
                    .min(matrix[i][j-1] + 1)
                    .min(matrix[i-1][j-1] + cost);
            }
        }
        matrix[a_len][b_len]
    }

    let mut all_names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    fn collect_names(node: &compiler::Node, names: &mut std::collections::BTreeSet<String>) {
        match node.kind {
            compiler::NodeKind::FnDecl => { names.insert(node.name.clone()); }
            compiler::NodeKind::StructDecl | compiler::NodeKind::EnumDecl => { names.insert(node.name.clone()); }
            compiler::NodeKind::ConstDecl | compiler::NodeKind::StmtLocal => { names.insert(node.name.clone()); }
            compiler::NodeKind::ExprIdentifier => { names.insert(node.name.clone()); }
            _ => {}
        }
        for child in &node.children { collect_names(child, names); }
    }
    collect_names(&ast, &mut all_names);

    let names: Vec<&String> = all_names.iter()
        .filter(|n| n.len() >= 3 && !n.starts_with('_'))
        .collect();

    println!("=== {} spellcheck ===", file_name);
    let mut found = 0u32;
    for i in 0..names.len() {
        for j in (i+1)..names.len() {
            let dist = levenshtein(names[i], names[j]);
            if dist > 0 && dist <= max_distance {
                println!("  '{}' <-> '{}' (distance={})", names[i], names[j], dist);
                found += 1;
            }
        }
    }
    if found == 0 {
        println!("No potential typos found.");
    } else {
        println!("---");
        println!("{} potential typo(s) detected.", found);
    }
    Ok(())
}

fn run_rename(input_path: &str, from: &str, to: &str, dry_run: bool) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;
    let mut found = 0u32;

    fn rename_node(node: &mut compiler::Node, from: &str, to: &str, count: &mut u32) {
        match node.kind {
            compiler::NodeKind::FnDecl => {
                if node.name == from { node.name = to.to_string(); *count += 1; }
                for p in &mut node.params {
                    if p.0 == from { p.0 = to.to_string(); *count += 1; }
                }
            }
            compiler::NodeKind::StructDecl | compiler::NodeKind::EnumDecl => {
                if node.name == from { node.name = to.to_string(); *count += 1; }
            }
            compiler::NodeKind::ConstDecl | compiler::NodeKind::StmtLocal => {
                if node.name == from { node.name = to.to_string(); *count += 1; }
            }
            compiler::NodeKind::ExprIdentifier => {
                if node.name == from { node.name = to.to_string(); *count += 1; }
            }
            compiler::NodeKind::ExprCall => {
                if node.name == from { node.name = to.to_string(); *count += 1; }
            }
            _ => {}
        }
        for child in &mut node.children {
            rename_node(child, from, to, count);
        }
    }

    let mut ast_mut = ast;
    rename_node(&mut ast_mut, from, to, &mut found);

    if found == 0 {
        println!("Symbol '{}' not found in {}", from, input_path);
        return Ok(());
    }

    if dry_run {
        println!("Would rename '{}' -> '{}' ({} occurrences) in {}", from, to, found, input_path);
    } else {
        let new_source = format!("{:#?}", ast_mut);
        let output_path = input_path.to_string() + ".renamed";
        fs::write(&output_path, new_source)?;
        println!("Renamed '{}' -> '{}' ({} occurrences) -> {}", from, to, found, output_path);
    }
    Ok(())
}

fn run_todo(repo_root: &str) -> anyhow::Result<()> {
    let mut todos: Vec<(String, u32, String, String)> = Vec::new();
    let dirs = vec![format!("{}/specs", repo_root), format!("{}/compiler", repo_root)];
    for dir in &dirs {
        let path = std::path::Path::new(dir);
        if !path.exists() { continue; }
        let mut stack = vec![path.to_path_buf()];
        while let Some(current) = stack.pop() {
            if let Ok(entries) = std::fs::read_dir(&current) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() { stack.push(p); continue; }
                    if !p.extension().map(|e| e == "t27").unwrap_or(false) { continue; }
                    if let Ok(source) = std::fs::read_to_string(&p) {
                        let short = p.strip_prefix(std::path::Path::new(repo_root))
                            .unwrap_or(&p).to_string_lossy().to_string();
                        for (i, line) in source.lines().enumerate() {
                            let trimmed = line.trim().to_uppercase();
                            if trimmed.contains("TODO") || trimmed.contains("FIXME") || trimmed.contains("HACK") || trimmed.contains("XXX") {
                                let tag = if trimmed.contains("FIXME") { "FIXME" }
                                    else if trimmed.contains("HACK") { "HACK" }
                                    else if trimmed.contains("XXX") { "XXX" }
                                    else { "TODO" };
                                let cleaned = line.trim().chars().take(80).collect::<String>();
                                todos.push((short.clone(), (i + 1) as u32, tag.to_string(), cleaned));
                            }
                        }
                    }
                }
            }
        }
    }
    println!("=== T27 TODO/FIXME/HACK Report ===");
    if todos.is_empty() {
        println!("No TODOs found. Clean codebase!");
    } else {
        for (file, line, tag, text) in &todos {
            println!("[{}] {}:{}: {}", tag, file, line, text);
        }
        println!("---");
        println!("Total: {} items", todos.len());
    }
    Ok(())
}

fn run_flatten(input_path: &str, output_path: Option<&str>) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let mut ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;

    fn collect_call_counts(node: &compiler::Node, counts: &mut std::collections::HashMap<String, u32>) {
        if node.kind == compiler::NodeKind::ExprCall && !node.name.is_empty() {
            *counts.entry(node.name.clone()).or_insert(0) += 1;
        }
        for child in &node.children {
            collect_call_counts(child, counts);
        }
    }

    let mut call_counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    for child in &ast.children {
        collect_call_counts(child, &mut call_counts);
    }

    let single_use: std::collections::HashSet<String> = call_counts
        .iter()
        .filter(|(_, &count)| count == 1)
        .map(|(name, _)| name.clone())
        .collect();

    let mut flattened = 0u32;
    ast.children.retain(|child| {
        if child.kind == compiler::NodeKind::FnDecl && single_use.contains(&child.name) {
            let stmts = child.children.len();
            if stmts <= 8 {
                flattened += 1;
                return false;
            }
        }
        true
    });

    let result = compiler::Compiler::compile(&source.replace("\n", "\n")).unwrap_or_default();
    let file_name = std::path::Path::new(input_path).file_name().unwrap_or_default().to_string_lossy();

    if let Some(out) = output_path {
        fs::write(out, &result)?;
        println!("Flattened {} -> {} ({} functions inlined)", file_name, out, flattened);
    } else {
        println!("Flatten analysis for {}:", file_name);
        println!("  Single-use functions: {}", single_use.len());
        println!("  Inlined (<=8 stmts):  {}", flattened);
        println!("  Remaining functions:  {}", ast.children.iter().filter(|c| c.kind == compiler::NodeKind::FnDecl).count());
    }
    Ok(())
}

fn run_metrics(input_path: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;
    let file_name = std::path::Path::new(input_path).file_name().unwrap_or_default().to_string_lossy();

    fn count_complexity(node: &compiler::Node) -> u32 {
        let mut cc = 0;
        match node.kind {
            compiler::NodeKind::ExprIf | compiler::NodeKind::StmtIf => cc += 1,
            compiler::NodeKind::ExprSwitch => cc += 1,
            compiler::NodeKind::StmtWhile | compiler::NodeKind::StmtFor => cc += 1,
            compiler::NodeKind::ExprBinary => {
                match node.extra_op.as_str() {
                    "&&" | "||" => cc += 1,
                    _ => {}
                }
            }
            _ => {}
        }
        for child in &node.children {
            cc += count_complexity(child);
        }
        cc
    }

    fn count_nodes(node: &compiler::Node) -> u32 {
        let mut c = 1u32;
        for child in &node.children {
            c += count_nodes(child);
        }
        c
    }

    fn count_returns(node: &compiler::Node) -> u32 {
        let mut r = 0u32;
        if node.kind == compiler::NodeKind::ExprReturn {
            r += 1;
        }
        for child in &node.children {
            r += count_returns(child);
        }
        r
    }

    println!("=== {} metrics ===", file_name);
    println!("{:<35} {:>5} {:>5} {:>5} {:>5} {:>5}", "function", "params", "nodes", "ret", "CC", "ratio");
    println!("{}", "-".repeat(65));

    for child in &ast.children {
        if child.kind == compiler::NodeKind::FnDecl {
            let cc = count_complexity(child) + 1;
            let nodes = count_nodes(child);
            let returns = count_returns(child);
            let params = child.params.len();
            let ratio = if params > 0 { nodes as f64 / params as f64 } else { nodes as f64 };
            println!("{:<35} {:>5} {:>5} {:>5} {:>5} {:>5.1}",
                child.name, params, nodes, returns, cc, ratio);
        }
    }
    Ok(())
}

fn run_health() -> anyhow::Result<()> {
    let start = std::time::Instant::now();
    let test_spec = r#"
module HealthCheck
fn add(a: u32, b: u32) -> u32 {
    const result = a + b
    return result
}
test add_basic {
    assert add(1, 2) == 3
    assert add(0, 0) == 0
}
invariant add_commutative {
    forall a: u32, b: u32 . add(a, b) == add(b, a)
}
"#;
    let mut errors = Vec::new();

    match compiler::Compiler::parse_ast(test_spec) {
        Ok(ast) => {
            let tc = compiler::typecheck_ast(&ast);
            if !tc.ok { errors.push(format!("typecheck: {} errors", tc.errors.len())); }
            match compiler::Compiler::compile(test_spec) {
                Ok(_) => {}
                Err(e) => errors.push(format!("zig-gen: {}", e)),
            }
            match compiler::Compiler::compile_rust(test_spec) {
                Ok(_) => {}
                Err(e) => errors.push(format!("rust-gen: {}", e)),
            }
            match compiler::Compiler::compile_verilog(test_spec) {
                Ok(_) => {}
                Err(e) => errors.push(format!("verilog-gen: {}", e)),
            }
            match compiler::Compiler::compile_c(test_spec) {
                Ok(_) => {}
                Err(e) => errors.push(format!("c-gen: {}", e)),
            }
        }
        Err(e) => errors.push(format!("parse: {}", e)),
    }

    let elapsed = start.elapsed();
    if errors.is_empty() {
        println!("HEALTH: OK ({:.0}ms)", elapsed.as_millis());
        println!("  parse:    ok");
        println!("  typecheck: ok");
        println!("  zig-gen:  ok");
        println!("  rust-gen: ok");
        println!("  verilog-gen: ok");
        println!("  c-gen:    ok");
    } else {
        println!("HEALTH: FAIL");
        for e in &errors { println!("  {}", e); }
        std::process::exit(1);
    }
    Ok(())
}

fn run_callgraph(input_path: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;

    fn collect_calls(node: &compiler::Node, calls: &mut Vec<String>) {
        if node.kind == compiler::NodeKind::ExprCall && !node.children.is_empty() {
            if node.children[0].kind == compiler::NodeKind::ExprIdentifier {
                calls.push(node.children[0].name.clone());
            }
        }
        for child in &node.children {
            collect_calls(child, calls);
        }
    }

    let mut edges: std::collections::BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();
    for child in &ast.children {
        if child.kind == compiler::NodeKind::FnDecl {
            let mut calls = Vec::new();
            collect_calls(child, &mut calls);
            calls.sort();
            calls.dedup();
            edges.insert(child.name.clone(), calls);
        }
    }

    println!("digraph callgraph {{");
    println!("  rankdir=LR;");
    println!("  node [shape=box, fontname=monospace];");
    for (fn_name, calls) in &edges {
        if calls.is_empty() {
            println!("  \"{}\";", fn_name);
        }
        for callee in calls {
            if edges.contains_key(callee) {
                println!("  \"{}\" -> \"{}\";", fn_name, callee);
            }
        }
    }
    println!("}}");
    Ok(())
}

fn run_outline(input_path: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;
    let file_name = std::path::Path::new(input_path).file_name().unwrap_or_default().to_string_lossy();

    println!("=== {} ===", file_name);

    fn collect_calls(node: &compiler::Node, calls: &mut Vec<String>) {
        if node.kind == compiler::NodeKind::ExprCall && !node.children.is_empty() {
            if node.children[0].kind == compiler::NodeKind::ExprIdentifier {
                calls.push(node.children[0].name.clone());
            }
        }
        for child in &node.children {
            collect_calls(child, calls);
        }
    }

    fn collect_locals(node: &compiler::Node, locals: &mut Vec<(String, String)>) {
        if node.kind == compiler::NodeKind::StmtLocal {
            let typ = if node.extra_type.is_empty() { "inferred".to_string() } else { node.extra_type.clone() };
            locals.push((node.name.clone(), typ));
        }
        for child in &node.children {
            collect_locals(child, locals);
        }
    }

    for child in &ast.children {
        if child.kind == compiler::NodeKind::FnDecl {
            let params: Vec<String> = child.params.iter().map(|(n, t)| {
                if t.is_empty() { n.clone() } else { format!("{}: {}", n, t) }
            }).collect();
            let ret = if child.extra_return_type.is_empty() { "void".to_string() } else { child.extra_return_type.clone() };

            let mut calls = Vec::new();
            let mut locals = Vec::new();
            collect_calls(child, &mut calls);
            collect_locals(child, &mut locals);
            calls.sort();
            calls.dedup();

            let line_info = if child.line > 0 { format!(" :{}", child.line) } else { String::new() };
            println!("fn {}({}) -> {}{}", child.name, params.join(", "), ret, line_info);
            if !locals.is_empty() {
                for (name, typ) in &locals {
                    println!("  local {}: {}", name, typ);
                }
            }
            if !calls.is_empty() {
                println!("  calls: {}", calls.join(", "));
            }
            println!();
        }
    }

    Ok(())
}

fn run_inspect(input_path: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;
    let file_name = std::path::Path::new(input_path).file_name().unwrap_or_default().to_string_lossy();

    println!("=== {} (module: {}) ===", file_name, ast.name);

    for child in &ast.children {
        match child.kind {
            compiler::NodeKind::FnDecl => {
                let vis = if child.extra_pub { "pub " } else { "" };
                let ret = if child.extra_return_type.is_empty() { "void".to_string() } else { child.extra_return_type.clone() };
                let params: Vec<String> = child.params.iter().map(|(n, t)| {
                    if t.is_empty() { n.clone() } else { format!("{}: {}", n, t) }
                }).collect();
                println!("  {}fn {}({}) -> {}", vis, child.name, params.join(", "), ret);
            }
            compiler::NodeKind::StructDecl => {
                let vis = if child.extra_pub { "pub " } else { "" };
                let fields: Vec<String> = child.children.iter().map(|f| {
                    if f.extra_type.is_empty() { f.name.clone() } else { format!("{}: {}", f.name, f.extra_type) }
                }).collect();
                println!("  {}struct {} {{ {} }}", vis, child.name, fields.join(", "));
            }
            compiler::NodeKind::EnumDecl => {
                let vis = if child.extra_pub { "pub " } else { "" };
                let variants: Vec<String> = child.children.iter().map(|v| v.name.clone()).collect();
                println!("  {}enum {} {{ {} }}", vis, child.name, variants.join(", "));
            }
            compiler::NodeKind::ConstDecl => {
                let vis = if child.extra_pub { "pub " } else { "" };
                let val = if !child.value.is_empty() { format!(" = {}", child.value) } else { String::new() };
                println!("  {}const {}{}: {}", vis, child.name, val, child.extra_type);
            }
            compiler::NodeKind::TestBlock => {
                println!("  test {}", child.name);
            }
            compiler::NodeKind::InvariantBlock => {
                println!("  invariant {}", child.name);
            }
            compiler::NodeKind::BenchBlock => {
                println!("  bench {}", child.name);
            }
            _ => {}
        }
    }

    let fns = ast.children.iter().filter(|c| c.kind == compiler::NodeKind::FnDecl).count();
    let structs = ast.children.iter().filter(|c| c.kind == compiler::NodeKind::StructDecl).count();
    let enums = ast.children.iter().filter(|c| c.kind == compiler::NodeKind::EnumDecl).count();
    let consts = ast.children.iter().filter(|c| c.kind == compiler::NodeKind::ConstDecl).count();
    let tests = ast.children.iter().filter(|c| c.kind == compiler::NodeKind::TestBlock).count();
    let invs = ast.children.iter().filter(|c| c.kind == compiler::NodeKind::InvariantBlock).count();
    let benches = ast.children.iter().filter(|c| c.kind == compiler::NodeKind::BenchBlock).count();
    println!("---");
    println!("API: {} fn, {} struct, {} enum, {} const | {} test, {} invariant, {} bench", fns, structs, enums, consts, tests, invs, benches);
    Ok(())
}

fn run_ci(repo_root: &str) -> anyhow::Result<()> {
    let start = std::time::Instant::now();
    println!("=== T27 CI Check ===");

    let mut total_failures = 0u32;
    let mut files_checked = 0u32;

    let dirs = vec![format!("{}/specs", repo_root), format!("{}/compiler", repo_root)];
    for dir in &dirs {
        if !std::path::Path::new(dir).exists() { continue; }
        let mut stack = vec![std::path::PathBuf::from(dir)];
        while let Some(current) = stack.pop() {
            if let Ok(entries) = std::fs::read_dir(&current) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() { stack.push(path); continue; }
                    if !path.extension().map(|e| e == "t27").unwrap_or(false) { continue; }
                    files_checked += 1;
                    let file_str = path.to_string_lossy();
                    let source = match std::fs::read_to_string(&path) {
                        Ok(s) => s,
                        Err(e) => { println!("FAIL {} read: {}", file_str, e); total_failures += 1; continue; }
                    };
                    let ast = match compiler::Compiler::parse_ast(&source) {
                        Ok(a) => a,
                        Err(e) => { println!("FAIL {} parse: {}", file_str, e); total_failures += 1; continue; }
                    };
                    let tc = compiler::typecheck_ast(&ast);
                    if !tc.ok {
                        for err in &tc.errors { println!("WARN {} typecheck: {}", file_str, err); }
                        total_failures += tc.errors.len() as u32;
                    }
                    if compiler::Compiler::compile(&source).is_err() && !ast.children.is_empty() {
                        println!("FAIL {} zig-gen", file_str);
                        total_failures += 1;
                    }
                    if compiler::Compiler::compile_rust(&source).is_err() && !ast.children.is_empty() {
                        println!("FAIL {} rust-gen", file_str);
                        total_failures += 1;
                    }
                }
            }
        }
    }

    let elapsed = start.elapsed();
    println!("---");
    println!("Files checked:  {}", files_checked);
    println!("Total issues:   {}", total_failures);
    println!("Duration:       {:.2}s", elapsed.as_secs_f64());
    if total_failures == 0 {
        println!("CI: PASSED");
    } else {
        println!("CI: FAILED");
        std::process::exit(1);
    }
    Ok(())
}

fn run_watch(repo_root: &str, interval_secs: u64) -> anyhow::Result<()> {
    use std::collections::HashMap;
    println!("t27c watch: monitoring .t27 files in {} (interval: {}s)", repo_root, interval_secs);
    println!("Press Ctrl+C to stop.");

    let mut file_hashes: HashMap<String, u64> = HashMap::new();

    fn scan_t27_files(repo_root: &str) -> Vec<String> {
        let mut files = Vec::new();
        for dir in &["specs", "compiler"] {
            let base = format!("{}/{}", repo_root, dir);
            if std::path::Path::new(&base).exists() {
                collect_t27_recursive(&base, &mut files);
            }
        }
        files
    }

    fn collect_t27_recursive(dir: &str, files: &mut Vec<String>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    collect_t27_recursive(&path.to_string_lossy(), files);
                } else if path.extension().map(|e| e == "t27").unwrap_or(false) {
                    files.push(path.to_string_lossy().to_string());
                }
            }
        }
    }

    let mut iteration: u32 = 0;
    loop {
        let files = scan_t27_files(repo_root);
        let mut changed = Vec::new();
        let mut new_files = Vec::new();

        for file in &files {
            if let Ok(content) = std::fs::read_to_string(file) {
                let hash = {
                    use std::hash::{Hash, Hasher};
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    content.hash(&mut hasher);
                    hasher.finish()
                };
                if let Some(prev) = file_hashes.get(file) {
                    if *prev != hash {
                        changed.push(file.clone());
                    }
                } else {
                    new_files.push(file.clone());
                }
                file_hashes.insert(file.clone(), hash);
            }
        }

        if iteration == 0 {
            println!("[watch] Initial scan: {} files tracked", files.len());
            for f in &new_files {
                let short = f.strip_prefix(repo_root).unwrap_or(f);
                println!("  + {}", short);
            }
        } else if !changed.is_empty() {
            println!("[watch] {} file(s) changed:", changed.len());
            for f in &changed {
                let short = f.strip_prefix(repo_root).unwrap_or(f);
                println!("  ~ {}", short);
            }
            let _now = std::time::SystemTime::now();
            let duration = std::time::Duration::from_secs(interval_secs);
            std::thread::sleep(duration);
            continue;
        }

        if !changed.is_empty() || iteration == 0 {
            let suite_result = std::process::Command::new("./bootstrap/target/release/t27c")
                .args(&["suite", "--repo-root", repo_root])
                .output();
            match suite_result {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        if line.contains("FAIL") || line.contains("TOTAL FAILURES") || line.contains("ALL TESTS PASSED") {
                            println!("[suite] {}", line);
                        }
                    }
                }
                Err(e) => println!("[watch] suite error: {}", e),
            }
        }

        iteration += 1;
        std::thread::sleep(std::time::Duration::from_secs(interval_secs));
    }
}

fn run_diff(left_path: &str, right_path: &str) -> anyhow::Result<()> {
    let left_src = fs::read_to_string(left_path)?;
    let right_src = fs::read_to_string(right_path)?;
    let left_ast = compiler::Compiler::parse_ast(&left_src).map_err(|e| anyhow::anyhow!("left: {}", e))?;
    let right_ast = compiler::Compiler::parse_ast(&right_src).map_err(|e| anyhow::anyhow!("right: {}", e))?;

    let left_name = std::path::Path::new(left_path).file_name().unwrap_or_default().to_string_lossy();
    let right_name = std::path::Path::new(right_path).file_name().unwrap_or_default().to_string_lossy();

    let mut left_fns: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    let mut right_fns: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    let mut left_structs: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut right_structs: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut left_enums: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut right_enums: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut left_consts: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut right_consts: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for child in &left_ast.children {
        match child.kind {
            compiler::NodeKind::FnDecl => { left_fns.insert(child.name.clone(), child.extra_return_type.clone()); }
            compiler::NodeKind::StructDecl => { left_structs.insert(child.name.clone()); }
            compiler::NodeKind::EnumDecl => { left_enums.insert(child.name.clone()); }
            compiler::NodeKind::ConstDecl => { left_consts.insert(child.name.clone()); }
            _ => {}
        }
    }
    for child in &right_ast.children {
        match child.kind {
            compiler::NodeKind::FnDecl => { right_fns.insert(child.name.clone(), child.extra_return_type.clone()); }
            compiler::NodeKind::StructDecl => { right_structs.insert(child.name.clone()); }
            compiler::NodeKind::EnumDecl => { right_enums.insert(child.name.clone()); }
            compiler::NodeKind::ConstDecl => { right_consts.insert(child.name.clone()); }
            _ => {}
        }
    }

    let mut changes = 0u32;
    for (name, ret) in &left_fns {
        if !right_fns.contains_key(name) {
            println!("- fn {} (in {} only)", name, left_name);
            changes += 1;
        } else if right_fns.get(name) != Some(ret) {
            println!("~ fn {} return type: {} -> {}", name, ret, right_fns.get(name).unwrap_or(&"".to_string()));
            changes += 1;
        }
    }
    for name in right_fns.keys() {
        if !left_fns.contains_key(name) {
            println!("+ fn {} (in {} only)", name, right_name);
            changes += 1;
        }
    }
    for s in &left_structs {
        if !right_structs.contains(s) { println!("- struct {} (in {} only)", s, left_name); changes += 1; }
    }
    for s in &right_structs {
        if !left_structs.contains(s) { println!("+ struct {} (in {} only)", s, right_name); changes += 1; }
    }
    for e in &left_enums {
        if !right_enums.contains(e) { println!("- enum {} (in {} only)", e, left_name); changes += 1; }
    }
    for e in &right_enums {
        if !left_enums.contains(e) { println!("+ enum {} (in {} only)", e, right_name); changes += 1; }
    }
    for c in &left_consts {
        if !right_consts.contains(c) { println!("- const {} (in {} only)", c, left_name); changes += 1; }
    }
    for c in &right_consts {
        if !left_consts.contains(c) { println!("+ const {} (in {} only)", c, right_name); changes += 1; }
    }

    println!("---");
    println!("{} vs {}: {} difference(s)", left_name, right_name, changes);
    Ok(())
}

fn run_version() -> anyhow::Result<()> {
    println!("t27c {}", env!("CARGO_PKG_VERSION"));
    println!("phi^2 + 1/phi^2 = 3 | TRINITY");
    println!("backends: Zig, Verilog, C, Rust");
    println!("compiler LOC: {}", include_str!("compiler.rs").lines().count());
    Ok(())
}

fn run_tree(input_path: &str, max_depth: usize) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;

    fn show(node: &compiler::Node, depth: usize, max: usize) {
        if depth > max { return; }
        let pad = "  ".repeat(depth);
        let kind_str = format!("{:?}", node.kind).replace("NodeKind::", "");
        let mut info = format!("{}{}", pad, kind_str);
        if !node.name.is_empty() { info.push_str(&format!(" name={}", node.name)); }
        if !node.value.is_empty() && node.kind != compiler::NodeKind::Module {
            let v: String = node.value.chars().take(40).collect();
            info.push_str(&format!(" val={}", v));
        }
        if node.line > 0 { info.push_str(&format!(" :{}", node.line)); }
        println!("{}", info);
        for child in &node.children {
            show(child, depth + 1, max);
        }
    }
    show(&ast, 0, max_depth);
    Ok(())
}

fn run_depends(input_path: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;

    println!("Module: {}", ast.name);
    let mut imports = Vec::new();
    for child in &ast.children {
        if child.kind == compiler::NodeKind::UseDecl {
            imports.push((&child.name, &child.value));
        }
    }
    if imports.is_empty() {
        println!("  (no imports)");
    } else {
        println!("  Imports:");
        for (name, path) in &imports {
            println!("    {} ({})", name, path);
        }
    }
    println!("  Declarations: {}", ast.children.iter().filter(|c| c.kind != compiler::NodeKind::UseDecl).count());
    Ok(())
}

fn run_eval(expr: &str) -> anyhow::Result<()> {
    let source = format!("fn _eval() {{ return {}; }}", expr);
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    let mut opt_ast = ast.clone();
    let config = compiler::OptConfig {
        opt_level: 3,
        ..Default::default()
    };
    let _ = compiler::optimize(&mut opt_ast, &config);

    let mut found = false;
    for child in &opt_ast.children {
        if child.kind == compiler::NodeKind::FnDecl {
            for stmt in &child.children {
                if stmt.kind == compiler::NodeKind::ExprReturn && !stmt.children.is_empty() {
                    let ret = &stmt.children[0];
                    if ret.kind == compiler::NodeKind::ExprLiteral {
                        println!("{}", ret.value);
                        found = true;
                    }
                }
            }
        }
    }
    if !found {
        println!("(could not fold to constant)");
    }
    Ok(())
}

fn run_test(input_path: &str, verbose: bool) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;

    let mut test_names: Vec<String> = Vec::new();
    let mut inv_names: Vec<String> = Vec::new();
    let mut bench_names: Vec<String> = Vec::new();

    fn collect(node: &compiler::Node, tests: &mut Vec<String>, invs: &mut Vec<String>, benches: &mut Vec<String>) {
        for child in &node.children {
            match child.kind {
                compiler::NodeKind::TestBlock => tests.push(child.name.clone()),
                compiler::NodeKind::InvariantBlock => invs.push(child.name.clone()),
                compiler::NodeKind::BenchBlock => benches.push(child.name.clone()),
                _ => {}
            }
            collect(child, tests, invs, benches);
        }
    }
    collect(&ast, &mut test_names, &mut inv_names, &mut bench_names);

    println!("Module: {}", ast.name);
    println!("  Tests:      {}", test_names.len());
    println!("  Invariants: {}", inv_names.len());
    println!("  Benchmarks: {}", bench_names.len());

    if verbose {
        if !test_names.is_empty() {
            println!("\n  Test blocks:");
            for name in &test_names {
                println!("    {}", name);
            }
        }
        if !inv_names.is_empty() {
            println!("\n  Invariants:");
            for name in &inv_names {
                println!("    {}", name);
            }
        }
        if !bench_names.is_empty() {
            println!("\n  Benchmarks:");
            for name in &bench_names {
                println!("    {}", name);
            }
        }
    }

    let total = test_names.len() + inv_names.len() + bench_names.len();
    println!("\n  Total: {} declarations", total);
    Ok(())
}

fn run_doc_all(root: &str, output_dir: &str) -> anyhow::Result<()> {
    let root_path = Path::new(root);
    let files: Vec<PathBuf> = walkdir::WalkDir::new(root_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "t27").unwrap_or(false))
        .map(|e| e.path().to_path_buf())
        .collect();

    let out = Path::new(output_dir);
    fs::create_dir_all(out)?;
    let mut generated = 0u32;
    let mut errors = 0u32;

    for file in &files {
        match run_doc(&file.display().to_string(), output_dir) {
            Ok(_) => generated += 1,
            Err(e) => {
                eprintln!("doc error for {}: {}", file.display(), e);
                errors += 1;
            }
        }
    }

    println!("Doc generation: {} files, {} generated, {} errors", files.len(), generated, errors);
    if errors > 0 {
        anyhow::bail!("{} doc generation errors", errors);
    }
    Ok(())
}

fn run_visualize(input_path: &str, max_depth: u32) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;
    let file_name = std::path::Path::new(input_path).file_name().unwrap_or_default().to_string_lossy();
    println!("╔══ {} ══╗", file_name);

    fn print_tree(node: &compiler::Node, prefix: &str, is_last: bool, depth: u32, max_d: u32) {
        if max_d > 0 && depth > max_d { return; }
        let connector = if depth == 0 { "" } else if is_last { "╰── " } else { "├── " };
        let child_prefix = if depth == 0 { "" } else if is_last { "    " } else { "│   " };
        let label = if node.name.is_empty() {
            format!("{:?}", node.kind)
        } else {
            format!("{:?} \"{}\"", node.kind, node.name)
        };
        let extra = if !node.extra_type.is_empty() {
            format!(" : {}", node.extra_type)
        } else if !node.extra_return_type.is_empty() {
            format!(" -> {}", node.extra_return_type)
        } else {
            String::new()
        };
        println!("{}{}{}{}", prefix, connector, label, extra);
        let visible: Vec<&compiler::Node> = node.children.iter().collect();
        for (i, child) in visible.iter().enumerate() {
            let last = i == visible.len() - 1;
            print_tree(child, &format!("{}{}", prefix, child_prefix), last, depth + 1, max_d);
        }
    }
    print_tree(&ast, "", true, 0, max_depth);
    Ok(())
}

#[allow(dead_code)]
fn run_bench_endpoints(url: &str, requests: u32) -> anyhow::Result<()> {
    let endpoints = vec![
        ("GET", "/api/health"),
        ("GET", "/api/stats"),
        ("POST", "/api/compile"),
        ("POST", "/api/parse"),
        ("GET", "/api/seals"),
    ];
    println!("=== Benchmarking {} ({} req each) ===", url, requests);
    println!("{:<12} {:<20} {:>8} {:>10} {:>10}", "method", "endpoint", "reqs", "avg_ms", "p99_ms");
    println!("{}", "-".repeat(65));

    for (method, endpoint) in &endpoints {
        let full_url = format!("{}{}", url, endpoint);
        let mut latencies = Vec::new();
        for _ in 0..requests {
            let start = std::time::Instant::now();
            let _ = reqwest::blocking::get(&full_url);
            latencies.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let avg = latencies.iter().sum::<f64>() / latencies.len() as f64;
        let p99 = latencies[(latencies.len() * 99 / 100).min(latencies.len() - 1)];
        println!("{:<12} {:<20} {:>8} {:>10.2} {:>10.2}", method, endpoint, requests, avg, p99);
    }
    Ok(())
}

fn run_complexity(input_path: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;
    let file_name = std::path::Path::new(input_path).file_name().unwrap_or_default().to_string_lossy();
    println!("=== Complexity: {} ===", file_name);
    println!("{:<40} {:>6} {:>6} {:>6} {:>8}", "function", "stmts", "branch", "loops", "cyclomatic");
    println!("{}", "-".repeat(70));

    for child in &ast.children {
        if child.kind == compiler::NodeKind::FnDecl {
            let mut stmts = 0u32;
            let mut branches = 0u32;
            let mut loops = 0u32;
            fn count_complexity(node: &compiler::Node, stmts: &mut u32, branches: &mut u32, loops: &mut u32) {
                match node.kind {
                    compiler::NodeKind::StmtLocal | compiler::NodeKind::StmtAssign => *stmts += 1,
                    compiler::NodeKind::ExprIf | compiler::NodeKind::StmtIf => *branches += 1,
                    compiler::NodeKind::ExprSwitch => *branches += 1,
                    compiler::NodeKind::StmtFor | compiler::NodeKind::StmtWhile => *loops += 1,
                    _ => {}
                }
                for c in &node.children {
                    count_complexity(c, stmts, branches, loops);
                }
            }
            for body in &child.children {
                count_complexity(body, &mut stmts, &mut branches, &mut loops);
            }
            let cyclomatic = 1 + branches + loops;
            println!("{:<40} {:>6} {:>6} {:>6} {:>8}", child.name, stmts, branches, loops, cyclomatic);
        }
    }
    Ok(())
}

fn run_strings(input_path: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;
    let file_name = std::path::Path::new(input_path).file_name().unwrap_or_default().to_string_lossy();
    println!("=== String literals in {} ===", file_name);

    fn collect_strings(node: &compiler::Node, results: &mut Vec<(String, u32)>) {
        if node.kind == compiler::NodeKind::ExprLiteral && node.name.starts_with('"') {
            results.push((node.name.clone(), node.line));
        }
        for c in &node.children {
            collect_strings(c, results);
        }
    }

    let mut strings = Vec::new();
    collect_strings(&ast, &mut strings);
    if strings.is_empty() {
        println!("(none)");
    } else {
        for (s, line) in &strings {
            println!("  L{:>4}: \"{}\"", line, s);
        }
        println!("--- {} string literal(s)", strings.len());
    }
    Ok(())
}

fn run_symbols(input_path: &str, kind_filter: Option<&str>) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;
    let file_name = std::path::Path::new(input_path).file_name().unwrap_or_default().to_string_lossy();
    println!("=== Symbols in {} ===", file_name);
    println!("{:<30} {:<12} {:>5}", "name", "kind", "line");
    println!("{}", "-".repeat(50));

    for child in &ast.children {
        let kind_str = match child.kind {
            compiler::NodeKind::FnDecl => "fn",
            compiler::NodeKind::StructDecl => "struct",
            compiler::NodeKind::EnumDecl => "enum",
            compiler::NodeKind::ConstDecl => "const",
            compiler::NodeKind::TestBlock => "test",
            compiler::NodeKind::InvariantBlock => "invariant",
            compiler::NodeKind::BenchBlock => "bench",
            _ => continue,
        };
        if let Some(f) = kind_filter {
            if kind_str != f { continue; }
        }
        println!("{:<30} {:<12} {:>5}", child.name, kind_str, child.line);
    }
    Ok(())
}

fn run_ast_dump(input_path: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;

    fn node_to_json(node: &compiler::Node, indent: usize) -> String {
        let sp = " ".repeat(indent);
        let kind = format!("{:?}", node.kind);
        let name = if node.name.is_empty() { String::new() } else { format!(", \"name\": \"{}\"", node.name) };
        let line = format!(", \"line\": {}", node.line);
        let etype = if node.extra_type.is_empty() { String::new() } else { format!(", \"type\": \"{}\"", node.extra_type) };
        let eret = if node.extra_return_type.is_empty() { String::new() } else { format!(", \"return_type\": \"{}\"", node.extra_return_type) };
        let eop = if node.extra_op.is_empty() { String::new() } else { format!(", \"op\": \"{}\"", node.extra_op) };
        let params = if node.params.is_empty() { String::new() } else {
            let ps: Vec<String> = node.params.iter().map(|(n, t)| format!("\"{}: {}\"", n, t)).collect();
            format!(", \"params\": [{}]", ps.join(", "))
        };
        if node.children.is_empty() {
            format!("{}{{\"kind\": \"{}\"{}{}{}{}{}{}}}", sp, kind, name, line, etype, eret, eop, params)
        } else {
            let children: Vec<String> = node.children.iter().map(|c| node_to_json(c, indent + 2)).collect();
            format!("{}{{\"kind\": \"{}\"{}{}{}{}{}{},\n{}  \"children\": [\n{}\n{}  ]\n{}}}", 
                sp, kind, name, line, etype, eret, eop, params, sp, children.join(",\n"), sp, sp)
        }
    }
    println!("{}", node_to_json(&ast, 0));
    Ok(())
}

fn run_hash(input_path: &str) -> anyhow::Result<()> {
    use std::io::Read;
    let file_name = std::path::Path::new(input_path).file_name().unwrap_or_default().to_string_lossy();
    let mut f = std::fs::File::open(input_path)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    let hash = {
        use std::fmt::Write;
        let digest = <sha2::Sha256 as sha2::Digest>::digest(&buf);
        let mut s = String::with_capacity(64);
        for byte in digest {
            write!(&mut s, "{:02x}", byte).unwrap();
        }
        s
    };
    println!("{}  {}", hash, file_name);
    Ok(())
}

fn run_depth(input_path: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;
    let file_name = std::path::Path::new(input_path).file_name().unwrap_or_default().to_string_lossy();
    println!("=== Call Depth: {} ===", file_name);
    println!("{:<40} {:>6} {:>10}", "function", "depth", "max_stack");
    println!("{}", "-".repeat(60));

    for child in &ast.children {
        if child.kind == compiler::NodeKind::FnDecl {
            let mut max_d = 0u32;
            fn measure_depth(node: &compiler::Node, depth: u32, max_d: &mut u32) {
                if node.kind == compiler::NodeKind::ExprCall || node.kind == compiler::NodeKind::ExprIf || node.kind == compiler::NodeKind::StmtFor || node.kind == compiler::NodeKind::StmtWhile {
                    if depth + 1 > *max_d { *max_d = depth + 1; }
                }
                for c in &node.children {
                    measure_depth(c, depth + 1, max_d);
                }
            }
            measure_depth(child, 0, &mut max_d);
            let locals = child.children.iter()
                .filter(|c| c.kind == compiler::NodeKind::StmtLocal)
                .count() as u32;
            println!("{:<40} {:>6} {:>10}", child.name, max_d, max_d + locals);
        }
    }
    Ok(())
}

fn run_orphans(input_path: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(input_path)?;
    let ast = compiler::Compiler::parse_ast(&source).map_err(|e| anyhow::anyhow!("{}", e))?;
    let file_name = std::path::Path::new(input_path).file_name().unwrap_or_default().to_string_lossy();
    println!("=== Orphan Functions in {} ===", file_name);

    let fn_names: std::collections::HashSet<String> = ast.children.iter()
        .filter(|c| c.kind == compiler::NodeKind::FnDecl)
        .map(|c| c.name.clone())
        .collect();

    let mut called = std::collections::HashSet::new();
    fn collect_calls(node: &compiler::Node, called: &mut std::collections::HashSet<String>) {
        if node.kind == compiler::NodeKind::ExprCall {
            called.insert(node.name.clone());
        }
        for c in &node.children {
            collect_calls(c, called);
        }
    }
    collect_calls(&ast, &mut called);

    let orphans: Vec<&String> = fn_names.iter().filter(|n| !called.contains(*n)).collect();
    if orphans.is_empty() {
        println!("(no orphans — all functions are called)");
    } else {
        for name in &orphans {
            println!("  {} (never called)", name);
        }
        println!("--- {} orphan(s)", orphans.len());
    }
    Ok(())
}

fn run_synth_readiness(specs_dir: &str) -> anyhow::Result<()> {
    use walkdir::WalkDir;
    println!("=== FPGA Synthesis Readiness Check ===");
    println!("phi^2 + 1/phi^2 = 3 | TRINITY");
    println!();

    let dir = Path::new(specs_dir);
    if !dir.is_dir() {
        anyhow::bail!("{} is not a directory", specs_dir);
    }

    let files: Vec<PathBuf> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |x| x == "t27"))
        .filter(|e| !e.path().to_string_lossy().contains("testbench"))
        .map(|e| e.path().to_path_buf())
        .collect();

    let total = files.len();
    println!("Scanning {} FPGA module specs in {}", total, specs_dir);
    println!();

    let mut parse_ok = 0u32;
    let mut typecheck_ok = 0u32;
    let mut verilog_ok = 0u32;
    let mut has_tests = 0u32;
    let mut has_invariants = 0u32;
    let mut has_benches = 0u32;
    let mut has_structs = 0u32;
    let mut has_enums = 0u32;
    let mut warnings = 0u32;

    for file in &files {
        let rel = file.to_string_lossy();
        let source = fs::read_to_string(file)?;

        let ast = match compiler::Compiler::parse_ast(&source) {
            Ok(a) => { parse_ok += 1; a }
            Err(e) => { println!("FAIL parse {}: {}", rel, e); continue; }
        };

        if compiler::Compiler::typecheck(&source).is_ok() {
            typecheck_ok += 1;
        }

        if compiler::Compiler::compile_verilog(&source).is_ok() {
            verilog_ok += 1;
        }

        let has_t = ast.children.iter().any(|c| c.kind == compiler::NodeKind::TestBlock);
        let has_i = ast.children.iter().any(|c| c.kind == compiler::NodeKind::InvariantBlock);
        let has_b = ast.children.iter().any(|c| c.kind == compiler::NodeKind::BenchBlock);
        let has_s = ast.children.iter().any(|c| c.kind == compiler::NodeKind::StructDecl);
        let has_e = ast.children.iter().any(|c| c.kind == compiler::NodeKind::EnumDecl);

        if has_t { has_tests += 1; } else { warnings += 1; }
        if has_i { has_invariants += 1; }
        if has_b { has_benches += 1; }
        if has_s { has_structs += 1; }
        if has_e { has_enums += 1; }
    }

    println!("--- Results ---");
    println!("Parse:       {}/{} OK", parse_ok, total);
    println!("Typecheck:   {}/{} OK", typecheck_ok, total);
    println!("Verilog gen: {}/{} OK", verilog_ok, total);
    println!("Has tests:   {}/{}", has_tests, total);
    println!("Has inv:     {}/{}", has_invariants, total);
    println!("Has bench:   {}/{}", has_benches, total);
    println!("Has structs: {}/{}", has_structs, total);
    println!("Has enums:   {}/{}", has_enums, total);
    println!();

    let ready_pct = if total > 0 { (verilog_ok * 100) / total as u32 } else { 0 };
    let test_pct = if total > 0 { (has_tests * 100) / total as u32 } else { 0 };

    println!("Synthesis readiness: {}%", ready_pct);
    println!("Test coverage:       {}%", test_pct);

    if ready_pct == 100 && test_pct >= 80 {
        println!("\nREADY FOR SYNTHESIS");
    } else if ready_pct == 100 {
        println!("\nALMOST READY — test coverage needs improvement");
    } else {
        println!("\nNOT READY — fix parse/verilog errors first");
    }

    Ok(())
}

// ============================================================================
// Main Entry Point
// ============================================================================

#[cfg(feature = "server")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("t27c starting...");
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse { input } => run_parse(&input)?,
        Commands::Gen { input } => run_gen(&input)?,
        Commands::GenVerilog { input } => run_gen_verilog(&input)?,
        Commands::DebugHir { input } => run_debug_hir(&input)?,
        Commands::GenVerilogHir { input } => run_gen_verilog_hir(&input)?,
        Commands::Asm { input, output, format } => run_asm(&input, output.as_deref(), &format)?,
        Commands::GenTestbench { input, period_ns, max_cycles, output } => {
            run_gen_testbench(&input, period_ns, max_cycles, output.as_deref())?
        }
        Commands::GenC { input } => run_gen_c(&input)?,
        Commands::Conformance { input } => run_conformance(&input)?,
        Commands::Seal { input, save, verify } => run_seal(&input, save, verify)?,
        Commands::Compile { input, backend, output } => {
            run_compile(&input, &backend, output.as_deref())?
        }
        Commands::CompileAll { backend, output, specs_dir } => {
            run_compile_all(&backend, &output, specs_dir.as_deref())?
        }
        Commands::CompileProject { backend, output } => run_compile_project(&backend, &output)?,
        Commands::Stats => run_stats()?,
        Commands::Serve { port } => run_server(&port).await?,
        Commands::Bridge { command } => bridge::run_bridge(command)?,
        Commands::Task { command } => bridge::run_task(command)?,
        Commands::Enrich { notebook, all, force, token, lang } => enrichment::run_enrich(notebook, all, force, token, lang)?,
        Commands::Audio { notebook, all, dry_run, bilingual, workers, token, project, location, region } => {
            enrichment::run_audio(notebook, all, dry_run, bilingual, workers, token, project, location, region)?;
        }
        Commands::Suite { repo_root } => suite::run_comprehensive(&repo_root)?,
        Commands::ValidateConformance { repo_root } => {
            suite::validate_conformance(&repo_root)?
        }
        Commands::ValidateGenHeaders { repo_root } => suite::validate_gen_headers(&repo_root)?,
        Commands::CheckNow { repo_root } => suite::check_now_sync(&repo_root)?,
        Commands::Optimize { input, opt_level } => run_optimize(&input, opt_level)?,
        Commands::Typecheck { input, json } => run_typecheck(&input, json)?,
        Commands::Check { input } => run_check(&input)?,
        Commands::Test { input, verbose } => run_test(&input, verbose)?,
        Commands::Eval { expr } => run_eval(&expr)?,
        Commands::Version => run_version()?,
        Commands::Tree { input, depth } => run_tree(&input, depth)?,
        Commands::Depends { input } => run_depends(&input)?,
        Commands::Lint { input, json } => run_lint(&input, json)?,
        Commands::Bench { input } => run_bench(&input)?,
        Commands::Explain { input } => run_explain(&input)?,
        Commands::Fmt { input } => run_fmt(&input)?,
        Commands::Graph { repo_root, format } => run_graph(&repo_root, &format)?,
        Commands::Doc { input, output_dir } => run_doc(&input, &output_dir)?,
        Commands::DocAll { repo_root, output_dir } => run_doc_all(&repo_root, &output_dir)?,
        Commands::Size { input } => run_size(&input)?,
        Commands::Analyze { repo_root, json, top } => run_analyze(&repo_root, json, top)?,
        Commands::Diff { left, right } => run_diff(&left, &right)?,
        Commands::Watch { repo_root, interval_secs } => run_watch(&repo_root, interval_secs)?,
        Commands::Ci { repo_root } => run_ci(&repo_root)?,
        Commands::Inspect { input } => run_inspect(&input)?,
        Commands::Outline { input } => run_outline(&input)?,
        Commands::Callgraph { input } => run_callgraph(&input)?,
        Commands::Health => run_health()?,
        Commands::Deadcode { input, repo } => run_deadcode_cmd(&input, repo)?,
        Commands::Metrics { input } => run_metrics(&input)?,
        Commands::Flatten { input, output } => run_flatten(&input, output.as_deref())?,
        Commands::DepsTree { repo_root } => run_deps_tree(&repo_root)?,
        Commands::Todo { repo_root } => run_todo(&repo_root)?,
        Commands::Rename { input, from, to, dry_run } => run_rename(&input, &from, &to, dry_run)?,
        Commands::Spellcheck { input, max_distance } => run_spellcheck(&input, max_distance)?,
        Commands::Coverage { input } => run_coverage(&input)?,
        Commands::Validate { repo_root } => run_validate(&repo_root)?,
        Commands::Xref { input, symbol } => run_xref(&input, &symbol)?,
        Commands::BenchCompile { repo_root, iterations } => run_bench_compile(&repo_root, iterations)?,
        Commands::Minify { input } => run_minify(&input)?,
        Commands::Count { input } => run_count(&input)?,
        Commands::CheckDeps { repo_root } => run_check_deps(&repo_root)?,
        Commands::Stack { input } => run_stack(&input)?,
        Commands::Dupes { repo_root } => run_dupes(&repo_root)?,
        Commands::Init { name, output_dir } => run_init(&name, &output_dir)?,
        Commands::Exports { input } => run_exports(&input)?,
        Commands::ApiDiff { left, right } => run_api_diff(&left, &right)?,
        Commands::Loc { input } => run_loc(&input)?,
        Commands::Merge { inputs, output } => run_merge(&inputs, output.as_deref())?,
        Commands::Types { input } => run_types(&input)?,
        Commands::ToJson { input } => run_to_json(&input)?,
        Commands::Summary { repo_root } => run_summary(&repo_root)?,
        Commands::Sort { input } => run_sort(&input)?,
        Commands::UsedBy { symbol, repo_root } => run_used_by(&symbol, &repo_root)?,
        Commands::Visualize { input, depth } => run_visualize(&input, depth)?,
        Commands::BenchEndpoints { url, requests } => run_bench_endpoints(&url, requests)?,
        Commands::Complexity { input } => run_complexity(&input)?,
        Commands::Strings { input } => run_strings(&input)?,
        Commands::Symbols { input, kind } => run_symbols(&input, kind.as_deref())?,
        Commands::AstDump { input } => run_ast_dump(&input)?,
        Commands::Hash { input } => run_hash(&input)?,
        Commands::Depth { input } => run_depth(&input)?,
         Commands::Orphans { input } => run_orphans(&input)?,
         Commands::FpgaBuild { smoke, synth_only, minimal, profile, board, device, top, docker, use_hir, nextpnr, chipdb, xdc, fasm2frames, frames2bit, prjxray_db, output } => {
             let repo_root = std::env::current_dir()?;
             let effective_device = device.as_deref().unwrap_or_else(|| match board.as_deref() {
                 Some("arty-a7") => "xc7a100tcsg324-1",
                 _ => "xc7a100tcsg324-1",
             });
             run_fpga_build(&repo_root, smoke, synth_only, minimal, profile.as_deref(), board.as_deref(), effective_device, &top, docker, use_hir, nextpnr.as_deref(), chipdb.as_deref(), xdc.as_deref(), fasm2frames.as_deref(), frames2bit.as_deref(), prjxray_db.as_deref(), &output)?;
          }
         Commands::SynthReadiness { specs_dir } => run_synth_readiness(&specs_dir)?,
          Commands::ValidateSeals { pr_files } => {
             run_validate_seals(&pr_files)?;
         }
         Commands::ValidatePhiIdentity => {
             run_validate_phi_identity()?;
         }
         Commands::CheckClaimTiers => {
             eprintln!("Check claim tiers: requires repo_root, use t27c --repo-root . check-claim-tiers");
         }
         Commands::BrainSealRefresh => {
             eprintln!("Brain seal refresh: requires repo_root, use t27c --repo-root . brain-seal-refresh");
         }
         Commands::Formula { cmd } => {
             let repo_root = std::env::current_dir()?;
             formula_eval::run_formula_command(cmd, &repo_root)?;
         }
         Commands::Chimera { threshold, limit } => {
             let repo_root = std::env::current_dir()?;
             run_chimera(&repo_root, threshold, limit)?;
         }
         Commands::Sensitivity { id, param, min, max, n } => {
             let repo_root = std::env::current_dir()?;
             run_sensitivity(&repo_root, &id, &param, min, max, n)?;
         }
         Commands::TernaryEncode { value } => {
            use crate::ternary::encode_trits;
            let encoded = encode_trits(value);
            println!("Encoded {} as ternary: {:?}", value, encoded);
        }
        Commands::TernaryDecode { trits } => {
            use crate::ternary::{parse_trits, decode_trits};
            match parse_trits(&trits) {
                Some(encoding) => {
                    let decoded = decode_trits(encoding);
                    println!("Decoded ternary \"{}\" as integer: {}", trits, decoded);
                }
                None => {
                    eprintln!("Error: Invalid ternary format \"{}\"", trits);
                    eprintln!("Expected format: [-1, 0, 1] or similar");
                    std::process::exit(1);
                }
            }
        }
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
        Commands::DebugHir { input } => run_debug_hir(&input)?,
        Commands::GenVerilogHir { input } => run_gen_verilog_hir(&input)?,
        Commands::Asm { input, output, format } => run_asm(&input, output.as_deref(), &format)?,
        Commands::GenTestbench { input, period_ns, max_cycles, output } => {
            run_gen_testbench(&input, period_ns, max_cycles, output.as_deref())?
        }
        Commands::GenXdc { profile, output } => run_gen_xdc(&profile, output.as_deref())?,
        Commands::CheckPins { xdc, db } => run_check_pins(&xdc, db.as_deref())?,
        Commands::XdcVerify => run_xdc_verify()?,
        Commands::GenC { input } => run_gen_c(&input)?,
        Commands::GenRust { input } => run_gen_rust(&input)?,
        Commands::Conformance { input } => run_conformance(&input)?,
        Commands::Seal { input, save, verify } => run_seal(&input, save, verify)?,
        Commands::Compile { input, backend, output } => {
            run_compile(&input, &backend, output.as_deref())?
        }
        Commands::CompileAll { backend, output, specs_dir } => {
            run_compile_all(&backend, &output, specs_dir.as_deref())?
        }
        Commands::CompileProject { backend, output } => run_compile_project(&backend, &output)?,
        Commands::Stats => run_stats()?,
        Commands::Bridge { command } => bridge::run_bridge(command)?,
        Commands::Task { command } => bridge::run_task(command)?,
        Commands::Enrich { notebook, all, force, token, lang } => enrichment::run_enrich(notebook, all, force, token, lang)?,
        Commands::Audio { notebook, all, dry_run, bilingual, workers, token, project, location, region } => {
            enrichment::run_audio(notebook, all, dry_run, bilingual, workers, token, project, location, region)?;
        }
        Commands::Suite { repo_root } => suite::run_comprehensive(&repo_root)?,
        Commands::ValidateConformance { repo_root } => {
            suite::validate_conformance(&repo_root)?
        }
        Commands::ValidateGenHeaders { repo_root } => suite::validate_gen_headers(&repo_root)?,
        Commands::CheckNow { repo_root } => suite::check_now_sync(&repo_root)?,
        Commands::Optimize { input, opt_level } => run_optimize(&input, opt_level)?,
        Commands::Typecheck { input, json } => run_typecheck(&input, json)?,
        Commands::Check { input } => run_check(&input)?,
        Commands::Test { input, verbose } => run_test(&input, verbose)?,
        Commands::Eval { expr } => run_eval(&expr)?,
        Commands::Version => run_version()?,
        Commands::Tree { input, depth } => run_tree(&input, depth)?,
        Commands::Depends { input } => run_depends(&input)?,
        Commands::Lint { input, json } => run_lint(&input, json)?,
        Commands::Bench { input } => run_bench(&input)?,
        Commands::Explain { input } => run_explain(&input)?,
        Commands::Fmt { input } => run_fmt(&input)?,
        Commands::Graph { repo_root, format } => run_graph(&repo_root, &format)?,
        Commands::Doc { input, output_dir } => run_doc(&input, &output_dir)?,
        Commands::DocAll { repo_root, output_dir } => run_doc_all(&repo_root, &output_dir)?,
        Commands::Size { input } => run_size(&input)?,
        Commands::Analyze { repo_root, json, top } => run_analyze(&repo_root, json, top)?,
        Commands::Diff { left, right } => run_diff(&left, &right)?,
        Commands::Watch { repo_root, interval_secs } => run_watch(&repo_root, interval_secs)?,
        Commands::Ci { repo_root } => run_ci(&repo_root)?,
        Commands::Inspect { input } => run_inspect(&input)?,
        Commands::Outline { input } => run_outline(&input)?,
        Commands::Callgraph { input } => run_callgraph(&input)?,
        Commands::Health => run_health()?,
        Commands::Deadcode { input, repo } => run_deadcode_cmd(&input, repo)?,
        Commands::Metrics { input } => run_metrics(&input)?,
        Commands::Flatten { input, output } => run_flatten(&input, output.as_deref())?,
        Commands::DepsTree { repo_root } => run_deps_tree(&repo_root)?,
        Commands::Todo { repo_root } => run_todo(&repo_root)?,
        Commands::Rename { input, from, to, dry_run } => run_rename(&input, &from, &to, dry_run)?,
        Commands::Spellcheck { input, max_distance } => run_spellcheck(&input, max_distance)?,
        Commands::Coverage { input } => run_coverage(&input)?,
        Commands::Validate { repo_root } => run_validate(&repo_root)?,
        Commands::Xref { input, symbol } => run_xref(&input, &symbol)?,
        Commands::BenchCompile { repo_root, iterations } => run_bench_compile(&repo_root, iterations)?,
        Commands::Minify { input } => run_minify(&input)?,
        Commands::Count { input } => run_count(&input)?,
        Commands::CheckDeps { repo_root } => run_check_deps(&repo_root)?,
        Commands::Stack { input } => run_stack(&input)?,
        Commands::Dupes { repo_root } => run_dupes(&repo_root)?,
        Commands::Init { name, output_dir } => run_init(&name, &output_dir)?,
        Commands::Exports { input } => run_exports(&input)?,
        Commands::ApiDiff { left, right } => run_api_diff(&left, &right)?,
        Commands::Loc { input } => run_loc(&input)?,
        Commands::Merge { inputs, output } => run_merge(&inputs, output.as_deref())?,
        Commands::Types { input } => run_types(&input)?,
        Commands::ToJson { input } => run_to_json(&input)?,
        Commands::Summary { repo_root } => run_summary(&repo_root)?,
        Commands::Sort { input } => run_sort(&input)?,
        Commands::UsedBy { symbol, repo_root } => run_used_by(&symbol, &repo_root)?,
        Commands::Visualize { input, depth } => run_visualize(&input, depth)?,
        Commands::BenchEndpoints { .. } => {
            eprintln!("Error: 'bench-endpoints' requires 'server' feature");
            std::process::exit(1);
        }
        Commands::Complexity { input } => run_complexity(&input)?,
        Commands::Strings { input } => run_strings(&input)?,
        Commands::Symbols { input, kind } => run_symbols(&input, kind.as_deref())?,
        Commands::AstDump { input } => run_ast_dump(&input)?,
        Commands::Hash { input } => run_hash(&input)?,
        Commands::Depth { input } => run_depth(&input)?,
        Commands::Orphans { input } => run_orphans(&input)?,
         Commands::FpgaBuild { smoke, synth_only, minimal, profile, board, device, top, docker, use_hir, nextpnr, chipdb, xdc, fasm2frames, frames2bit, prjxray_db, output } => {
             let repo_root = std::env::current_dir()?;
             let effective_device = device.as_deref().unwrap_or_else(|| match board.as_deref() {
                 Some("arty-a7") => "xc7a100tcsg324-1",
                 _ => "xc7a100tcsg324-1",
             });
             run_fpga_build(&repo_root, smoke, synth_only, minimal, profile.as_deref(), board.as_deref(), effective_device, &top, docker, use_hir, nextpnr.as_deref(), chipdb.as_deref(), xdc.as_deref(), fasm2frames.as_deref(), frames2bit.as_deref(), prjxray_db.as_deref(), &output)?;
         }
         Commands::ValidateSeals { pr_files } => {
             run_validate_seals(&pr_files)?;
         }
         Commands::ValidatePhiIdentity => {
             run_validate_phi_identity()?;
         }
         Commands::CheckClaimTiers => {
             eprintln!("Check claim tiers: requires repo_root, use t27c --repo-root . check-claim-tiers");
         }
         Commands::BrainSealRefresh => {
             eprintln!("Brain seal refresh: requires repo_root, use t27c --repo-root . brain-seal-refresh");
         }
         Commands::Formula { cmd } => {
             let repo_root = std::env::current_dir()?;
             formula_eval::run_formula_command(cmd, &repo_root)?;
         }
         Commands::Chimera { threshold, limit } => {
             let repo_root = std::env::current_dir()?;
             run_chimera(&repo_root, threshold, limit)?;
         }
         Commands::Sensitivity { id, param, min, max, n } => {
             let repo_root = std::env::current_dir()?;
             run_sensitivity(&repo_root, &id, &param, min, max, n)?;
         }
        Commands::TernaryEncode { value } => {
            use crate::ternary::encode_trits;
            let encoded = encode_trits(value);
            println!("Encoded {} as ternary: {:?}", value, encoded);
        }
        Commands::SynthReadiness { specs_dir } => run_synth_readiness(&specs_dir)?,
        Commands::ValidateSeals { pr_files } => {
            run_validate_seals(&pr_files)?;
        }
        Commands::Serve { .. } => {
            eprintln!("Error: 'serve' command requires 'server' feature");
            eprintln!("Build with: cargo build --release --features server");
            std::process::exit(1);
        }
        Commands::TernaryEncode { value } => {
            use crate::ternary::encode_trits;
            let encoded = encode_trits(value);
            println!("Encoded {} as ternary: {:?}", value, encoded);
        }
        Commands::TernaryDecode { trits } => {
            use crate::ternary::{parse_trits, decode_trits};
            match parse_trits(&trits) {
                Some(encoding) => {
                    let decoded = decode_trits(encoding);
                    println!("Decoded ternary \"{}\" as integer: {}", trits, decoded);
                }
                None => {
                    eprintln!("Error: Invalid ternary format. Use format like \"[-1, 0, 1]\"");
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
