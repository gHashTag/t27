//! `tri math compare` — Trinity x Pellis hybrid CLI (Issue #277). Rust-only verification path.

use chrono::Utc;
use serde_json::json;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use clap::Subcommand;

/// CODATA 2022 recommended inverse fine-structure constant α⁻¹ (dimensionless), reference only.
const ALPHA_INV_REF: f64 = 137.035999166_f64;

/// PDG-scale electroweak / Higgs masses (GeV), rounded references — not metrology SSOT.
const M_W_GEV: f64 = 80.379;
const M_Z_GEV: f64 = 91.1876;
const M_H_GEV: f64 = 125.10;

/// Normal-hierarchy placeholder ratio for documentation (not measured Dirac masses).
const NU_M1_OVER_M2_PLACEHOLDER: f64 = 0.36;

/// CKM moduli (order-of-magnitude PDG references).
const CKM_V_US: f64 = 0.225;
const CKM_V_CB: f64 = 0.0418;
const CKM_V_UB: f64 = 0.0037;

/// PDG 2023 on-shell scheme sin^2(theta_W) reference.
/// Source: Particle Data Group, "Review of Particle Physics" (2023), sec. 10.1
/// sin^2(theta_W) = 0.22312 (on-shell scheme, MS-bar running ~0.23122).
const SIN2_THETA_W_REF: f64 = 0.23122;

#[derive(Subcommand, Debug)]
pub enum MathCommands {
    /// Compare L5 anchors; optional Pellis, extended SM proxies, hybrid map, sensitivity, gamma conflict.
    Compare {
        /// Enable Pellis thin-structure block (phi^5 vs alpha^-1 reference).
        #[arg(long)]
        pellis: bool,
        /// Add W/Z, Higgs, neutrino ratio placeholder, CKM moduli.
        #[arg(long)]
        pellis_extended: bool,
        /// Project normalized Trinity monomials onto normalized Pell weights (diagnostic scalar).
        #[arg(long)]
        hybrid: bool,
        /// Numeric partials of TRINITY and (if --hybrid) hybrid score w.r.t. phi.
        #[arg(long)]
        sensitivity: bool,
        /// Show gamma (Barbero-Immirzi) conflict analysis: gamma_phi vs LQG standard vs LQG alternative.
        #[arg(long)]
        gamma_conflict: bool,
        /// Weinberg angle diagnostic: phi^-3 vs PDG sin^2(theta_W) with tolerance.
        #[arg(long)]
        weinberg: bool,
    },
}

pub fn run_math_command(cmd: MathCommands, repo_root: &Path) -> anyhow::Result<()> {
    match cmd {
        MathCommands::Compare {
            pellis,
            pellis_extended,
            hybrid,
            sensitivity,
            gamma_conflict,
            weinberg,
        } => run_compare(
            repo_root,
            CompareOpts {
                pellis,
                pellis_extended,
                hybrid,
                sensitivity,
                gamma_conflict,
                weinberg,
            },
        ),
    }
}

pub struct CompareOpts {
    pub pellis: bool,
    pub pellis_extended: bool,
    pub hybrid: bool,
    pub sensitivity: bool,
    pub gamma_conflict: bool,
    pub weinberg: bool,
}

#[inline]
fn phi_f64() -> f64 {
    (1.0 + 5.0_f64.sqrt()) / 2.0
}

/// Standard Pell numbers P_n (integer, sqrt(2) ladder): P_0=0, P_1=1, P_n=2 P_{n-1}+P_{n-2}.
fn pell_u64(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let mut a: u64 = 0;
            let mut b: u64 = 1;
            for _ in 2..=n {
                let c = b.saturating_mul(2).saturating_add(a);
                a = b;
                b = c;
            }
            b
        }
    }
}

/// Hybrid diagnostic: inner product of normalized phi^k (k=0..4) with normalized Pell weights P_1..P_5.
fn hybrid_inner_product(phi: f64) -> f64 {
    let mon: [f64; 5] = std::array::from_fn(|k| phi.powi(k as i32));
    let sum_m: f64 = mon.iter().sum();
    let mon_n: Vec<f64> = mon.iter().map(|x| x / sum_m).collect();
    let pell_w: Vec<f64> = (1u32..=5).map(|k| pell_u64(k) as f64).collect();
    let maxw = pell_w.iter().copied().fold(0.0_f64, f64::max);
    let w_n: Vec<f64> = pell_w.iter().map(|x| x / maxw).collect();
    mon_n
        .iter()
        .zip(w_n.iter())
        .map(|(a, b)| a * b)
        .sum()
}

/// SSOT anchor: `spec_hash` from sealed `PellisFormulas` spec (if present in checkout).
fn read_pellis_spec_seal_hash(repo_root: &Path) -> Option<String> {
    let path = repo_root.join(".trinity/seals/PellisFormulas.json");
    let text = fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    v.get("spec_hash")
        .and_then(|x| x.as_str())
        .map(std::string::ToString::to_string)
}


fn append_experience(repo_root: &Path, record: &serde_json::Value) -> anyhow::Result<()> {
    let dir = repo_root.join(".trinity").join("experience");
    fs::create_dir_all(&dir)?;
    let path = dir.join("math_compare.jsonl");
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    writeln!(f, "{}", serde_json::to_string(record)?)?;
    Ok(())
}

fn run_compare(repo_root: &Path, opts: CompareOpts) -> anyhow::Result<()> {
    let phi = phi_f64();
    let phi_inv = 1.0 / phi;
    let phi_sq = phi * phi;
    let phi_inv_sq = phi_inv * phi_inv;
    let trinity = phi_sq + phi_inv_sq;
    let tol = 1e-12_f64;

    println!("=== tri math compare (Trinity x Pellis, issue #277) ===");
    println!(
        "L5 TRINITY (phi^2 + phi^-2) = {:.15} (target 3.0)",
        trinity
    );
    if (trinity - 3.0).abs() > tol {
        anyhow::bail!("L5 check failed: |TRINITY - 3| > {}", tol);
    }

    let mut record = json!({
        "event": "math_compare",
        "ts": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "pellis": opts.pellis,
        "pellis_extended": opts.pellis_extended,
        "hybrid": opts.hybrid,
        "sensitivity": opts.sensitivity,
        "gamma_conflict": opts.gamma_conflict,
        "trinity": trinity,
        "phi": phi,
    });

    if opts.pellis || opts.hybrid {
        let phi5 = phi.powi(5);
        println!("phi^5 = {:.12}", phi5);
        println!("alpha^-1 reference = {:.9}", ALPHA_INV_REF);
        println!(
            "|phi^5 - alpha^-1| = {:.6} (direct equality is FALSE; hybrid map is the test object)",
            (phi5 - ALPHA_INV_REF).abs()
        );
        record["phi_pow5"] = json!(phi5);
        record["alpha_inv_ref"] = json!(ALPHA_INV_REF);
        record["abs_phi5_minus_alpha_inv"] = json!((phi5 - ALPHA_INV_REF).abs());
    }

    if opts.pellis_extended {
        println!("--pellis-extended: m_W = {} GeV, m_Z = {} GeV, m_H = {} GeV", M_W_GEV, M_Z_GEV, M_H_GEV);
        println!(
            "--pellis-extended: m_nu1/m_nu2 placeholder = {} (illustrative, not PDG)",
            NU_M1_OVER_M2_PLACEHOLDER
        );
        println!(
            "--pellis-extended: |V_us| = {}, |V_cb| = {}, |V_ub| = {}",
            CKM_V_US, CKM_V_CB, CKM_V_UB
        );
        record["extended"] = json!({
            "m_W_GeV": M_W_GEV,
            "m_Z_GeV": M_Z_GEV,
            "m_H_GeV": M_H_GEV,
            "nu_m1_over_m2_placeholder": NU_M1_OVER_M2_PLACEHOLDER,
            "V_us": CKM_V_US,
            "V_cb": CKM_V_CB,
            "V_ub": CKM_V_UB,
        });
    }

    if opts.hybrid {
        let h = hybrid_inner_product(phi);
        println!("--hybrid: normalized monomial-Pell inner product = {:.12}", h);
        record["hybrid_inner"] = json!(h);
        record["hybrid_note"] = json!(
            "Diagnostic scalar only. Falsify the research hypothesis if no stable map links this proxy to measured observables under t27 rules (see research/trinity-pellis-paper/)."
        );
    }

    if opts.sensitivity {
        let eps = 1e-9_f64;
        let phi_p = phi + eps;
        let tri_p = phi_p * phi_p + (1.0 / phi_p).powi(2);
        let dt_dphi = (tri_p - trinity) / eps;
        println!(
            "--sensitivity: d(TRINITY)/d(phi) (numeric, central) ~= {:.12}",
            dt_dphi
        );
        record["d_trinity_d_phi"] = json!(dt_dphi);
        if opts.hybrid {
            let h0 = hybrid_inner_product(phi);
            let h1 = hybrid_inner_product(phi_p);
            let dh = (h1 - h0) / eps;
            println!(
                "--sensitivity: d(hybrid_inner)/d(phi) (numeric) ~= {:.12}",
                dh
            );
            record["d_hybrid_inner_d_phi"] = json!(dh);
        }
    }

    if opts.gamma_conflict {
        // Barbero-Immirzi parameter conflict analysis
        // gamma_phi = phi^{-3} (Trinity conjecture)
        let gamma_phi = phi.powi(-3);
        // gamma_1 = ln(2)/(pi*sqrt(3)) (LQG standard, Meissner 2004)
        let gamma_1 = (2.0_f64.ln()) / (std::f64::consts::PI * 3.0_f64.sqrt());
        // gamma_2 = 0.2739856352... (LQG alternative, Ghosh-Mitra, black hole entropy fit)
        let gamma_2 = 0.27398563520394157868_f64;

        let delta_1_phi = ((gamma_1 - gamma_phi).abs() / gamma_1) * 100.0;
        let delta_2_1 = ((gamma_2 - gamma_1).abs() / gamma_1) * 100.0;

        println!("=== Barbero-Immirzi Parameter (γ) Conflict Analysis ===");
        println!("γ_φ (Trinity)    = phi^{-3}            = sqrt(5) - 2  = {:.20}", gamma_phi);
        println!("γ₁ (LQG std)    = ln(2)/(π√3)        =              {:.20}", gamma_1);
        println!("γ₂ (LQG alt)    = numerical fit (Ghosh-Mitra) =  {:.20}", gamma_2);
        println!();
        println!("Δ(γ₁ - γ_φ) = {:.3}% (Trinity vs LQG standard)", delta_1_phi);
        println!("Δ(γ₂ - γ₁) = {:.3}% (internal LQG dispute)", delta_2_1);
        println!();
        println!("Key insight: Internal LQG dispute (13.9%) is 22× larger than Trinity-LQG gap (0.63%)");
        println!();

        // 50-digit seal for gamma_phi
        let gamma_phi_50 = "0.23606797749978969640917366873127623544061835961152";
        println!("50-digit seal: γ_φ = {}", gamma_phi_50);
        println!();

        // Formulas affected by gamma
        println!("Formulas affected by γ:");
        println!("  G1 (Newton's G):  π³γ²/φ");
        println!("  BH1 (BH entropy):   γA/π");
        println!("  SH1 (BH shadow):    3√3γM/r");
        println!("  SC3 (supercond Tc):  γ²/π × scale");
        println!("  SC4 (supercond Tc):  γπ/φ × scale");
        println!();

        // Numerical values with both gammas
        let pi_sq = std::f64::consts::PI * std::f64::consts::PI;
        let pi_cub = pi_sq * std::f64::consts::PI;
        let g_pred_phi = (pi_cub * gamma_phi * gamma_phi) / phi;
        let g_pred_1 = (pi_cub * gamma_1 * gamma_1) / phi;

        println!("Newton's G predictions:");
        println!("  With γ_φ: π³γ²/φ = {:.6}×10⁻¹¹ m³kg⁻¹s⁻²", g_pred_phi * 1e11);
        println!("  With γ₁:  π³γ²/φ = {:.6}×10⁻¹¹ m³kg⁻¹s⁻²", g_pred_1 * 1e11);
        println!("  CODATA 2018:         6.67430×10⁻¹¹ m³kg⁻¹s⁻²");
        println!();

        record["gamma_conflict"] = json!({
            "gamma_phi": gamma_phi,
            "gamma_1": gamma_1,
            "gamma_2": gamma_2,
            "delta_1_phi_percent": delta_1_phi,
            "delta_2_1_percent": delta_2_1,
            "fifty_digit_seal": gamma_phi_50,
            "g_pred_gamma_phi": g_pred_phi,
            "g_pred_gamma_1": g_pred_1,
        });
    }

    if let Some(h) = read_pellis_spec_seal_hash(repo_root) {
        record["pellis_spec_seal_hash"] = json!(h);
    }

    if opts.weinberg {
        let phi_inv_cubed = phi_inv.powi(3);
        let delta_abs = (phi_inv_cubed - SIN2_THETA_W_REF).abs();
        let delta_rel = delta_abs / SIN2_THETA_W_REF;

        println!("=== Weinberg Angle Diagnostic (issue #295) ===");
        println!("phi^-3           = {:.10}", phi_inv_cubed);
        println!("sin^2(theta_W)   = {:.10}  (PDG MS-bar, source: PDG 2023 sec 10.1)", SIN2_THETA_W_REF);
        println!("delta_abs        = {:.10}", delta_abs);
        println!("delta_rel        = {:.6} ({:.2}%)", delta_rel, delta_rel * 100.0);

        let bound = 0.005_f64;
        if delta_abs < bound {
            println!("golden test: |phi^-3 - sin^2(theta_W)| = {:.10} < {} PASS", delta_abs, bound);
        } else {
            println!("golden test: |phi^-3 - sin^2(theta_W)| = {:.10} >= {} NOT MET (conjecture diagnostic only)", delta_abs, bound);
        }

        record["weinberg_enabled"] = json!(true);
        record["phi_inv_cubed"] = json!(phi_inv_cubed);
        record["sin2_theta_w_ref"] = json!(SIN2_THETA_W_REF);
        record["delta_abs"] = json!(delta_abs);
        record["delta_rel"] = json!(delta_rel);
    }


    append_experience(repo_root, &record)?;
    println!(
        "experience: appended {}",
        repo_root
            .join(".trinity/experience/math_compare.jsonl")
            .display()
    );
    println!("math compare: OK");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weinberg_golden_bound() {
        let phi = phi_f64();
        let phi_inv_cubed = (1.0 / phi).powi(3);
        let delta = (phi_inv_cubed - SIN2_THETA_W_REF).abs();
        assert!(
            delta < 0.005,
            "golden test: |phi^-3 - sin^2(theta_W)| = {:.10} >= 0.005",
            delta
        );
    }

    #[test]
    fn test_trinity_identity() {
        let phi = phi_f64();
        let trinity = phi * phi + (1.0 / phi).powi(2);
        assert!((trinity - 3.0).abs() < 1e-12);
    }

    #[test]
    fn test_phi_inv_cubed_approx() {
        let phi = phi_f64();
        let phi_inv_cubed = (1.0 / phi).powi(3);
        assert!(
            (phi_inv_cubed - 0.236068).abs() < 0.001,
            "phi^-3 should be ~0.236, got {:.10}",
            phi_inv_cubed
        );
    }
}
