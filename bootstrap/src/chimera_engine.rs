//! Chimera Engine: find new formulas by combining existing ones.

use std::collections::HashMap;

pub enum ChimeraOp {
    Mul,
    Div,
    Add,
    Sub,
    Sin,   // sin(x)
    Cos,   // cos(x)
    Log,   // ln(x), logₑ(x)
    Exp,   // eˣ
    Pow,   // xⁿ (integer exponents only)
}

pub struct ChimeraCandidate {
    pub expr: String,
    pub target_name: String,
    pub target_value: f64,
    pub chimera_value: f64,
    pub error_pct: f64,
    pub status: String,
}

/// Run chimera search on base formulas
pub fn chimera_search(
    base_formulas: &[(&str, f64)],
    operators: &[ChimeraOp],
    targets: &HashMap<String, f64>,
    threshold: f64,
) -> Vec<ChimeraCandidate> {
    let mut results = Vec::new();

    for (i, (f1_id, f1_val)) in base_formulas.iter().enumerate() {
        for (f2_id, f2_val) in base_formulas.iter().skip(i + 1) {
            for op in operators {
                let chimera_val = match op {
                    ChimeraOp::Mul => f1_val * f2_val,
                    ChimeraOp::Div => {
                        if f2_val.abs() < 1e-15 {
                            continue; // Skip division by near-zero
                        }
                        f1_val / f2_val
                    }
                    ChimeraOp::Add => f1_val + f2_val,
                    ChimeraOp::Sub => f1_val - f2_val,
                    ChimeraOp::Sin => f1_val.sin(),
                    ChimeraOp::Cos => f1_val.cos(),
                    ChimeraOp::Log => {
                        if *f1_val <= 0.0 {
                            continue; // Skip log of non-positive values
                        }
                        f1_val.ln()
                    }
                    ChimeraOp::Exp => f1_val.exp(),
                    ChimeraOp::Pow => f1_val.powf(*f2_val),
                };

                let op_str = match op {
                    ChimeraOp::Mul => "*",
                    ChimeraOp::Div => "/",
                    ChimeraOp::Add => "+",
                    ChimeraOp::Sub => "-",
                    ChimeraOp::Sin => "sin",
                    ChimeraOp::Cos => "cos",
                    ChimeraOp::Log => "ln",
                    ChimeraOp::Exp => "exp",
                    ChimeraOp::Pow => "^",
                };

                for (target_name, target_val) in targets {
                    let error_pct = if target_val.abs() > 1e-15 {
                        (chimera_val - target_val).abs() / target_val.abs() * 100.0
                    } else {
                        (chimera_val - target_val).abs() * 100.0
                    };

                    if error_pct < threshold {
                        let status = if error_pct < 0.1 {
                            "APPROX"
                        } else if error_pct < 5.0 {
                            "CANDIDATE"
                        } else {
                            "FOUND"
                        };

                        results.push(ChimeraCandidate {
                            expr: format!("{} {} {}", f1_id, op_str, f2_id),
                            target_name: target_name.clone(),
                            target_value: *target_val,
                            chimera_value: chimera_val,
                            error_pct,
                            status: status.to_string(),
                        });
                    }
                }
            }
        }
    }

    results.sort_by(|a, b| a.error_pct.partial_cmp(&b.error_pct).unwrap());
    results
}

/// PDG 2024 target constants for chimera search
pub fn pdg_targets() -> HashMap<String, f64> {
    let mut targets = HashMap::new();
    targets.insert("V_us".to_string(), 0.22431);
    targets.insert("V_td".to_string(), 0.00868);
    targets.insert("V_ub".to_string(), 0.0037);
    targets.insert("V_ud".to_string(), 0.97435);
    targets.insert("V_cs".to_string(), 0.97548);
    targets.insert("sin2th12".to_string(), 0.307);
    targets.insert("sin2th13".to_string(), 0.02195);
    targets.insert("W_mass".to_string(), 80.377);
    targets.insert("Z_mass".to_string(), 91.1876);
    targets.insert("top_mass".to_string(), 172.69);
    targets
}

/// Get base formula values for chimera search
pub fn base_formula_values() -> Vec<(&'static str, f64)> {
    vec![
        ("gamma", 0.23607),
        ("alpha_s", 0.118034),
        ("delta_CP", 196.965),
        ("sin2th12", 0.307023),
        ("sin2th23", 0.545985),
        ("V_cb", 0.04133),
    ]
}

/// Default operators for chimera search
pub fn default_operators() -> Vec<ChimeraOp> {
    vec![
        ChimeraOp::Mul,
        ChimeraOp::Div,
        ChimeraOp::Add,
        ChimeraOp::Sub,
        ChimeraOp::Sin,
        ChimeraOp::Cos,
        ChimeraOp::Log,
        ChimeraOp::Exp,
        ChimeraOp::Pow,
    ]
}

/// Generate all possible φ·π·e combinations up to max_pow
pub fn generate_basis(max_pow: i32) -> Vec<(String, f64)> {
    let mut basis = Vec::new();
    let phi = 1.6180339887498948_f64;
    let pi = std::f64::consts::PI;
    let e = std::f64::consts::E;

    for i in -max_pow..=max_pow {
        for j in -max_pow..=max_pow {
            for k in -max_pow..=max_pow {
                // n·φⁱ·πʲ·eᵏ
                let val = 1.0_f64 * phi.powi(i) * pi.powi(j) * e.powi(k);
                basis.push((format!("φ^{}π^{}e^{}", i, j, k), val));
            }
        }
    }

    basis
}
