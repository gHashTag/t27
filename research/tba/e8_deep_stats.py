#!/usr/bin/env python3
"""
E₈ Deep Statistical Analysis + Enhanced 14-Target Optimization
================================================================

1. Run 1M random samples for rigorous p-value
2. Use dual_annealing for 14-target (global optimizer, better than basin-hopping)
3. Characterize the solution space: is the optimum unique or degenerate?
"""

import numpy as np
from scipy.optimize import minimize, dual_annealing
import math
import json
import time
from multiprocessing import Pool

PHI = (1 + math.sqrt(5)) / 2
PI = math.pi

def zamolodchikov_masses():
    return np.array([
        1.0,
        2 * math.cos(PI/5),
        2 * math.cos(PI/30),
        4 * math.cos(PI/5) * math.cos(7*PI/30),
        4 * math.cos(PI/5) * math.cos(2*PI/15),
        4 * math.cos(PI/5) * math.cos(PI/30),
        8 * math.cos(PI/5)**2 * math.cos(7*PI/30),
        8 * math.cos(PI/5)**2 * math.cos(2*PI/15),
    ])

E8_INCIDENCE = np.array([
    [0, 1, 0, 0, 0, 0, 0, 0],
    [1, 0, 1, 0, 0, 0, 0, 0],
    [0, 1, 0, 1, 0, 0, 0, 0],
    [0, 0, 1, 0, 1, 0, 0, 0],
    [0, 0, 0, 1, 0, 1, 0, 1],
    [0, 0, 0, 0, 1, 0, 1, 0],
    [0, 0, 0, 0, 0, 1, 0, 0],
    [0, 0, 0, 0, 1, 0, 0, 0],
], dtype=float)

eigenvalues, eigenvectors = np.linalg.eigh(E8_INCIDENCE)

def deformed_masses(mu):
    m0 = zamolodchikov_masses()
    log_shift = eigenvectors @ mu
    M = m0 * np.exp(log_shift)
    return M

def compute_ratios(M):
    ratios = {}
    for i in range(7):
        ratios[f"M{i+2}/M{i+1}"] = M[i+1] / M[i]
    for i in range(8):
        for j in range(i+1, 8):
            ratios[f"M{j+1}/M{i+1}"] = M[j] / M[i]
    ratios["M8*M1/M4^2"] = M[7] * M[0] / (M[3]**2)
    ratios["M5/M2"] = M[4] / M[1]
    ratios["(M3/M1)^2"] = (M[2] / M[0])**2
    ratios["M6*M7/(M8*M3)"] = M[5] * M[6] / (M[7] * M[2])
    ratios["M2*M3/M5"] = M[1] * M[2] / M[4]
    ratios["M1*M8/M6"] = M[0] * M[7] / M[5]
    ratios["(M2/M1)^3"] = (M[1] / M[0])**3
    ratios["(M3/M2)^2"] = (M[2] / M[1])**2
    ratios["M7/M2^2"] = M[6] / (M[1]**2)
    # Additional compound ratios
    ratios["M3*M5/M8"] = M[2] * M[4] / M[7]
    ratios["M4*M6/M7^2"] = M[3] * M[5] / (M[6]**2) if M[6] > 0 else 1e10
    ratios["(M4/M1)^2"] = (M[3] / M[0])**2
    ratios["M2^3/M1"] = M[1]**3 / M[0]
    ratios["M1*M5/M3"] = M[0] * M[4] / M[2]
    return ratios

SM_TARGETS = {
    "phi": {"value": PHI},
    "phi2": {"value": PHI**2},
    "phi3": {"value": PHI**3},
    "mu_e": {"value": 206.768},
    "tau_mu": {"value": 16.817},
    "mp_me": {"value": 1836.15},
    "alpha_inv": {"value": 137.036},
    "sin2tw": {"value": 0.23121},
    "MZ_MW": {"value": 1.1342},
    "koide": {"value": 2.0/3.0},
    "MH_MW": {"value": 125.25/80.377},
    "Mt_MW": {"value": 172.69/80.377},
    "mp_mpi": {"value": 938.272/139.570},
    "rho_ratio": {"value": 0.6847/0.3153},
}

def best_match_for_target(ratios, target_val):
    best_err = float('inf')
    best_val = None
    best_name = None
    for name, val in ratios.items():
        if val > 0:
            err = abs(val - target_val) / target_val * 100
            if err < best_err:
                best_err = err
                best_val = val
                best_name = name
    return best_name, best_val, best_err

def count_matches(mu, target_names, threshold=1.0):
    """Count how many targets match at <threshold%"""
    M = deformed_masses(mu)
    if np.any(M <= 0):
        return 0
    ratios = compute_ratios(M)
    n = 0
    for tname in target_names:
        tv = SM_TARGETS[tname]["value"]
        _, _, err = best_match_for_target(ratios, tv)
        if err < threshold:
            n += 1
    return n

def cost_function_14(mu):
    """Cost for all 14 targets"""
    M = deformed_masses(mu)
    if np.any(M <= 0):
        return 1e10
    ratios = compute_ratios(M)
    cost = 0.0
    for tname in SM_TARGETS:
        tv = SM_TARGETS[tname]["value"]
        _, _, err = best_match_for_target(ratios, tv)
        cost += err**2
    return cost

# ═══════════════════════════════════════════════════════════════
# 1M RANDOM BASELINE
# ═══════════════════════════════════════════════════════════════

def batch_random_test(args):
    """Test a batch of random μ samples"""
    batch_size, seed, target_names = args
    rng = np.random.RandomState(seed)
    
    distribution_1pct = np.zeros(len(target_names) + 1, dtype=int)
    distribution_5pct = np.zeros(len(target_names) + 1, dtype=int)
    
    for _ in range(batch_size):
        mu = rng.randn(8) * 3.0
        M = deformed_masses(mu)
        if np.any(M <= 0):
            distribution_1pct[0] += 1
            distribution_5pct[0] += 1
            continue
        
        ratios = compute_ratios(M)
        n1, n5 = 0, 0
        for tname in target_names:
            tv = SM_TARGETS[tname]["value"]
            _, _, err = best_match_for_target(ratios, tv)
            if err < 1.0: n1 += 1
            if err < 5.0: n5 += 1
        
        distribution_1pct[n1] += 1
        distribution_5pct[n5] += 1
    
    return distribution_1pct, distribution_5pct


if __name__ == "__main__":
    np.random.seed(42)
    
    target_names_10 = ["phi", "phi2", "phi3", "mu_e", "tau_mu", 
                        "mp_me", "alpha_inv", "sin2tw", "MZ_MW", "koide"]
    target_names_14 = list(SM_TARGETS.keys())
    
    # ─── Part 1: 1M Random baseline for 10 targets ───
    print("=" * 80)
    print("PART 1: 1M RANDOM BASELINE (10 targets)")
    print("=" * 80)
    
    t0 = time.time()
    N_TOTAL = 1_000_000
    N_BATCHES = 10
    BATCH_SIZE = N_TOTAL // N_BATCHES
    
    args_list = [(BATCH_SIZE, seed, target_names_10) for seed in range(N_BATCHES)]
    
    dist_1pct_total = np.zeros(11, dtype=int)
    dist_5pct_total = np.zeros(11, dtype=int)
    
    for args in args_list:
        d1, d5 = batch_random_test(args)
        dist_1pct_total += d1
        dist_5pct_total += d5
    
    t1 = time.time()
    
    print(f"\n  {N_TOTAL:,} random samples, {t1-t0:.1f}s")
    print(f"\n  Distribution of <1% matches (10 targets):")
    for k in range(11):
        if dist_1pct_total[k] > 0:
            pct = dist_1pct_total[k] / N_TOTAL * 100
            print(f"    {k:2d} matches: {dist_1pct_total[k]:>8,} ({pct:.4f}%)")
    
    print(f"\n  Distribution of <5% matches (10 targets):")
    for k in range(11):
        if dist_5pct_total[k] > 0:
            pct = dist_5pct_total[k] / N_TOTAL * 100
            print(f"    {k:2d} matches: {dist_5pct_total[k]:>8,} ({pct:.4f}%)")
    
    # P-value: P(≥10 at <1%) and P(≥9 at <1%)
    cum_1pct = np.cumsum(dist_1pct_total[::-1])[::-1]
    print(f"\n  Cumulative P-values (≥k matches at <1%):")
    for k in range(max(0, 10-5), 11):
        p = cum_1pct[k] / N_TOTAL
        print(f"    P(≥{k:2d}) = {p:.8f}" + (" ← WE ACHIEVED THIS" if k == 10 else ""))
    
    # ─── Part 2: 1M Random baseline for 14 targets ───
    print(f"\n{'='*80}")
    print("PART 2: 1M RANDOM BASELINE (14 targets)")
    print(f"{'='*80}")
    
    t0 = time.time()
    args_list = [(BATCH_SIZE, seed + 100, target_names_14) for seed in range(N_BATCHES)]
    
    dist_1pct_14 = np.zeros(15, dtype=int)
    dist_5pct_14 = np.zeros(15, dtype=int)
    
    for args in args_list:
        d1, d5 = batch_random_test(args)
        dist_1pct_14[:len(d1)] += d1
        dist_5pct_14[:len(d5)] += d5
    
    t1 = time.time()
    
    print(f"\n  {N_TOTAL:,} random samples, {t1-t0:.1f}s")
    print(f"\n  Distribution of <1% matches (14 targets):")
    for k in range(15):
        if dist_1pct_14[k] > 0:
            pct = dist_1pct_14[k] / N_TOTAL * 100
            print(f"    {k:2d} matches: {dist_1pct_14[k]:>8,} ({pct:.4f}%)")
    
    cum_1pct_14 = np.cumsum(dist_1pct_14[::-1])[::-1]
    print(f"\n  Cumulative P-values (≥k at <1%, 14 targets):")
    for k in range(max(0, 9-5), 15):
        if cum_1pct_14[k] > 0 or k <= 9:
            p = cum_1pct_14[k] / N_TOTAL
            print(f"    P(≥{k:2d}) = {p:.8f}" + (" ← WE ACHIEVED THIS" if k == 9 else ""))
    
    # ─── Part 3: Enhanced 14-target with dual_annealing ───
    print(f"\n{'='*80}")
    print("PART 3: ENHANCED 14-TARGET OPTIMIZER (dual_annealing)")
    print(f"{'='*80}")
    
    bounds = [(-5, 5)] * 8
    
    best_n1_14 = 0
    best_mu_14 = None
    
    for trial in range(5):
        print(f"\n  Trial {trial+1}/5...")
        t0 = time.time()
        
        result = dual_annealing(
            cost_function_14,
            bounds=bounds,
            maxiter=500,
            seed=trial * 13 + 7
        )
        
        M = deformed_masses(result.x)
        ratios = compute_ratios(M)
        
        n1 = 0
        n5 = 0
        for tname in SM_TARGETS:
            tv = SM_TARGETS[tname]["value"]
            _, _, err = best_match_for_target(ratios, tv)
            if err < 1.0: n1 += 1
            if err < 5.0: n5 += 1
        
        t1 = time.time()
        print(f"    {n1}/14 at <1%, {n5}/14 at <5% (cost={result.fun:.2f}, {t1-t0:.1f}s)")
        
        if n1 > best_n1_14:
            best_n1_14 = n1
            best_mu_14 = result.x.copy()
            best_cost_14 = result.fun
    
    # Final evaluation of best 14-target result
    print(f"\n  BEST 14-target result: {best_n1_14}/14 at <1%")
    if best_mu_14 is not None:
        M = deformed_masses(best_mu_14)
        ratios = compute_ratios(M)
        print(f"  μ = {best_mu_14.tolist()}")
        for tname in sorted(SM_TARGETS.keys()):
            tv = SM_TARGETS[tname]["value"]
            rname, rval, err = best_match_for_target(ratios, tv)
            mark = "✅" if err < 1.0 else ("⚠️" if err < 5.0 else "❌")
            print(f"    {mark} {tname:12s}: {rval:.6f} vs {tv:.6f} ({err:.3f}%) via {rname}")
    
    # ─── Part 4: Solution space characterization ───
    print(f"\n{'='*80}")
    print("PART 4: SOLUTION SPACE (10-target)")
    print(f"{'='*80}")
    
    # Find multiple 10/10 solutions
    print("  Finding multiple 10/10 solutions to check uniqueness...")
    
    solutions_10of10 = []
    mu_prev_best = np.array([-1.1666368804744063, 3.3524114853769658, 1.402816669857752, 
                              -0.5687732434597406, -2.63284038506671, 3.8621987667747324, 
                              -0.2424390306460934, 4.231225551664229])
    solutions_10of10.append(mu_prev_best)
    
    for trial in range(30):
        mu0 = np.random.randn(8) * 2.0
        
        def cost_10(mu):
            M = deformed_masses(mu)
            if np.any(M <= 0): return 1e10
            ratios = compute_ratios(M)
            cost = 0.0
            for tname in target_names_10:
                tv = SM_TARGETS[tname]["value"]
                _, _, err = best_match_for_target(ratios, tv)
                cost += err**2
            return cost
        
        result = minimize(cost_10, mu0, method='Nelder-Mead',
                          options={'maxiter': 10000, 'xatol': 1e-10, 'fatol': 1e-10})
        
        n1 = count_matches(result.x, target_names_10, threshold=1.0)
        if n1 == 10:
            solutions_10of10.append(result.x.copy())
    
    print(f"  Found {len(solutions_10of10)} solutions with 10/10 at <1%")
    
    if len(solutions_10of10) > 1:
        # Check if solutions are really different
        unique_solutions = [solutions_10of10[0]]
        for sol in solutions_10of10[1:]:
            is_new = True
            for usol in unique_solutions:
                dist = np.linalg.norm(sol - usol)
                if dist < 0.5:
                    is_new = False
                    break
            if is_new:
                unique_solutions.append(sol)
        
        print(f"  Unique solutions (distance > 0.5): {len(unique_solutions)}")
        for i, sol in enumerate(unique_solutions[:5]):
            print(f"    Solution {i+1}: μ = [{', '.join(f'{x:.3f}' for x in sol)}]")
            print(f"      norm = {np.linalg.norm(sol):.3f}")
    
    # ─── Save comprehensive results ───
    output = {
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%S"),
        "random_baseline_1M": {
            "n_samples": N_TOTAL,
            "dist_1pct_10target": dist_1pct_total.tolist(),
            "dist_5pct_10target": dist_5pct_total.tolist(),
            "dist_1pct_14target": dist_1pct_14.tolist(),
            "p_value_10of10": float(cum_1pct[10] / N_TOTAL) if 10 < len(cum_1pct) else 0.0,
            "p_value_9of14": float(cum_1pct_14[9] / N_TOTAL) if 9 < len(cum_1pct_14) else 0.0,
        },
        "best_14target": {
            "n_1pct": best_n1_14,
            "mu": best_mu_14.tolist() if best_mu_14 is not None else None,
        },
        "solution_space": {
            "n_10of10_solutions_found": len(solutions_10of10),
            "n_unique": len(unique_solutions) if len(solutions_10of10) > 1 else 1,
        }
    }
    
    with open('research/tba/e8_deep_stats.json', 'w') as f:
        json.dump(output, f, indent=2)
    
    print(f"\nResults saved to research/tba/e8_deep_stats.json")
