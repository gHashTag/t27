#!/usr/bin/env python3
"""
ULTRA ENGINE v6.8 — NEW STRUCTURES FRONTIER
Beyond n·φ^a·π^b·e^c: explores sin/cos/ln/exp/sqrt/roots/n-ary trees
"""

import numpy as np
import time
from datetime import datetime
from itertools import product, permutations

PHI = 1.6180339887498948
PI = np.pi
E = np.e

# ALL PDG 2024 TARGETS
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
    "gamma_e": 0.00115965918128,
    "V_us": 0.22431,
    "V_ud": 0.97435,
    "V_cb": 0.0408,
    "V_ub": 0.0037,
    "V_td": 0.00814,
    "V_cs": 0.97548,
    "theta12": np.deg2rad(33.44),
    "theta13": np.deg2rad(8.61),
    "theta23": np.deg2rad(49.3),
    "G_F": 1.1663787e-5,
    "R_inf": 0.5,
    "Omega_Lambda": 0.6889,
    "Delta_cp": np.deg2rad(68.0),
}

# NEW STRUCTURE SEARCH PARAMETERS
COEFF_MIN, COEFF_MAX = 1, 10000
EXP_MIN, EXP_MAX = -15, 15
THRESHOLD = 0.1

def sin_structure_search(base_coeffs, base_exps, targets, threshold):
    """Search sin(n·φ^a·π^b·e^c) structures"""
    results = []

    print("  Searching sin(n·X) structures...")
    start = time.time()

    for n in base_coeffs:
        phi_vals = PHI ** base_exps[0]
        pi_vals = PI ** base_exps[1]
        e_vals = E ** base_exps[2]

        base_values = (phi_vals * pi_vals * e_vals).flatten()

        # sin(n·val) for all combinations
        for val in base_values:
            sin_val = np.sin(n * val)

            for target_name, target_val in targets.items():
                if target_val == 0:
                    continue
                error = abs(sin_val - target_val) / target_val * 100
                if error < threshold:
                    results.append({
                        "structure": "sin",
                        "params": f"sin({n}·{val:.4f})",
                        "value": float(sin_val),
                        "target": target_name,
                        "error": float(error)
                    })

    elapsed = time.time() - start
    print(f"  sin search: {elapsed:.2f}s, {len(results)} results")
    return results

def cos_structure_search(base_coeffs, base_exps, targets, threshold):
    """Search cos(n·X) structures"""
    results = []

    print("  Searching cos(n·X) structures...")
    start = time.time()

    for n in base_coeffs:
        phi_vals = PHI ** base_exps[0]
        pi_vals = PI ** base_exps[1]
        e_vals = E ** base_exps[2]

        base_values = (phi_vals * pi_vals * e_vals).flatten()

        # cos(n·val) for all combinations
        for val in base_values:
            cos_val = np.cos(n * val)

            for target_name, target_val in targets.items():
                if target_val == 0:
                    continue
                error = abs(cos_val - target_val) / target_val * 100
                if error < threshold:
                    results.append({
                        "structure": "cos",
                        "params": f"cos({n}·{val:.4f})",
                        "value": float(cos_val),
                        "target": target_name,
                        "error": float(error)
                    })

    elapsed = time.time() - start
    print(f"  cos search: {elapsed:.2f}s, {len(results)} results")
    return results

def ln_structure_search(base_exps, targets, threshold):
    """Search ln(φ^a·π^b·e^c) structures"""
    results = []

    print("  Searching ln(X) structures...")
    start = time.time()

    phi_vals = PHI ** base_exps[0]
    pi_vals = PI ** base_exps[1]
    e_vals = E ** base_exps[2]

    base_values = (phi_vals * pi_vals * e_vals).flatten()

    # ln(val) for all combinations
    for val in base_values:
        if val <= 0:
            continue
        ln_val = np.log(val)

        for target_name, target_val in targets.items():
            if target_val == 0:
                continue
            error = abs(ln_val - target_val) / target_val * 100
            if error < threshold:
                results.append({
                    "structure": "ln",
                    "params": f"ln({val:.4f})",
                    "value": float(ln_val),
                    "target": target_name,
                    "error": float(error)
                })

    elapsed = time.time() - start
    print(f"  ln search: {elapsed:.2f}s, {len(results)} results")
    return results

def exp_structure_search(base_coeffs, base_exps, targets, threshold):
    """Search exp(n·φ^a·π^b·e^c) structures"""
    results = []

    print("  Searching exp(n·X) structures...")
    start = time.time()

    for n in base_coeffs:
        phi_vals = PHI ** base_exps[0]
        pi_vals = PI ** base_exps[1]
        e_vals = E ** base_exps[2]

        base_values = (phi_vals * pi_vals * e_vals).flatten()

        # exp(n·val) for all combinations
        for val in base_values:
            exp_val = np.exp(n * val)

            for target_name, target_val in targets.items():
                if target_val == 0:
                    continue
                error = abs(exp_val - target_val) / target_val * 100
                if error < threshold:
                    results.append({
                        "structure": "exp",
                        "params": f"exp({n}·{val:.4f})",
                        "value": float(exp_val),
                        "target": target_name,
                        "error": float(error)
                    })

    elapsed = time.time() - start
    print(f"  exp search: {elapsed:.2f}s, {len(results)} results")
    return results

def sqrt_structure_search(base_coeffs, base_exps, targets, threshold):
    """Search sqrt(n·φ^a·π^b·e^c) structures"""
    results = []

    print("  Searching sqrt(n·X) structures...")
    start = time.time()

    for n in base_coeffs:
        phi_vals = PHI ** base_exps[0]
        pi_vals = PI ** base_exps[1]
        e_vals = E ** base_exps[2]

        base_values = (phi_vals * pi_vals * e_vals).flatten()

        # sqrt(n·val) for all combinations
        for val in base_values:
            if val < 0:
                continue
            sqrt_val = np.sqrt(n * val)

            for target_name, target_val in targets.items():
                if target_val == 0:
                    continue
                error = abs(sqrt_val - target_val) / target_val * 100
                if error < threshold:
                    results.append({
                        "structure": "sqrt",
                        "params": f"sqrt({n}·{val:.4f})",
                        "value": float(sqrt_val),
                        "target": target_name,
                        "error": float(error)
                    })

    elapsed = time.time() - start
    print(f"  sqrt search: {elapsed:.2f}s, {len(results)} results")
    return results

def n_root_structure_search(base_coeffs, base_exps, targets, threshold):
    """Search n-root(n·φ^a·π^b·e^c) structures"""
    results = []

    print("  Searching n-root structures...")
    start = time.time()

    phi_vals = PHI ** base_exps[0]
    pi_vals = PI ** base_exps[1]
    e_vals = E ** base_exps[2]

    base_values = (phi_vals * pi_vals * e_vals).flatten()

    for n in base_coeffs:
        for val in base_values:
            if val <= 0 and n % 2 == 0:
                continue
            root_val = val ** (1.0 / n)

            for target_name, target_val in targets.items():
                if target_val == 0:
                    continue
                error = abs(root_val - target_val) / target_val * 100
                if error < threshold:
                    results.append({
                        "structure": f"{n}-root",
                        "params": f"{n}-root({val:.4f})",
                        "value": float(root_val),
                        "target": target_name,
                        "error": float(error)
                    })

    elapsed = time.time() - start
    print(f"  n-root search: {elapsed:.2f}s, {len(results)} results")
    return results

def mixed_tree_search(base_coeffs, base_exps, targets, threshold, max_depth=3):
    """Search arbitrary operator trees (a+b)*(c+d)+e-f etc.)"""
    results = []

    print(f"  Searching mixed trees (depth={max_depth})...")
    start = time.time()

    phi_vals = PHI ** base_exps[0]
    pi_vals = PI ** base_exps[1]
    e_vals = E ** base_exps[2]

    base_values = (phi_vals * pi_vals * e_vals).flatten()

    # Generate all trees up to max_depth
    from functools import reduce

    def evaluate_tree(tree, idx=0):
        """Evaluate tree at given index"""
        if isinstance(tree, tuple):
            a, op, b = tree
            a_val = evaluate_tree(a)
            b_val = evaluate_tree(b)
            if op == '+':
                return a_val + b_val
            elif op == '-':
                return a_val - b_val
            elif op == '*':
                return a_val * b_val
            elif op == '/':
                if b_val != 0:
                    return a_val / b_val
                return 0
            else:
                return a_val
        elif isinstance(tree, str):
            return tree
        else:
            return float(tree)

    # Generate simple trees
    all_values = []
    for depth in range(2, max_depth + 1):
        print(f"    Depth {depth} trees...")
        ops = ['+', '-', '*', '/']
        values = base_values[:1000]  # Limit for performance

        for tree_size in range(depth):
            for combo in product(values, ops, repeat=tree_size):
                try:
                    # Build tree: a + b
                    current = combo[0]
                    for i in range(tree_size):
                        current = (current, ops[i], combo[i+1])

                    val = evaluate_tree(current)

                    for target_name, target_val in targets.items():
                        if target_val == 0:
                            continue
                        error = abs(val - target_val) / target_val * 100
                        if error < threshold:
                            # Build expression string
                            expr_parts = []
                            temp = current
                            while isinstance(temp, tuple):
                                expr_parts.insert(0, f"({temp[1]} {temp[0]} {temp[2]})")
                                temp = temp[2]
                            expr = ''.join(expr_parts)

                            results.append({
                                "structure": f"tree-depth{depth}",
                                "params": expr,
                                "value": float(val),
                                "target": target_name,
                                "error": float(error)
                            })
                except (ZeroDivisionError, OverflowError, ValueError):
                    continue

    elapsed = time.time() - start
    print(f"  mixed tree search: {elapsed:.2f}s, {len(results)} results")
    return results

def main():
    print("="*70)
    print("  ULTRA ENGINE v6.8 — NEW STRUCTURES FRONTIER")
    print("="*70)
    print("  Exploring BEYOND n·φ^a·π^b·e^c template")
    print("  Structures: sin, cos, ln, exp, sqrt, n-root, mixed trees")
    print()

    # Generate base values
    coeff_range = np.arange(COEFF_MIN, COEFF_MAX + 1)
    phi_pows = np.arange(EXP_MIN, EXP_MAX + 1)
    pi_pows = np.arange(EXP_MIN, EXP_MAX + 1)
    e_pows = np.arange(EXP_MIN, EXP_MAX + 1)

    phi_exp_grid, pi_exp_grid, e_exp_grid = np.meshgrid(
        phi_pows, pi_pows, e_pows, indexing='ij'
    )

    phi_exps = phi_exp_grid.flatten()
    pi_exps = pi_exp_grid.flatten()
    e_exps = e_exp_grid.flatten()

    all_results = []

    # Method 1: sin structures
    results_sin = sin_structure_search(coeff_range, (phi_exps, pi_exps, e_exps), TARGETS, THRESHOLD)
    all_results.extend(results_sin)

    # Method 2: cos structures
    results_cos = cos_structure_search(coeff_range, (phi_exps, pi_exps, e_exps), TARGETS, THRESHOLD)
    all_results.extend(results_cos)

    # Method 3: ln structures
    results_ln = ln_structure_search((phi_exps, pi_exps, e_exps), TARGETS, THRESHOLD)
    all_results.extend(results_ln)

    # Method 4: exp structures
    results_exp = exp_structure_search(coeff_range, (phi_exps, pi_exps, e_exps), TARGETS, THRESHOLD)
    all_results.extend(results_exp)

    # Method 5: sqrt structures
    results_sqrt = sqrt_structure_search(coeff_range, (phi_exps, pi_exps, e_exps), TARGETS, THRESHOLD)
    all_results.extend(results_sqrt)

    # Method 6: n-root structures
    results_root = n_root_structure_search(coeff_range, (phi_exps, pi_exps, e_exps), TARGETS, THRESHOLD)
    all_results.extend(results_root)

    # Method 7: mixed operator trees
    results_trees = mixed_tree_search(coeff_range, (phi_exps, pi_exps, e_exps), TARGETS, THRESHOLD, max_depth=2)
    all_results.extend(results_trees)

    # Print summary
    print("\n" + "="*70)
    print("  NEW STRUCTURES SEARCH COMPLETE")
    print("="*70)

    all_results.sort(key=lambda x: x["error"])

    # Count by structure type
    by_structure = {}
    for r in all_results:
        struct = r["structure"]
        by_structure[struct] = by_structure.get(struct, 0) + 1

    print("\n  RESULTS BY STRUCTURE:")
    for struct, count in sorted(by_structure.items(), key=lambda x: -x[1]):
        print(f"    {struct}: {count} formulas")

    print(f"\n  TOTAL NEW STRUCTURES: {len(all_results)}")

    # Print W/Z top discoveries
    wz_results = [r for r in all_results if r["target"] in ["W_mass", "Z_mass"]]
    wz_sorted = sorted(wz_results, key=lambda x: x["error"])[:20]

    print("\n  TOP W/Z NEW STRUCTURE CANDIDATES:")
    for r in wz_sorted:
        print(f"    {r['params']} = {r['value']:.8f} | Δ={r['error']:.6f}% | {r['target']}")

    # Save results
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_file = f"/tmp/discovery_new_structures_{timestamp}.txt"

    with open(output_file, "w") as f:
        f.write("# ULTRA ENGINE v6.8 — NEW STRUCTURES DISCOVERY\n")
        f.write(f"# Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n")
        f.write("# Explored: sin, cos, ln, exp, sqrt, n-root, mixed trees\n")
        f.write(f"# Total new structures: {len(all_results)}\n\n")

        # W/Z candidates
        f.write("## TOP W/Z NEW STRUCTURE CANDIDATES\n\n")
        for r in wz_sorted[:50]:
            f.write(f"{r['params']} = {r['value']:.10f} | Δ={r['error']:.10f}% | {r['target']}\n")

        # Summary by structure
        f.write("\n## SUMMARY BY STRUCTURE\n\n")
        for struct, count in sorted(by_structure.items(), key=lambda x: -x[1]):
            f.write(f"{struct}: {count} formulas\n")

    print(f"\nResults saved to: {output_file}")

if __name__ == "__main__":
    main()
