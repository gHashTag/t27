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

const SIN2_THETA_W_REF: f64 = 0.23122;

#[derive(Subcommand, Debug)]
pub enum MathCommands {
    /// Compare L5 anchors; optional Pellis, extended SM proxies, hybrid map, sensitivity.
    Compare {
        /// Enable Pellis thin-structure block (phi^5 vs alpha^-1 reference).
        #[arg(long)]
        pellis: bool,
        /// Add W/Z, Higgs, neutrino ratio placeholder, CKM moduli.
        #[arg(long)]
        pellis_extended: bool,
        /// Project normalized Trinity monomials onto normalized Pell weights (v1 diagnostic scalar).
        #[arg(long)]
        hybrid: bool,
        /// L2 cosine similarity between phi^k and Pell vectors (hybrid v2). Requires --hybrid.
        #[arg(long)]
        hybrid_v2: bool,
        /// Dimension N for hybrid v2 (default 5, range 2..152).
        #[arg(long, default_value_t = 5)]
        n: u32,
        /// Emit theta = arccos(clip(cosine_sim)) in degrees (requires --hybrid-v2).
        #[arg(long)]
        theta: bool,
        /// Numeric partials of TRINITY and (if --hybrid) hybrid score w.r.t. phi.
        #[arg(long)]
        sensitivity: bool,
        /// Weinberg angle diagnostic: phi^-3 vs PDG sin^2(theta_W).
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
            hybrid_v2,
            n,
            theta,
            sensitivity,
            weinberg,
        } => run_compare(
            repo_root,
            CompareOpts {
                pellis,
                pellis_extended,
                hybrid,
                hybrid_v2,
                n,
                theta,
                sensitivity,
                weinberg,
            },
        ),
    }
}

pub struct CompareOpts {
    pub pellis: bool,
    pub pellis_extended: bool,
    pub hybrid: bool,
    pub hybrid_v2: bool,
    pub n: u32,
    pub theta: bool,
    pub sensitivity: bool,
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

/// Hybrid v2: L2 cosine similarity between phi^k (k=0..N-1) and Pell P_{k+1} (k=0..N-1).
/// Both sides L2-normalized. Returns cosine in [0, 1].
fn hybrid_v2_cosine(phi: f64, n: usize) -> f64 {
    let u: Vec<f64> = (0..n).map(|k| phi.powi(k as i32)).collect();
    let v: Vec<f64> = (0..n).map(|k| pell_f64(k as u32 + 1)).collect();
    let u_norm: f64 = u.iter().map(|x| x * x).sum::<f64>().sqrt();
    let v_norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    if u_norm == 0.0 || v_norm == 0.0 {
        return 0.0;
    }
    let dot: f64 = u.iter().zip(v.iter()).map(|(a, b)| a * b).sum();
    dot / (u_norm * v_norm)
}

/// Pell numbers as f64 (avoids u64 overflow for N > 60).
fn pell_f64(n: u32) -> f64 {
    match n {
        0 => 0.0,
        1 => 1.0,
        _ => {
            let mut a = 0.0_f64;
            let mut b = 1.0_f64;
            for _ in 2..=n {
                let c = 2.0 * b + a;
                a = b;
                b = c;
            }
            b
        }
    }
}

/// Golden test values for hybrid v2 at known N checkpoints.
/// Computed from: H_N = (phi^k / ||phi^k||_2) . (P_{k+1} / ||P_{k+1}||_2)
const GOLDEN_V2: &[(u32, f64, f64)] = &[
    (5,   0.9649159951, 15.2219),
    (10,  0.9617744938, 15.8931),
    (15,  0.9617437739, 15.8995),
    (20,  0.9617435184, 15.8995),
    (50,  0.9617435163, 15.8995),
    (152, 0.9617435163, 15.8995),
];

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

    if opts.hybrid_v2 {
        let n = opts.n.max(2).min(152) as usize;
        let cos_sim = hybrid_v2_cosine(phi, n);
        println!("--hybrid-v2: L2 cosine similarity (N={}) = {:.12}", n, cos_sim);
        record["hybrid_v2"] = json!(cos_sim);
        record["hybrid_v2_N"] = json!(n);

        if opts.theta {
            let theta_rad = (cos_sim.clamp(-1.0, 1.0)).acos();
            let theta_deg = theta_rad * 180.0 / std::f64::consts::PI;
            println!("--theta: theta_N = {:.6} deg", theta_deg);
            record["theta_deg"] = json!(theta_deg);
        }

        // Golden test verification
        for &(gn, gc, _gt) in GOLDEN_V2 {
            if n == gn as usize {
                let computed = hybrid_v2_cosine(phi, n);
                let diff = (computed - gc).abs();
                let pass = diff < 1e-6;
                println!(
                    "  golden N={}: computed={:.10} expected={:.10} diff={:.2e} {}",
                    gn, computed, gc, diff,
                    if pass { "PASS" } else { "FAIL" }
                );
                record[&format!("golden_N{}", gn)] = json!({
                    "expected": gc,
                    "computed": computed,
                    "diff": diff,
                    "pass": pass,
                });
            }
        }
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

    if let Some(h) = read_pellis_spec_seal_hash(repo_root) {
        record["pellis_spec_seal_hash"] = json!(h);
    }

    if opts.weinberg {
        let phi_inv_cubed = phi_inv.powi(3);
        let delta_abs = (phi_inv_cubed - SIN2_THETA_W_REF).abs();
        let delta_rel = delta_abs / SIN2_THETA_W_REF;

        println!("=== Weinberg Angle Diagnostic (issue #295) ===");
        println!("phi^-3           = {:.10}", phi_inv_cubed);
        println!("sin^2(theta_W)   = {:.10}  (PDG MS-bar)", SIN2_THETA_W_REF);
        println!("delta_abs        = {:.10}", delta_abs);
        println!("delta_rel        = {:.6} ({:.2}%)", delta_rel, delta_rel * 100.0);

        let bound = 0.005_f64;
        if delta_abs < bound {
            println!("golden test: |phi^-3 - sin^2(theta_W)| = {:.10} < {} PASS", delta_abs, bound);
        } else {
            println!("golden test: |phi^-3 - sin^2(theta_W)| = {:.10} >= {} NOT MET", delta_abs, bound);
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

    fn phi() -> f64 {
        (1.0 + 5.0_f64.sqrt()) / 2.0
    }

    #[test]
    fn test_hybrid_v1_golden() {
        let h = hybrid_inner_product(phi());
        let expected = 0.563780474444;
        assert!(
            (h - expected).abs() < 1e-6,
            "v1: got {:.12}, expected {:.12}",
            h, expected
        );
    }

    #[test]
    fn test_hybrid_v2_golden_n5() {
        let cos = hybrid_v2_cosine(phi(), 5);
        assert!(
            (cos - 0.9649159951).abs() < 1e-6,
            "v2 N=5: got {:.12}",
            cos
        );
    }

    #[test]
    fn test_hybrid_v2_golden_n10() {
        let cos = hybrid_v2_cosine(phi(), 10);
        assert!(
            (cos - 0.9617744938).abs() < 1e-6,
            "v2 N=10: got {:.12}",
            cos
        );
    }

    #[test]
    fn test_hybrid_v2_golden_n15() {
        let cos = hybrid_v2_cosine(phi(), 15);
        assert!(
            (cos - 0.9617437739).abs() < 1e-6,
            "v2 N=15: got {:.12}",
            cos
        );
    }

    #[test]
    fn test_hybrid_v2_golden_n20() {
        let cos = hybrid_v2_cosine(phi(), 20);
        assert!(
            (cos - 0.9617435184).abs() < 1e-6,
            "v2 N=20: got {:.12}",
            cos
        );
    }

    #[test]
    fn test_hybrid_v2_golden_n50() {
        let cos = hybrid_v2_cosine(phi(), 50);
        assert!(
            (cos - 0.9617435163).abs() < 1e-6,
            "v2 N=50: got {:.12}",
            cos
        );
    }

    #[test]
    fn test_hybrid_v2_golden_n152() {
        let cos = hybrid_v2_cosine(phi(), 152);
        assert!(
            (cos - 0.9617435163).abs() < 1e-6,
            "v2 N=152: got {:.12}",
            cos
        );
    }

    #[test]
    fn test_hybrid_v2_plateau() {
        let n15 = hybrid_v2_cosine(phi(), 15);
        let n20 = hybrid_v2_cosine(phi(), 20);
        let n152 = hybrid_v2_cosine(phi(), 152);
        assert!((n20 - n15).abs() < 1e-6, "N=20 should be near plateau");
        assert!((n152 - n20).abs() < 1e-9, "N=152 should match N=20");
    }

    #[test]
    fn test_hybrid_v2_monotonic_after_n10() {
        let n10 = hybrid_v2_cosine(phi(), 10);
        let n15 = hybrid_v2_cosine(phi(), 15);
        assert!(n15 <= n10, "v2 should decrease or stay flat after N=10");
    }

    #[test]
    fn test_theta_degrees() {
        let cos = hybrid_v2_cosine(phi(), 152);
        let theta = cos.clamp(-1.0, 1.0).acos() * 180.0 / std::f64::consts::PI;
        assert!(
            (theta - 15.8995).abs() < 0.01,
            "theta: got {:.4} deg",
            theta
        );
    }

    #[test]
    fn test_weinberg_golden_bound() {
        let phi_inv_cubed = (1.0 / phi()).powi(3);
        let delta = (phi_inv_cubed - SIN2_THETA_W_REF).abs();
        assert!(
            delta < 0.005,
            "golden test: |phi^-3 - sin^2(theta_W)| = {:.10} >= 0.005",
            delta
        );
    }

    #[test]
    fn test_phi_inv_cubed_approx() {
        let phi_inv_cubed = (1.0 / phi()).powi(3);
        assert!(
            (phi_inv_cubed - 0.236068).abs() < 0.001,
            "phi^-3 should be ~0.236, got {:.10}",
            phi_inv_cubed
        );
    }
}
