//! Sensitivity Analysis: formula response to parameter variations.

use crate::formula_eval::find_pdg_reference;

pub struct SensitivityPoint {
    pub param_value: f64,
    pub formula_value: f64,
    pub error_pct: f64,
}

/// Run sensitivity scan for a formula
pub fn sensitivity_scan(
    formula_id: &str,
    param_name: &str,
    range: (f64, f64),
    n_points: usize,
) -> Vec<SensitivityPoint> {
    let mut results = Vec::new();

    if n_points == 0 {
        return results;
    }

    let step = if n_points > 1 {
        (range.1 - range.0) / (n_points - 1) as f64
    } else {
        0.0
    };

    for i in 0..n_points {
        let param_val = range.0 + i as f64 * step;

        // Evaluate formula with modified parameter
        let formula_val = evaluate_formula_with_params(formula_id, &[(param_name, param_val)]);

        // Get reference PDG value
        let pdg_ref = get_pdg_reference(formula_id);

        let error_pct = if let Some(pdg) = pdg_ref {
            if pdg.abs() > 1e-15 {
                (formula_val - pdg).abs() / pdg.abs() * 100.0
            } else {
                (formula_val - pdg).abs() * 100.0
            }
        } else {
            0.0
        };

        results.push(SensitivityPoint {
            param_value: param_val,
            formula_value: formula_val,
            error_pct,
        });
    }

    results
}

/// Find the parameter value that minimizes error
pub fn find_minimum(points: &[SensitivityPoint]) -> Option<&SensitivityPoint> {
    points.iter().min_by(|a, b| a.error_pct.partial_cmp(&b.error_pct).unwrap())
}

fn get_pdg_reference(formula_id: &str) -> Option<f64> {
    find_pdg_reference(formula_id).map(|(v, _)| v)
}

/// Evaluate formula with overridden parameters
fn evaluate_formula_with_params(formula_id: &str, params: &[(&str, f64)]) -> f64 {
    match formula_id {
        "delta_CP" => {
            // δ_CP = 9*φ^(-2)*180/π
            let phi = params.iter().find(|(n, _)| *n == "phi").map(|(_, v)| *v).unwrap_or(1.6180339887498948);
            let pi = params.iter().find(|(n, _)| *n == "pi").map(|(_, v)| *v).unwrap_or(std::f64::consts::PI);
            9.0 * phi.powi(-2) * 180.0 / pi
        }
        "gamma" => {
            let phi = params.iter().find(|(n, _)| *n == "phi").map(|(_, v)| *v).unwrap_or(1.6180339887498948);
            phi.powi(-3)
        }
        "sin2th12" => {
            let phi = params.iter().find(|(n, _)| *n == "phi").map(|(_, v)| *v).unwrap_or(1.6180339887498948);
            let pi = params.iter().find(|(n, _)| *n == "pi").map(|(_, v)| *v).unwrap_or(std::f64::consts::PI);
            let e = params.iter().find(|(n, _)| *n == "e").map(|(_, v)| *v).unwrap_or(std::f64::consts::E);
            7.0 * phi.powf(5.0) / (3.0 * pi.powf(3.0) * e)
        }
        "sin2th23" => {
            let phi = params.iter().find(|(n, _)| *n == "phi").map(|(_, v)| *v).unwrap_or(1.6180339887498948);
            let pi = params.iter().find(|(n, _)| *n == "pi").map(|(_, v)| *v).unwrap_or(std::f64::consts::PI);
            let e = params.iter().find(|(n, _)| *n == "e").map(|(_, v)| *v).unwrap_or(std::f64::consts::E);
            4.0 * pi * phi.powf(2.0) / (3.0 * e.powf(3.0))
        }
        "alpha_s" => {
            let phi = params.iter().find(|(n, _)| *n == "phi").map(|(_, v)| *v).unwrap_or(1.6180339887498948);
            1.0 / (phi.powf(4.0) + phi)
        }
        "mH_mZ" => {
            let phi = params.iter().find(|(n, _)| *n == "phi").map(|(_, v)| *v).unwrap_or(1.6180339887498948);
            let pi = params.iter().find(|(n, _)| *n == "pi").map(|(_, v)| *v).unwrap_or(std::f64::consts::PI);
            let e = params.iter().find(|(n, _)| *n == "e").map(|(_, v)| *v).unwrap_or(std::f64::consts::E);
            (1.0 / 8.0) * phi.powf(2.0) * pi.powf(3.0) * e.powf(-2.0)
        }
        "V_cb" => {
            let phi = params.iter().find(|(n, _)| *n == "phi").map(|(_, v)| *v).unwrap_or(1.6180339887498948);
            let pi = params.iter().find(|(n, _)| *n == "pi").map(|(_, v)| *v).unwrap_or(std::f64::consts::PI);
            let e = params.iter().find(|(n, _)| *n == "e").map(|(_, v)| *v).unwrap_or(std::f64::consts::E);
            (1.0 / 7.0) * phi.powf(-2.0) * pi.powf(-2.0) * e.powf(2.0)
        }
        "V_us" => {
            let phi = params.iter().find(|(n, _)| *n == "phi").map(|(_, v)| *v).unwrap_or(1.6180339887498948);
            let pi = params.iter().find(|(n, _)| *n == "pi").map(|(_, v)| *v).unwrap_or(std::f64::consts::PI);
            3.0 * phi.powf(-3.0) / pi
        }
        "trinity" => {
            let phi = params.iter().find(|(n, _)| *n == "phi").map(|(_, v)| *v).unwrap_or(1.6180339887498948);
            phi.powf(2.0) + 1.0 / phi.powf(2.0)
        }
        _ => 0.0,
    }
}

/// Get default parameter range for a given parameter
pub fn default_param_range(param_name: &str) -> (f64, f64) {
    match param_name {
        "phi" => (1.61, 1.625),
        "pi" => (3.14, 3.143),
        "e" => (2.717, 2.72),
        _ => (0.9, 1.1),
    }
}
