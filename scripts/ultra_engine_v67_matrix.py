#!/usr/bin/env python3
"""
ULTRA ENGINE v6.7 — MATRIX SEARCH (2x2, 3x3, NxN formula matrices)
Discover ALL matrix combinations of Trinity formulas
"""

import numpy as np
import time
import itertools
from datetime import datetime
from collections import defaultdict

PHI = 1.6180339887498948
PI = np.pi
E = np.e

# Base formula values from FORMULA_TABLE_v07
BASE_FORMULAS = {
    "gamma": PHI**-3,
    "alpha_inv": 137.035999,
    "alpha_s": 1.0 / (PHI**4 + PHI),
    "theta_C": 13.02 * PI / 180,
    "V_cb": (1/7) * PHI**-2 * PI**-2 * E**2,
    "sin2th23": 4 * PI * PHI**2 / (3 * E**3),
    "delta_CP": 9 * PHI**-2,
    "mH_mZ": (1/8) * PHI**2 * PI**3 * E**-2,
    "V_ud": 7 * PHI**-5 * PI**3 * E**-3,
    "V_cs": 7 * PHI**-5 * PI**3 * E**-3,
    "V_td": 2 * PHI**-4 * PI**-4 * E,
    "sin2th12_chimera": 8 * PHI**-5 * PI * E**-2,
    "delta_CP_rad": 9 * PHI**-2,
}

# PDG 2024 targets
TARGETS = {
    "W_mass": 80.377,
    "Z_mass": 91.1876,
    "H_mass": 125.25,
    "top_mass": 172.69,
    "bottom_mass": 4.18,
    "charm_mass": 1.27,
    "strange_mass": 0.095,
    "tau_mass": 1.77686,
    "muon_mass": 0.105658,
    "electron_mass": 0.000511,
    "alpha_em": 1/137.035999084,
    "alpha_s": 0.1184,
    "V_us": 0.22431,
    "V_ud": 0.97435,
    "V_cs": 0.97548,
    "V_cb": 0.0408,
    "V_ub": 0.0037,
    "V_td": 0.00814,
    "theta12": np.deg2rad(33.44),
    "theta13": np.deg2rad(8.61),
    "theta23": np.deg2rad(49.3),
    "G_F": 1.1663787e-5,
    "R_inf": 0.5,
    "Omega_Lambda": 0.6889,
    "Delta_cp": np.deg2rad(68.0),
}

def matrix_det_2x2(a, b, c, d):
    """2x2 matrix determinant"""
    return a * d - b * c

def matrix_trace_2x2(a, b, c, d):
    """2x2 matrix trace"""
    return a + d

def matrix_det_3x3(m):
    """3x3 matrix determinant"""
    return (m[0]*(m[4]*m[8] - m[5]*m[7]) -
            m[1]*(m[3]*m[8] - m[5]*m[6]) +
            m[2]*(m[3]*m[7] - m[4]*m[6]))

def matrix_trace_3x3(m):
    """3x3 matrix trace"""
    return m[0] + m[4] + m[8]

def search_2x2_matrices(base_formulas, targets, threshold):
    """Search all 2x2 matrix combinations"""
    results = []
    formulas = list(base_formulas.items())

    print("  Searching 2x2 matrices...")
    start = time.time()

    count = 0
    total = len(formulas)**4

    for (n1, v1), (n2, v2), (n3, v3), (n4, v4) in itertools.product(formulas, repeat=4):
        count += 1
        if count % 100000 == 0:
            print(f"    Progress: {count}/{total} ({100*count/total:.1f}%)")

        # Matrix: [[v1, v2], [v3, v4]]
        mat = [v1, v2, v3, v4]

        # Compute matrix invariants
        det = matrix_det_2x2(*mat)
        trace = matrix_trace_2x2(*mat)
        frobenius = np.sqrt(v1**2 + v2**2 + v3**2 + v4**2)

        invariants = {
            "det": det,
            "trace": trace,
            "frobenius": frobenius,
            "det+trace": det + trace,
            "det*trace": det * trace,
            "det/trace": det / trace if trace != 0 else 0,
        }

        # Check against targets
        for inv_name, inv_val in invariants.items():
            for target_name, target_val in targets.items():
                if target_val == 0:
                    continue
                error = abs(inv_val - target_val) / target_val * 100
                if error < threshold:
                    results.append({
                        "matrix_type": "2x2",
                        "elements": [n1, n2, n3, n4],
                        "invariant": inv_name,
                        "value": inv_val,
                        "target": target_name,
                        "error": error
                    })

    elapsed = time.time() - start
    print(f"  2x2 search complete: {elapsed:.2f}s, {len(results)} results")
    return results

def search_formula_combinations(base_formulas, targets, threshold, max_terms=4):
    """Search n-ary combinations of formulas with operations"""
    results = []
    formulas = list(base_formulas.items())
    operations = [
        lambda a, b: a * b,
        lambda a, b: a / b if b != 0 else 0,
        lambda a, b: a + b,
        lambda a, b: a - b,
        lambda a, b: a ** b if a > 0 else 0,
    ]
    op_names = ["*", "/", "+", "-", "^"]

    print(f"  Searching {max_terms}-term combinations...")
    start = time.time()

    for n_terms in range(2, max_terms + 1):
        print(f"    {n_terms}-term combinations...")

        for combo in itertools.combinations(formulas, n_terms):
            values = [v for _, v in combo]
            names = [n for n, _ in combo]

            # Try all permutations
            for perm in itertools.permutations(values):
                # Try all operation trees
                for ops in itertools.product(range(len(operations)), repeat=n_terms-1):
                    try:
                        # Left-associative evaluation
                        val = perm[0]
                        for i in range(1, n_terms):
                            val = operations[ops[i-1]](val, perm[i])

                        # Check against targets
                        for target_name, target_val in targets.items():
                            if target_val == 0:
                                continue
                            error = abs(val - target_val) / target_val * 100
                            if error < threshold:
                                results.append({
                                    "n_terms": n_terms,
                                    "formulas": names,
                                    "operations": [op_names[o] for o in ops],
                                    "value": val,
                                    "target": target_name,
                                    "error": error
                                })
                    except (OverflowError, ZeroDivisionError):
                        continue

    elapsed = time.time() - start
    print(f"  Combination search complete: {elapsed:.2f}s, {len(results)} results")
    return results

def search_n_pow_phi_pi_e(targets, threshold, max_pow=10):
    """Search n·φ^a·π^b·e^c with expanded coefficient range"""
    results = []

    print(f"  Searching n·φ^a·π^b^e^c up to max_pow={max_pow}...")

    # Generate exponent grid
    phi_pows = np.arange(-max_pow, max_pow + 1)
    pi_pows = np.arange(-max_pow, max_pow + 1)
    e_pows = np.arange(-max_pow, max_pow + 1)

    phi_exp_grid, pi_exp_grid, e_exp_grid = np.meshgrid(
        phi_pows, pi_pows, e_pows, indexing='ij'
    )

    phi_exp_flat = phi_exp_grid.flatten()
    pi_exp_flat = pi_exp_grid.flatten()
    e_exp_flat = e_exp_grid.flatten()

    total_exps = len(phi_exp_flat)
    print(f"    Exponent combinations: {total_exps}")

    # Pre-compute base values
    phi_vals = PHI ** phi_exp_flat
    pi_vals = PI ** pi_exp_flat
    e_vals = E ** e_exp_flat
    base_vals = phi_vals * pi_vals * e_vals

    # Search for each coefficient
    max_coeff = 10000
    for coeff in range(1, max_coeff + 1):
        vals = coeff * base_vals

        for target_name, target_val in targets.items():
            errors = np.abs(vals - target_val) / target_val * 100
            match_indices = np.where(errors < threshold)[0]

            for idx in match_indices:
                results.append({
                    "coeff": coeff,
                    "phi_exp": int(phi_exp_flat[idx]),
                    "pi_exp": int(pi_exp_flat[idx]),
                    "e_exp": int(e_exp_flat[idx]),
                    "value": float(vals[idx]),
                    "target": target_name,
                    "error": float(errors[idx])
                })

        if coeff % 1000 == 0:
            print(f"    Progress: coeff={coeff}/{max_coeff}, found={len(results)}")

    return results

def main():
    print("="*70)
    print("  ULTRA ENGINE v6.7 — MATRIX SEARCH (ALL MATRICES)")
    print("="*70)
    print("  Searching ALL matrix combinations:")
    print("  - 2x2 matrices (determinant, trace, Frobenius)")
    print("  - n-ary formula combinations")
    print("  - Expanded n·φ^a·π^b·e^c search")
    print()

    threshold = 0.05
    all_results = []

    # Method 1: 2x2 matrices
    results_2x2 = search_2x2_matrices(BASE_FORMULAS, TARGETS, threshold)
    all_results.extend(results_2x2)

    # Method 2: Formula combinations
    results_comb = search_formula_combinations(BASE_FORMULAS, TARGETS, threshold, max_terms=3)
    all_results.extend(results_comb)

    # Method 3: Expanded phi/pi/e search
    results_pow = search_n_pow_phi_pi_e(TARGETS, threshold, max_pow=10)
    all_results.extend(results_pow)

    # Print summary
    print("\n" + "="*70)
    print("  MATRIX SEARCH COMPLETE")
    print("="*70)
    print(f"  Total results: {len(all_results)}")

    # Sort by error
    all_results.sort(key=lambda x: x["error"])

    # Print top results
    print("\n  TOP DISCOVERIES:")
    for r in all_results[:20]:
        if "coeff" in r:
            print(f"    {r['coeff']}*phi^{r['phi_exp']}*pi^{r['pi_exp']}*e^{r['e_exp']} = {r['value']:.6f} | Δ={r['error']:.6f}% | {r['target']}")
        elif "matrix_type" in r:
            print(f"    {r['matrix_type']} {r['invariant']}({r['elements']}) = {r['value']:.6f} | Δ={r['error']:.6f}% | {r['target']}")
        else:
            print(f"    {r['formulas']} {r['operations']} = {r['value']:.6f} | Δ={r['error']:.6f}% | {r['target']}")

    # Save results
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_file = f"/tmp/discovery_matrix_{timestamp}.txt"

    with open(output_file, "w") as f:
        f.write("# ULTRA ENGINE v6.7 — MATRIX SEARCH RESULTS\n")
        f.write(f"# Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n")
        f.write(f"# Total results: {len(all_results)}\n\n")

        f.write("## TOP 100 RESULTS\n\n")
        for r in all_results[:100]:
            if "coeff" in r:
                f.write(f"{r['coeff']}*phi^{r['phi_exp']}*pi^{r['pi_exp']}*e^{r['e_exp']} = {r['value']:.10f} | Δ={r['error']:.10f}% | {r['target']}\n")
            elif "matrix_type" in r:
                f.write(f"{r['matrix_type']} {r['invariant']}({r['elements']}) = {r['value']:.10f} | Δ={r['error']:.10f}% | {r['target']}\n")
            else:
                f.write(f"{r['formulas']} {r['operations']} = {r['value']:.10f} | Δ={r['error']:.10f}% | {r['target']}\n")

    print(f"\nResults saved to: {output_file}")

if __name__ == "__main__":
    main()
