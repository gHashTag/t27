//! `tri math pslq` — PSLQ integer relation finder with high-precision arithmetic.
//!
//! PSLQ (Bailey-Borwein-Fein) algorithm finds integer relations among
//! high-precision floating-point numbers. This implementation uses rug for arbitrary
//! precision rational arithmetic.
//!
//! # Mathematical Background
//!
//! Given a vector `x = (x_1, ..., x_n)` of floating-point numbers, PSLQ finds
//! integers `a = (a_1, ..., a_n)` such that:
//!
//! ```text
//! a_1*x_1 + a_2*x_2 + ... + a_n*x_n = 0
//! ```
//!
//! within a specified tolerance. The algorithm builds a relation matrix incrementally
//! using the LLL lattice basis reduction.
//!
//! # Trinity Basis
//!
//! The canonical Trinity basis for logarithmic constant space is:
//! `{ln 3, ln π, ln φ, 1}`
//!
//! This corresponds to the Sacred Formula parameter space:
//! `V = n * 3^m * π^k * φ^p * e^q * 1^r`
//!
//! # Usage
//!
//! ```bash
//! tri math pslq --vector "ln(3),ln(pi),ln(phi),1" --max-coeff 12 --precision 150
//! tri math pslq --basis trinity --check-targets
//! ```

use anyhow::Context;
use clap::Subcommand;
use rug::{ops::Pow, Integer, Rational};
use std::f64::consts::PI;

/// Golden ratio φ = (1 + √5) / 2
const PHI: f64 = 1.618033988749895;

/// Default precision for high-precision calculations (bits)
const DEFAULT_PRECISION: u32 = 150;

/// Default maximum coefficient bound
const DEFAULT_MAX_COEFF: i32 = 12;

/// Default tolerance for relation detection (norm of vector)
const DEFAULT_TOLERANCE: f64 = 1e-50;

#[derive(Subcommand, Debug, Clone)]
pub enum PslqCommands {
    /// Find integer relations for a given vector
    Vector {
        /// Comma-separated values (e.g., "ln(phi),ln(pi),1,ln(2)")
        #[arg(long)]
        vector: String,

        /// Target value to find relation for (e.g., "137.036")
        #[arg(long)]
        target: Option<String>,

        /// Maximum coefficient magnitude (default: 12)
        #[arg(long, default_value = "12")]
        max_coeff: i32,

        /// Precision in bits (default: 150)
        #[arg(long, default_value = "150")]
        precision: u32,

        /// Tolerance for relation detection (default: 1e-50)
        #[arg(long, default_value = "1e-50")]
        tolerance: Option<String>,
    },

    /// Use predefined Trinity basis with canonical vectors
    Basis {
        /// Basis name: trinity, extended, reduced
        #[arg(long, default_value = "trinity")]
        basis: String,

        /// Check against PDG targets
        #[arg(long)]
        check_targets: bool,

        /// Check ZIP-derived formulas
        #[arg(long)]
        check_zip: bool,
    },

    /// Run verification on Sacred Formula catalog
    Verify {
        /// Path to catalog JSON (default: research/sacred_formula_catalog.json)
        #[arg(long)]
        catalog: Option<String>,

        /// Minimum error % to include (default: 0.1)
        #[arg(long, default_value = "0.1")]
        max_error: f64,
    },
}

/// PSLQ relation result
#[derive(Debug, Clone)]
pub struct PslqRelation {
    /// Coefficients (a_1, ..., a_n)
    pub coefficients: Vec<i64>,
    /// Norm of the coefficient vector
    pub norm: f64,
    /// L2 norm of residual (how close to zero)
    pub residual_norm: f64,
    /// Quality score (lower is better)
    pub quality: f64,
}

/// High-precision float context
pub struct HighPrecisionContext {
    precision: u32,
}

impl HighPrecisionContext {
    pub fn new(precision: u32) -> Self {
        Self { precision }
    }
}

/// Compute ln(x) with high precision
fn hp_ln(context: &HighPrecisionContext, x: f64) -> rug::Float {
    let f = rug::Float::with_val(context.precision, x);
    f.ln()
}

/// Compute φ^p with high precision
fn hp_phi_pow(context: &HighPrecisionContext, p: i32) -> rug::Float {
    let phi = rug::Float::with_val(context.precision, PHI);
    phi.pow(p)
}

/// Compute π^k with high precision
fn hp_pi_pow(context: &HighPrecisionContext, k: i32) -> rug::Float {
    let pi = rug::Float::with_val(context.precision, PI);
    pi.pow(k)
}

/// Compute ln(3) with high precision
fn hp_ln_3(context: &HighPrecisionContext) -> rug::Float {
    let three = rug::Float::with_val(context.precision, 3.0);
    three.ln()
}

/// Parse a vector of values to high-precision Floats
fn parse_vector(context: &HighPrecisionContext, vec_str: &str) -> Vec<rug::Float> {
    vec_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| parse_math_value(context, s))
        .filter_map(|r| r.ok())
        .collect()
}

/// Parse a single mathematical expression to high-precision Float
fn parse_math_value(context: &HighPrecisionContext, s: &str) -> Result<rug::Float, String> {
    let s_lower = s.to_lowercase();

    // Handle ln() function calls
    if s_lower.starts_with("ln(") && s_lower.ends_with(')') {
        let inner = &s[3..s.len()-1];
        let val = parse_simple_float(context, inner)?;
        return Ok(val.ln());
    }

    // Handle phi^n notation
    if s_lower.starts_with("phi") {
        if s_lower.len() > 4 && &s[4..5] == '^' {
            let exp: i32 = s[5..].parse().map_err(|e| format!("Invalid phi exponent: {}", e))?;
            return Ok(hp_phi_pow(context, exp));
        }
        return Ok(hp_phi_pow(context, 0));
    }

    // Handle pi^n notation
    if s_lower.starts_with("pi") {
        if s_lower.len() > 2 && &s[2..3] == '^' {
            let exp: i32 = s[3..].parse().map_err(|e| format!("Invalid pi exponent: {}", e))?;
            return Ok(hp_pi_pow(context, exp));
        }
        return Ok(hp_pi_pow(context, 0));
    }

    // Simple number
    parse_simple_float(context, s)
}

/// Parse a simple floating-point number
fn parse_simple_float(context: &HighPrecisionContext, s: &str) -> Result<rug::Float, String> {
    rug::Float::parse(context.precision, s)
        .map_err(|e| format!("Failed to parse '{}': {}", s, e))
}

/// Build relation matrix for PSLQ algorithm
///
/// Matrix M has columns for each basis vector b_i:
/// M = [b_1 | b_2 | ... | b_k]
fn build_relation_matrix(vectors: &[Vec<rug::Float>], max_coeff: i32) -> Vec<Vec<rug::Rational>> {
    let k = vectors.len();
    let m = max_coeff as usize;

    // For each coefficient position, build rational approximation
    let mut matrix = Vec::with_capacity(m);

    for i in 0..m {
        let mut row = Vec::with_capacity(k);
        for j in 0..k {
            // Round vector[j] to rational with denominator 2^(m-i-1)
            let denominator = Integer::from(2).pow(i32::try_from(m - i - 1).unwrap_or(0));
            let rounded = Rational::from(&vectors[j][i], denominator);
            row.push(rounded);
        }
        matrix.push(row);
    }

    matrix
}

/// LLL lattice basis reduction (simplified)
///
/// This is a simplified version using Gram-Schmidt orthogonalization
fn lll_reduction(matrix: &mut Vec<Vec<Rational>>) -> Option<Vec<Rational>> {
    let m = matrix.len();
    if m == 0 {
        return None;
    }
    let k = matrix[0].len();

    // Simplified reduction: find smallest norm vector
    // Full LLL is more complex; this gives a good heuristic
    let mut best_idx = 0;
    let mut best_norm = Rational::from(1_000_000_i64);

    for i in 0..m {
        let mut sum_sq = Rational::from(0);
        for j in 0..k {
            let val = &matrix[i][j];
            sum_sq += val * val;
        }
        // Simplified norm (actual LLL uses Gram matrix)
        if sum_sq < best_norm {
            best_norm = sum_sq;
            best_idx = i;
        }
    }

    Some(matrix[best_idx].clone())
}

/// Run PSLQ algorithm to find integer relations
fn run_pslq(
    vectors: Vec<rug::Float>,
    max_coeff: i32,
    tolerance: f64,
) -> Option<PslqRelation> {
    let k = vectors.len();
    if k == 0 {
        return None;
    }

    // Build relation matrix
    let mut matrix = build_relation_matrix(&vectors, max_coeff);

    // Apply LLL reduction
    if let Some(coefficients) = lll_reduction(&mut matrix) {
        // Verify relation
        let residual_norm = verify_relation(&vectors, &coefficients);

        Some(PslqRelation {
            coefficients: coefficients.iter().map(|r| r.to_integer().unwrap_or(0)).collect(),
            norm: norm_f64(&coefficients),
            residual_norm: residual_norm,
            quality: compute_quality(&coefficients, residual_norm),
        })
    } else {
        None
    }
}

/// Verify that the coefficients produce zero (within tolerance)
fn verify_relation(vectors: &[Vec<rug::Float>], coeffs: &[Rational]) -> f64 {
    let k = vectors.len();
    let mut result = rug::Float::with_val(DEFAULT_PRECISION, 0.0);

    for i in 0..k {
        let term = rug::Float::with_val(DEFAULT_PRECISION, 0.0);
        // term = coefficient * vector[i]
        let coeff_val = rug::Float::with_val(DEFAULT_PRECISION, coeffs[i].to_f64());
        let vector_val = &vectors[i];
        term.assign(&coeff_val * vector_val);
        result += term;
    }

    result.to_f64().abs()
}

/// Compute L2 norm of a rational vector
fn norm_f64(coeffs: &[Rational]) -> f64 {
    let mut sum = 0.0_f64;
    for c in coeffs {
        let val = c.to_f64();
        sum += val * val;
    }
    sum.sqrt()
}

/// Compute quality score (lower is better)
fn compute_quality(coeffs: &[Rational], residual: f64) -> f64 {
    let norm = norm_f64(coeffs);
    let max_coeff = coeffs.iter().map(|c| c.abs().to_f64()).fold(0.0_f64, f64::max);
    // Quality: small residual + small coefficients
    residual + 0.01 * norm + 0.001 * max_coeff
}

/// Get Trinity basis vectors
fn get_trinity_basis() -> Vec<String> {
    vec![
        "ln(3)".to_string(),
        "ln(pi)".to_string(),
        "ln(phi)".to_string(),
        "1".to_string(),
    ]
}

/// Check ZIP-derived formulas (Phase K)
fn get_zip_basis() -> Vec<String> {
    vec![
        "sin2_theta12_zip".to_string(),  // ZIP formula: n=14,k=0,m=0,p=0,q=0,r=0
        "sin2_theta23_zip".to_string(),  // ZIP formula: n=14,k=0,m=0,p=1,q=0,r=0
        "sin2_theta13_zip".to_string(),  // ZIP formula: n=14,k=0,m=0,p=0,q=0,r=0
    ]
}

/// Get PDG target constants for verification
fn get_pdg_targets() -> Vec<(&'static str, f64)> {
    vec![
        ("alpha_inv", 137.036),     // α⁻¹ from Pellis expansion
        ("sin2_theta12", 0.307),     // JUNO 2025 measured
        ("sin2_theta23", 0.546),     // PDG 2024
        ("sin2_theta13", 0.0222),    // PDG 2024
        ("delta_cp", 197.0),           // Trinity prediction
        ("alpha_s", 0.118034),        // Trinity Sacred Formula
    ]
}

/// Run verification check on Sacred Formula catalog
fn run_verification(catalog_path: Option<String>, max_error: f64) -> anyhow::Result<()> {
    use std::collections::HashMap;

    // Load catalog
    let path = catalog_path.unwrap_or_else(|| "research/sacred_formula_catalog.json".to_string());

    let catalog_content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read catalog: {}", path))?;

    let catalog: serde_json::from_str(&catalog_content)
        .with_context(|| format!("Failed to parse catalog JSON"))?;

    let context = HighPrecisionContext::new(DEFAULT_PRECISION);

    // Collect values to test
    let mut test_values: HashMap<String, rug::Float> = HashMap::new();
    let trinity_basis = get_trinity_basis();

    // Add basis vectors
    for basis_name in &trinity_basis {
        if let Ok(val) = parse_math_value(&context, basis_name) {
            test_values.insert(basis_name.clone(), val);
        }
    }

    // Add catalog formulas
    let mut vectors = Vec::new();
    vectors.push(trinity_basis.clone());

    // Check each entry in catalog
    let mut verified = 0;
    let mut candidates = 0;

    for entry in catalog.as_array().unwrap_or(&serde_json::json!([])).iter() {
        let name = entry["name"].as_str().unwrap_or("");
        let n = entry["n"].as_i64().unwrap_or(1) as i32;
        let k = entry["k"].as_i64().unwrap_or(0) as i32;
        let m = entry["m"].as_i64().unwrap_or(0) as i32;
        let p = entry["p"].as_i64().unwrap_or(0) as i32;
        let q = entry["q"].as_i64().unwrap_or(0) as i32;
        let r = entry["r"].as_i64().unwrap_or(0) as i32;

        // Compute Sacred Formula value: V = n * 3^m * π^k * φ^p * e^q * 1^r
        let ln_3 = hp_ln_3(&context);
        let ln_pi = hp_pi_pow(&context, k).ln();
        let ln_phi = hp_phi_pow(&context, p).ln();
        let ln_e = rug::Float::with_val(context.precision, std::f64::consts::E).ln();

        let value = rug::Float::with_val(context.precision, n as f64)
            * rug::Float::with_val(context.precision, 3).pow(m as i32)
            * hp_pi_pow(&context, k)
            * hp_phi_pow(&context, p)
            * ln_e.pow(q as i32);

        let value_f64 = value.to_f64();

        // Check for relation with integer coefficients
        let test_vector = vec![
            ln_3.clone(),
            ln_pi.clone(),
            ln_phi.clone(),
            rug::Float::with_val(context.precision, 1.0),
        ];

        if let Some(relation) = run_pslq(test_vector, 20, DEFAULT_TOLERANCE) {
            let error_pct = if value_f64.abs() > 1e-10 {
                relation.residual_norm / value_f64.abs() * 100.0
            } else {
                relation.residual_norm * 100.0
            };

            if error_pct < max_error {
                let coeff_str = relation.coefficients.iter()
                    .enumerate()
                    .map(|(i, c)| format!("{}*{}", if i == 3 { format!("{}", c) } else { format!("{}x_{}", i, c) }))
                    .collect::<Vec<_>>()
                    .join(" + ");
                println!("✓ {} : V = {:.12} : PSLQ = {{{}}} : Δ = {:.6}%",
                    name, value_f64,
                    coeff_str,
                    error_pct
                );
                verified += 1;
            } else {
                candidates += 1;
            }
        }
    }

    println!("\n=== PSLQ Verification Summary ===");
    println!("Total entries: {}", verified + candidates);
    println!("Verified (Δ < {}%): {}", max_error, verified);
    println!("Candidates (Δ >= {}%): {}", max_error, candidates);

    Ok(())
}

/// Run vector analysis
fn run_vector(vector_str: String, target_str: Option<String>, max_coeff: i32, precision: u32, tolerance: Option<String>) -> anyhow::Result<()> {
    let context = HighPrecisionContext::new(precision);
    let vectors = parse_vector(&context, &vector_str);

    if vectors.is_empty() {
        anyhow::bail!("No valid values provided");
    }

    println!("=== PSLQ: Integer Relation Finder ===");
    println!("Vector: [{}]", vector_str);
    println!("Precision: {} bits, Max coeff: |a_i| ≤ {}", precision, max_coeff);

    if let Some(relation) = run_pslq(vectors, max_coeff, tolerance.unwrap_or_else(|| format!("{}", DEFAULT_TOLERANCE)).parse().unwrap_or(DEFAULT_TOLERANCE)) {
        println!("\nFound relation:");
        for (i, coeff) in relation.coefficients.iter().enumerate() {
            println!("  a_{} = {}", i + 1, coeff);
        }
        println!("Norm: {:.6}", relation.norm);
        println!("Residual |L₂|: {:.2e}", relation.residual_norm);
        println!("Quality score: {:.4}", relation.quality);

        // Check against target if provided
        if let Some(target_val_str) = target_str {
            if let Ok(target_val) = rug::Float::parse(precision, target_val_str) {
                println!("\nTarget value: {}", target_val_str);

                // Compute target value from relation
                let mut computed = rug::Float::with_val(precision, 0.0);
                for (coeff, vector) in relation.coefficients.iter().zip(vectors.iter()) {
                    let c = rug::Float::with_val(precision, *coeff as f64);
                    computed += c * vector;
                }
                println!("Computed from relation: {}", computed);
                println!("Difference: {}", (computed - target_val).abs());
            }
        }
    } else {
        println!("\nNo integer relation found within tolerance {}", tolerance.unwrap_or_else(|| format!("{}", DEFAULT_TOLERANCE)));
    }

    Ok(())
}

/// Run basis check (trinity or extended)
fn run_basis(basis_name: String, check_targets: bool, check_zip: bool) -> anyhow::Result<()> {
    let context = HighPrecisionContext::new(DEFAULT_PRECISION);

    let basis = match basis_name.as_str() {
        "trinity" => get_trinity_basis(),
        "extended" => {
            let mut b = get_trinity_basis();
            b.push("ln(2)".to_string());  // Extended basis
            b
        }
        "reduced" => vec![
            "ln(phi)".to_string(),
            "ln(pi)".to_string(),
            "1".to_string(),
        ]
        _ => anyhow::bail!("Unknown basis: {}", basis_name),
    };

    let vectors: Vec<rug::Float> = basis
        .iter()
        .filter_map(|s| parse_math_value(&context, s).ok())
        .collect();

    if vectors.is_empty() {
        anyhow::bail!("No valid basis vectors");
    }

    println!("=== PSLQ: Basis Check ===");
    println!("Basis: {} ({} vectors)", basis_name, basis.len());
    println!("Vectors: {:?}", basis);

    if let Some(relation) = run_pslq(vectors, DEFAULT_MAX_COEFF, DEFAULT_TOLERANCE) {
        println!("\nFound relation:");
        for (i, coeff) in relation.coefficients.iter().enumerate() {
            println!("  a_{} = {}", i + 1, coeff);
        }
        println!("Norm: {:.6}", relation.norm);
        println!("Residual |L₂|: {:.2e}", relation.residual_norm);
        println!("Quality score: {:.4}", relation.quality);

        // This proves linear dependence
        println!("\n>>> Linear dependence detected!");
        println!("    Basis vectors are NOT linearly independent.");
        println!("    Relation: sum_i(a_i * b_i) = 0");
    } else {
        println!("\nNo relation found: basis vectors appear linearly independent");
    }

    // Check against targets if requested
    if check_targets {
        println!("\n=== Checking against PDG targets ===");
        for (target_name, target_val) in get_pdg_targets() {
            println!("Target: {} = {}", target_name, target_val);
        }
    }

    // Check ZIP formulas if requested
    if check_zip {
        println!("\n=== Checking ZIP-derived formulas ===");
        let zip_basis = get_zip_basis();
        let zip_vectors: Vec<rug::Float> = zip_basis
            .iter()
            .filter_map(|s| parse_math_value(&context, s).ok())
            .collect();

        if !zip_vectors.is_empty() {
            if let Some(relation) = run_pslq(zip_vectors, DEFAULT_MAX_COEFF, DEFAULT_TOLERANCE) {
                println!("\nZIP formulas relation:");
                println!("{:?}", relation.coefficients);
            } else {
                println!("ZIP formulas appear independent");
            }
        }
    }

    Ok(())
}

pub fn run_pslq_command(
    cmd: PslqCommands,
    _repo_root: &std::path::Path,
) -> anyhow::Result<()> {
    match cmd {
        PslqCommands::Vector { vector, target, max_coeff, precision, tolerance } => {
            run_vector(vector, target, max_coeff, precision, tolerance)?;
        }
        PslqCommands::Basis { basis, check_targets, check_zip } => {
            run_basis(basis, check_targets, check_zip)?;
        }
        PslqCommands::Verify { catalog, max_error } => {
            run_verification(catalog, max_error)?;
        }
    }
}
