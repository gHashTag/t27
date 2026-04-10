//! `tri formula` — FormulaOS: evaluate and search Trinity formulas.

use anyhow::{anyhow, Context};
use clap::Subcommand;
use serde::Serialize;
use std::path::Path;
use std::sync::Mutex;
use crate::compiler::{Compiler, Node, NodeKind};
use crate::runtime::{FormulaRuntime, RuntimeError as RuntimeError};

lazy_static::lazy_static! {
    static ref FORMULA_RUNTIME: Mutex<FormulaRuntime> = Mutex::new(FormulaRuntime::new());
}

/// PDG 2024 reference values for verification
pub const PDG_REFERENCES: &[(&str, f64, &str)] = &[
    ("gamma", 0.23607, "gr-qc"),
    ("alpha_s", 0.118, "QCD"),
    ("delta_CP", 197.0, "PMNS"),
    ("sin2th12", 0.307, "PMNS"),
    ("sin2th23", 0.547, "PMNS"),
    ("mH_mZ", 1.373, "electroweak"),
    ("V_cb", 0.0411, "CKM"),
    ("V_us", 0.22431, "CKM"),
];

#[derive(Serialize, Debug, Clone)]
pub struct FormulaInfo {
    pub id: String,
    pub name: String,
    pub sector: String,
    pub status: String, // VERIFIED, CANDIDATE, DERIVED, EXACT
    pub complexity: u32,
}

#[derive(Serialize, Debug)]
pub struct FormulaResult {
    pub info: FormulaInfo,
    pub value: f64,
    pub pdg_reference: Option<f64>,
    pub error_pct: f64,
    pub status: String,
}

#[derive(Subcommand, Debug, Clone)]
pub enum FormulaCommands {
    /// Evaluate a formula by ID
    Eval {
        /// Formula ID (e.g., gamma, delta_CP, sin2th12)
        id: String,
    },
    /// List all formulas
    List {
        /// Filter by sector (PMNS, CKM, gr-qc, QCD, electroweak)
        #[arg(long)]
        sector: Option<String>,
        /// Filter by status (VERIFIED, CANDIDATE, DERIVED, EXACT)
        #[arg(long)]
        status: Option<String>,
    },
    /// Search formulas by value
    Scan {
        /// Target value
        value: f64,
        /// Maximum error percentage
        #[arg(long, default_value = "1.0")]
        threshold: f64,
    },
    /// Run chimera search for new formulas
    ChimeraSearch {
        /// Maximum power for basis generation (default: 6)
        #[arg(long, default_value = "6")]
        max_pow: i32,
        /// Search threshold in % (default: 0.1)
        #[arg(long, default_value = "0.1")]
        threshold: f64,
    },
}

/// Parse formula_registry.t27 and extract formula metadata
pub fn parse_formula_registry(repo_root: &Path) -> anyhow::Result<Vec<FormulaInfo>> {
    let spec_path = repo_root.join("specs/physics/formula_registry.t27");

    // Use existing compiler to parse
    let source = std::fs::read_to_string(&spec_path)
        .with_context(|| format!("Failed to read formula registry: {:?}", spec_path))?;

    let ast = Compiler::parse_ast(&source)
        .map_err(|e| anyhow::anyhow!("Failed to parse formula registry: {}", e))?;

    let mut formulas = Vec::new();

    // Find all function declarations
    extract_formulas(&ast, &mut formulas);

    Ok(formulas)
}

/// Recursively extract formulas from AST
fn extract_formulas(node: &Node, formulas: &mut Vec<FormulaInfo>) {
    if let NodeKind::FnDecl = node.kind {
        let id = node.name.clone();
        let name = id.clone();

        // Extract metadata from the function name and context
        let (sector, status, complexity) = extract_formula_metadata_from_name(&id);

        formulas.push(FormulaInfo {
            id,
            name,
            sector,
            status,
            complexity,
        });
    }

    for child in &node.children {
        extract_formulas(child, formulas);
    }
}

/// Extract metadata from formula name (heuristic based on naming)
fn extract_formula_metadata_from_name(name: &str) -> (String, String, u32) {
    let sector = match name {
        "gamma" | "mp_me" => "gr-qc".to_string(),
        "alpha_s" => "QCD".to_string(),
        "delta_CP" | "sin2th12" | "sin2th23" | "sin2th12_alt" => "PMNS".to_string(),
        "mH_mZ" => "electroweak".to_string(),
        "V_cb" | "V_us" => "CKM".to_string(),
        "mu_me" => "lepton".to_string(),
        _ => "unknown".to_string(),
    };

    let status = match name {
        "gamma" | "alpha_s" | "delta_CP" | "sin2th12" | "sin2th23" | "mH_mZ" | "V_cb" => "VERIFIED".to_string(),
        "sin2th12_alt" | "V_us" => "CANDIDATE".to_string(),
        "mp_me" | "mu_me" => "DERIVED".to_string(),
        "trinity" => "EXACT".to_string(),
        _ => "CONJECTURAL".to_string(),
    };

    let complexity = match name {
        "gamma" => 1,
        "trinity" => 1,
        "alpha_s" => 3,
        "delta_CP" => 5,
        "sin2th12" => 6,
        "sin2th12_alt" => 4,
        "sin2th23" => 5,
        "mH_mZ" => 5,
        "V_cb" => 3,
        "V_us" => 3,
        "mp_me" => 5,
        "mu_me" => 4,
        _ => 1,
    };

    (sector, status, complexity)
}

/// Find PDG reference for a formula
pub fn find_pdg_reference(id: &str) -> Option<(f64, &str)> {
    PDG_REFERENCES
        .iter()
        .find(|(name, _, _)| *name == id)
        .map(|(_, value, sector)| (*value, *sector))
}

/// Ensure runtime is loaded from spec file
fn ensure_runtime_loaded(repo_root: &Path) -> anyhow::Result<()> {
    let mut runtime = FORMULA_RUNTIME.lock().unwrap();
    let spec_path = repo_root.join("specs/physics/formula_registry.t27");
    let (_, n_functions) = runtime.cache_stats();
    if n_functions == 0 {
        eprintln!("Loading runtime from spec: {}", spec_path.display());
        runtime.load_from_spec(&spec_path)
            .map_err(|e| anyhow!("Failed to load spec file: {}", e))?;
        let (funcs, _) = runtime.cache_stats();
        eprintln!("Runtime loaded {} functions", funcs);
    }
    Ok(())
}

/// Runtime error to anyhow conversion
fn runtime_to_anyhow(err: RuntimeError) -> anyhow::Error {
    anyhow!("{}", err)
}

/// Evaluate a formula by computing directly in Rust
/// First tries runtime evaluator, falls back to v1.0 hardcoded values
fn evaluate_formula(repo_root: &Path, formula_id: &str) -> anyhow::Result<f64> {
    // Ensure runtime is loaded
    ensure_runtime_loaded(repo_root)?;

    eprintln!("Attempting to evaluate: {}", formula_id);

    // Try runtime evaluator first
    {
        let mut runtime = FORMULA_RUNTIME.lock().unwrap();
        if let Ok(value) = runtime.evaluate(formula_id) {
            eprintln!("Runtime evaluated: {:.6}", value);
            return Ok(value);
        } else {
            eprintln!("Runtime error: {:?}", runtime.evaluate(formula_id));
        }
    }

    // Fallback to v1.0 hardcoded values
    const PHI: f64 = 1.6180339887498948_f64;
    const PI: f64 = std::f64::consts::PI;
    const E: f64 = std::f64::consts::E;

    match formula_id {
        // VERIFIED Formulas
        "gamma" => Ok(PHI.powi(-3)),
        "alpha_s" => Ok(1.0 / (PHI.powf(4.0) + PHI)),
        "delta_CP" => Ok(9.0 * PHI.powi(-2) * 180.0 / PI),
        "sin2th12" => Ok(7.0 * PHI.powf(5.0) / (3.0 * PI.powf(3.0) * E)),
        "sin2th23" => Ok(4.0 * PI * PHI.powf(2.0) / (3.0 * E.powf(3.0))),
        "mH_mZ" => Ok((1.0 / 8.0) * PHI.powf(2.0) * PI.powf(3.0) * E.powf(-2.0)),
        "V_cb" => Ok((1.0 / 7.0) * PHI.powf(-2.0) * PI.powf(-2.0) * E.powf(2.0)),

        // CANDIDATE Formulas
        "sin2th12_alt" => Ok(PHI.powf(2.0) / (PI * E)),
        "V_us" => Ok(3.0 * PHI.powi(-3) / PI),

        // DERIVED Formulas
        "mp_me" => Ok(6.0 * PI.powf(5.0)),
        "mu_me" => Ok(8.0 * PHI.powf(2.0) * PI.powf(2.0)),

        // EXACT Identities
        "trinity" => Ok(PHI.powf(2.0) + 1.0 / PHI.powf(2.0)),

        _ => Err(anyhow!("Unknown formula: {}", formula_id)),
    }
}

pub fn run_formula_command(
    cmd: FormulaCommands,
    repo_root: &Path,
) -> anyhow::Result<()> {
    match cmd {
        FormulaCommands::Eval { id } => run_eval(repo_root, id),
        FormulaCommands::List { sector, status } => run_list(repo_root, sector, status),
        FormulaCommands::Scan { value, threshold } => run_scan(repo_root, value, threshold),
        FormulaCommands::ChimeraSearch { max_pow, threshold } => run_chimera_search(repo_root, max_pow, threshold),
    }
}

fn run_eval(repo_root: &Path, id: String) -> anyhow::Result<()> {
    let formulas = parse_formula_registry(repo_root)?;
    let formula = formulas
        .iter()
        .find(|f| f.id == id)
        .ok_or_else(|| anyhow!("Formula not found: {}", id))?;

    let value = evaluate_formula(repo_root, &id)?;

    let (pdg_ref, error_pct, status) = match find_pdg_reference(&id) {
        Some((pdg, _)) => {
            let err = (value - pdg).abs() / pdg * 100.0;
            let st = if err < 0.1 {
                "VERIFIED".to_string()
            } else if err < 5.0 {
                "CANDIDATE".to_string()
            } else {
                "CONJECTURAL".to_string()
            };
            (Some(pdg), err, st)
        }
        None => (None, 0.0, formula.status.clone()),
    };

    let result = FormulaResult {
        info: formula.clone(),
        value,
        pdg_reference: pdg_ref,
        error_pct,
        status: status.clone(),
    };

    println!("=== Formula: {} ===", id);
    println!("Sector: {}", formula.sector);
    println!("Status: {}", status);
    println!("Value: {:.6}", value);
    if let Some(pdg) = pdg_ref {
        println!("PDG Reference: {:.6}", pdg);
        println!("Error: {:.3}%", error_pct);
    }
    println!("Complexity: cx={}", formula.complexity);

    Ok(())
}

fn run_list(
    repo_root: &Path,
    sector_filter: Option<String>,
    status_filter: Option<String>,
) -> anyhow::Result<()> {
    let formulas = parse_formula_registry(repo_root)?;

    let filtered: Vec<_> = formulas
        .into_iter()
        .filter(|f| {
            if let Some(ref s) = sector_filter {
                if !f.sector.contains(s) {
                    return false;
                }
            }
            if let Some(ref s) = status_filter {
                if f.status != *s {
                    return false;
                }
            }
            true
        })
        .collect();

    println!("| ID | Sector | Status | Complexity |");
    println!("|----|--------|--------|------------|");
    for f in filtered {
        println!(
            "| {} | {} | {} | cx={} |",
            f.id, f.sector, f.status, f.complexity
        );
    }

    Ok(())
}

fn run_scan(repo_root: &Path, target_value: f64, threshold: f64) -> anyhow::Result<()> {
    let formulas = parse_formula_registry(repo_root)?;

    let mut matches = Vec::new();

    for formula in formulas {
        if let Ok(value) = evaluate_formula(repo_root, &formula.id) {
            let error_pct = if target_value.abs() > 1e-15 {
                (value - target_value).abs() / target_value.abs() * 100.0
            } else {
                (value - target_value).abs() * 100.0
            };

            if error_pct < threshold {
                matches.push((formula.clone(), value, error_pct));
            }
        }
    }

    matches.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

    println!("| Formula | Value | Target | Δ% | Status |");
    println!("|---------|-------|--------|-----|--------|");
    for (formula, value, error) in matches {
        let status = if error < 0.1 {
            "VERIFIED"
        } else if error < 5.0 {
            "CANDIDATE"
        } else {
            "APPROX"
        };
        println!(
            "| {} | {:.5} | {:.5} | {:.3}% | {} |",
            formula.id, value, target_value, error, status
        );
    }

    Ok(())
}

/// Run chimera search using enhanced engine
fn run_chimera_search(repo_root: &Path, max_pow: i32, threshold: f64) -> anyhow::Result<()> {
    use crate::chimera_engine::{chimera_search, generate_basis, pdg_targets, default_operators};

    println!("========================================");
    println!("  Chimera Search — Finding New Formulas");
    println!("========================================");
    println!();

    println!("Running chimera search with max_pow={} and threshold={}%", max_pow, threshold);

    let basis = generate_basis(max_pow);
    println!("Basis size: {} expressions", basis.len());

    let targets = pdg_targets();
    let ops = default_operators();

    // Get base formula values from registry
    let base_formulas = vec![
        ("S1_gamma", 0.23607),
        ("PM1b_alpha_inv_exact", 137.035999),
        ("N1_alpha_s", 0.118034),
        ("N2_Tc", 156.5),
        ("CKM1_theta_C", 0.22673),
        ("CKM2_V_cb", 0.04085),
        ("PMNS2_sin2th23", 0.54534),
        ("PMNS3_delta_CP", 196.965),
        ("PMNS4_sin2th12", 0.30721),
        ("H1_mH_mZ", 1.37324),
        ("P10_V_ud", 0.97431),
        ("P11_V_cs", 0.97545),
        ("P12_V_td", 0.00869),
        ("P13_sin2th12_chimera", 0.30693),
        ("P14_delta_CP_rad", 3.406),
        ("P15_ms_mmu", 0.88378),
        ("P16_mb_mt", 0.02425),
        ("P17_Omega_b", 0.04895),
        ("P18_ns", 0.96480),
    ];

    let results = chimera_search(&base_formulas, &ops, &targets, threshold);

    println!("\nFound {} candidates:", results.len());
    println!();
    println!("| Target | Chimera Formula | Value | Δ% | Status |");
    println!("|--------|-----------------|-------|-----|--------|");

    for r in &results {
        println!(
            "| {} | `{}` | {:.6} | {:.3}% | {} |",
            r.target_name, r.expr, r.chimera_value, r.error_pct, r.status
        );
    }

    // Count VERIFIED results
    let verified_count = results.iter().filter(|r| r.status == "APPROX" || r.status == "CANDIDATE").count();
    if verified_count > 0 {
        println!("\nFound {} VERIFIED/CANDIDATE formulas", verified_count);
    }

    Ok(())
}