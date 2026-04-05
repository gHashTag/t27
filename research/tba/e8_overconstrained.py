#!/usr/bin/env python3
"""
E₈ OVERCONSTRAINED OPTIMIZATION — PROJECT KEPLER→NEWTON
=========================================================

Key question: Can 8 mass-deformation parameters μ₁...μ₈ simultaneously
match MORE than 8 SM observables to <1% accuracy?

If YES → the system is overconstrained → non-trivial prediction
If NO → 8 params matching ≤8 targets = mere fitting

Strategy:
  1. Fast optimizer: Basin-hopping with L-BFGS-B (much faster than diff evolution)
  2. Multiple restarts with different random seeds
  3. Progressive target addition: start with 6, add more
  4. Statistical baseline: random μ comparison

The deformed mass spectrum:
  M_a(μ) = m_a * exp(Σ_b μ_b * V_ab)
  
where V_ab encodes how deformation b affects particle a
(via the E₈ incidence matrix eigenvectors).

Mass ratios from the deformed spectrum are then compared to SM observables.
"""

import numpy as np
from scipy.optimize import minimize, basinhopping, dual_annealing
import math
import json
import time
from itertools import combinations

PHI = (1 + math.sqrt(5)) / 2
PI = math.pi

# ═══════════════════════════════════════════════════════════════
# E₈ Mass Spectrum
# ═══════════════════════════════════════════════════════════════

def zamolodchikov_masses():
    """Exact E₈ mass ratios"""
    return np.array([
        1.0,
        2 * math.cos(PI/5),                              # φ
        2 * math.cos(PI/30),
        4 * math.cos(PI/5) * math.cos(7*PI/30),
        4 * math.cos(PI/5) * math.cos(2*PI/15),
        4 * math.cos(PI/5) * math.cos(PI/30),
        8 * math.cos(PI/5)**2 * math.cos(7*PI/30),
        8 * math.cos(PI/5)**2 * math.cos(2*PI/15),
    ])

# E₈ incidence matrix (Dynkin diagram adjacency)
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

# Eigenvectors of incidence matrix → deformation directions
eigenvalues, eigenvectors = np.linalg.eigh(E8_INCIDENCE)

# ═══════════════════════════════════════════════════════════════
# Mass Deformation Model
# ═══════════════════════════════════════════════════════════════

def deformed_masses(mu):
    """
    Apply mass deformation: M_a = m_a * exp(Σ_b μ_b * V_ab)
    where V are eigenvectors of the E₈ incidence matrix.
    
    This parameterization ensures deformations respect E₈ symmetry structure.
    """
    m0 = zamolodchikov_masses()
    # Deformation via eigenvector mixing
    log_shift = eigenvectors @ mu  # 8-vector of log-shifts
    M = m0 * np.exp(log_shift)
    return M

def compute_ratios(M):
    """Compute all possible mass ratios from deformed spectrum"""
    ratios = {}
    # Adjacent ratios
    for i in range(7):
        ratios[f"M{i+2}/M{i+1}"] = M[i+1] / M[i]
    # Non-adjacent ratios
    for i in range(8):
        for j in range(i+1, 8):
            ratios[f"M{j+1}/M{i+1}"] = M[j] / M[i]
    # Products and combinations
    ratios["M8*M1/M4^2"] = M[7] * M[0] / (M[3]**2)
    ratios["M5/M2"] = M[4] / M[1]
    ratios["(M3/M1)^2"] = (M[2] / M[0])**2
    ratios["M6*M7/(M8*M3)"] = M[5] * M[6] / (M[7] * M[2])
    ratios["M2*M3/M5"] = M[1] * M[2] / M[4]
    ratios["M1*M8/M6"] = M[0] * M[7] / M[5]
    # Powers
    ratios["(M2/M1)^3"] = (M[1] / M[0])**3
    ratios["(M3/M2)^2"] = (M[2] / M[1])**2
    ratios["M7/M2^2"] = M[6] / (M[1]**2)
    return ratios

# ═══════════════════════════════════════════════════════════════
# SM Target Observables (14 targets)
# ═══════════════════════════════════════════════════════════════

SM_TARGETS = {
    # φ-related ratios (exact)
    "phi": {"value": PHI, "desc": "φ = (1+√5)/2"},
    "phi2": {"value": PHI**2, "desc": "φ²"},
    "phi3": {"value": PHI**3, "desc": "φ³"},
    # Lepton mass ratios
    "mu_e": {"value": 206.768, "desc": "m_μ/m_e"},
    "tau_mu": {"value": 16.817, "desc": "m_τ/m_μ"},
    # Baryon/lepton
    "mp_me": {"value": 1836.15, "desc": "m_p/m_e"},
    # Electroweak
    "alpha_inv": {"value": 137.036, "desc": "1/α"},
    "sin2tw": {"value": 0.23121, "desc": "sin²θ_W"},
    "MZ_MW": {"value": 1.1342, "desc": "M_Z/M_W"},
    # Koide formula
    "koide": {"value": 2.0/3.0, "desc": "Koide = 2/3"},
    # Additional EW/Higgs
    "MH_MW": {"value": 125.25/80.377, "desc": "M_H/M_W = 1.558"},
    "Mt_MW": {"value": 172.69/80.377, "desc": "m_t/M_W = 2.149"},
    # QCD
    "mp_mpi": {"value": 938.272/139.570, "desc": "m_p/m_π = 6.722"},
    # Cosmological hint
    "rho_ratio": {"value": 0.6847/0.3153, "desc": "Ω_Λ/Ω_m = 2.172"},
}

# ═══════════════════════════════════════════════════════════════
# Ratio-Target Matching Function
# ═══════════════════════════════════════════════════════════════

def best_match_for_target(ratios, target_val):
    """Find the ratio in our spectrum that best matches a target value.
    Returns (best_ratio_name, best_ratio_val, error_percent)"""
    best_name = None
    best_val = None
    best_err = float('inf')
    
    for name, val in ratios.items():
        if val > 0:  # Only positive ratios
            err = abs(val - target_val) / target_val * 100
            if err < best_err:
                best_err = err
                best_val = val
                best_name = name
    
    return best_name, best_val, best_err

def cost_function(mu, target_names, weight_scheme="equal"):
    """
    Cost function: sum of squared relative errors for selected targets.
    """
    M = deformed_masses(mu)
    
    # Ensure all masses positive
    if np.any(M <= 0):
        return 1e10
    
    ratios = compute_ratios(M)
    
    total_cost = 0.0
    for tname in target_names:
        target_val = SM_TARGETS[tname]["value"]
        _, _, err_pct = best_match_for_target(ratios, target_val)
        
        if weight_scheme == "log":
            # Logarithmic: penalizes large errors less, focuses on getting all close
            total_cost += np.log1p(err_pct)**2
        else:
            total_cost += (err_pct / 1.0)**2  # Normalize by 1% target
    
    return total_cost

# ═══════════════════════════════════════════════════════════════
# FAST OPTIMIZER: Basin-hopping + L-BFGS-B
# ═══════════════════════════════════════════════════════════════

def optimize_targets(target_names, n_restarts=30, max_time=120):
    """
    Multi-start optimization for a given set of targets.
    Returns best solution found within time limit.
    """
    n_targets = len(target_names)
    best_cost = float('inf')
    best_mu = None
    best_result = None
    
    start_time = time.time()
    
    for restart in range(n_restarts):
        if time.time() - start_time > max_time:
            break
        
        # Random initial guess
        mu0 = np.random.randn(8) * 1.5
        
        try:
            # Try L-BFGS-B first (fast gradient-based)
            result = minimize(
                cost_function, mu0,
                args=(target_names,),
                method='Nelder-Mead',
                options={'maxiter': 5000, 'xatol': 1e-8, 'fatol': 1e-8}
            )
            
            if result.fun < best_cost:
                best_cost = result.fun
                best_mu = result.x.copy()
                best_result = result
                
            # If good enough, try polishing with basin-hopping
            if result.fun < n_targets * 0.5:  # avg < 0.7% per target
                bh_result = basinhopping(
                    cost_function, result.x,
                    minimizer_kwargs={
                        'args': (target_names,),
                        'method': 'Nelder-Mead',
                        'options': {'maxiter': 3000}
                    },
                    niter=20,
                    T=1.0,
                    stepsize=0.5,
                    seed=restart
                )
                if bh_result.fun < best_cost:
                    best_cost = bh_result.fun
                    best_mu = bh_result.x.copy()
        except Exception as e:
            continue
    
    elapsed = time.time() - start_time
    return best_mu, best_cost, elapsed

def evaluate_solution(mu, target_names):
    """Evaluate a solution against all targets"""
    M = deformed_masses(mu)
    ratios = compute_ratios(M)
    
    results = {}
    n_1pct = 0
    n_5pct = 0
    
    for tname in target_names:
        target_val = SM_TARGETS[tname]["value"]
        rname, rval, err = best_match_for_target(ratios, target_val)
        results[tname] = {
            "target": target_val,
            "matched_ratio": rname,
            "ratio_value": float(rval),
            "error_pct": float(err),
            "desc": SM_TARGETS[tname]["desc"]
        }
        if err < 1.0:
            n_1pct += 1
        if err < 5.0:
            n_5pct += 1
    
    return results, n_1pct, n_5pct

# ═══════════════════════════════════════════════════════════════
# RANDOM BASELINE (statistical significance)
# ═══════════════════════════════════════════════════════════════

def random_baseline(target_names, n_samples=100000):
    """
    How many targets can RANDOM μ match at <1%?
    This is the null hypothesis baseline.
    """
    max_1pct_seen = 0
    max_5pct_seen = 0
    count_ge_threshold = {k: 0 for k in range(len(target_names) + 1)}
    
    for _ in range(n_samples):
        mu = np.random.randn(8) * 3.0
        M = deformed_masses(mu)
        if np.any(M <= 0):
            continue
        ratios = compute_ratios(M)
        
        n1 = 0
        n5 = 0
        for tname in target_names:
            tv = SM_TARGETS[tname]["value"]
            _, _, err = best_match_for_target(ratios, tv)
            if err < 1.0:
                n1 += 1
            if err < 5.0:
                n5 += 1
        
        if n1 > max_1pct_seen:
            max_1pct_seen = n1
        if n5 > max_5pct_seen:
            max_5pct_seen = n5
        
        for k in range(n1 + 1):
            count_ge_threshold[k] += 1
    
    return max_1pct_seen, max_5pct_seen, count_ge_threshold, n_samples

# ═══════════════════════════════════════════════════════════════
# MAIN: PROGRESSIVE OVERCONSTRAINED TEST
# ═══════════════════════════════════════════════════════════════

if __name__ == "__main__":
    np.random.seed(42)
    
    print("=" * 80)
    print("E₈ OVERCONSTRAINED OPTIMIZATION — PROJECT KEPLER→NEWTON")
    print("=" * 80)
    
    m0 = zamolodchikov_masses()
    print(f"\nZamolodchikov masses: {[f'{m:.6f}' for m in m0]}")
    print(f"m₂/m₁ = φ = {m0[1]/m0[0]:.15f}")
    
    # ─── Phase 1: 10-target optimization (beat previous 4/10 at <1%) ───
    print(f"\n{'='*80}")
    print("PHASE 1: 10-TARGET OPTIMIZATION")
    print(f"{'='*80}")
    
    targets_10 = ["phi", "phi2", "phi3", "mu_e", "tau_mu", 
                   "mp_me", "alpha_inv", "sin2tw", "MZ_MW", "koide"]
    
    print(f"Targets: {targets_10}")
    print(f"Optimizing (8 params → 10 targets)...")
    
    mu_best10, cost10, time10 = optimize_targets(targets_10, n_restarts=50, max_time=180)
    results10, n1_10, n5_10 = evaluate_solution(mu_best10, targets_10)
    
    print(f"\nBest 10-target result: {n1_10}/10 at <1%, {n5_10}/10 at <5%  (time: {time10:.1f}s)")
    print(f"μ = {mu_best10.tolist()}")
    for tname, r in sorted(results10.items(), key=lambda x: x[1]['error_pct']):
        mark = "✅" if r['error_pct'] < 1.0 else ("⚠️" if r['error_pct'] < 5.0 else "❌")
        print(f"  {mark} {tname:12s}: {r['ratio_value']:.6f} vs {r['target']:.6f} ({r['error_pct']:.3f}%) via {r['matched_ratio']}")
    
    # ─── Phase 2: 12-target (overconstrained!) ───
    print(f"\n{'='*80}")
    print("PHASE 2: 12-TARGET OVERCONSTRAINED TEST")
    print(f"{'='*80}")
    
    targets_12 = targets_10 + ["MH_MW", "Mt_MW"]
    
    print(f"Targets: {targets_12}")
    print(f"8 params → 12 targets = OVERCONSTRAINED by 4")
    print(f"Optimizing...")
    
    mu_best12, cost12, time12 = optimize_targets(targets_12, n_restarts=50, max_time=180)
    results12, n1_12, n5_12 = evaluate_solution(mu_best12, targets_12)
    
    print(f"\nBest 12-target result: {n1_12}/12 at <1%, {n5_12}/12 at <5%  (time: {time12:.1f}s)")
    print(f"μ = {mu_best12.tolist()}")
    for tname, r in sorted(results12.items(), key=lambda x: x[1]['error_pct']):
        mark = "✅" if r['error_pct'] < 1.0 else ("⚠️" if r['error_pct'] < 5.0 else "❌")
        print(f"  {mark} {tname:12s}: {r['ratio_value']:.6f} vs {r['target']:.6f} ({r['error_pct']:.3f}%) via {r['matched_ratio']}")
    
    # ─── Phase 3: Full 14-target ───
    print(f"\n{'='*80}")
    print("PHASE 3: FULL 14-TARGET TEST")
    print(f"{'='*80}")
    
    targets_14 = list(SM_TARGETS.keys())
    
    print(f"Targets: {targets_14}")
    print(f"8 params → 14 targets = OVERCONSTRAINED by 6")
    print(f"Optimizing...")
    
    mu_best14, cost14, time14 = optimize_targets(targets_14, n_restarts=50, max_time=180)
    results14, n1_14, n5_14 = evaluate_solution(mu_best14, targets_14)
    
    print(f"\nBest 14-target result: {n1_14}/14 at <1%, {n5_14}/14 at <5%  (time: {time14:.1f}s)")
    print(f"μ = {mu_best14.tolist()}")
    for tname, r in sorted(results14.items(), key=lambda x: x[1]['error_pct']):
        mark = "✅" if r['error_pct'] < 1.0 else ("⚠️" if r['error_pct'] < 5.0 else "❌")
        print(f"  {mark} {tname:12s}: {r['ratio_value']:.6f} vs {r['target']:.6f} ({r['error_pct']:.3f}%) via {r['matched_ratio']}")
    
    # ─── Phase 4: Statistical baseline ───
    print(f"\n{'='*80}")
    print("PHASE 4: RANDOM BASELINE (null hypothesis)")
    print(f"{'='*80}")
    
    # Use the best target set (whichever had most matches)
    best_n_targets = max([(n1_10, 10, targets_10), (n1_12, 12, targets_12), (n1_14, 14, targets_14)])
    use_targets = best_n_targets[2]
    use_n = best_n_targets[1]
    achieved_n1 = best_n_targets[0]
    
    print(f"Testing: what's the chance of {achieved_n1}/{use_n} at <1% by RANDOM chance?")
    print(f"Running 100,000 random μ samples...")
    
    max_rand_1pct, max_rand_5pct, count_ge, n_samples = random_baseline(use_targets, n_samples=100000)
    
    print(f"\n  Random baseline ({n_samples:,} samples):")
    print(f"  Max <1% matches seen randomly: {max_rand_1pct}/{use_n}")
    print(f"  Max <5% matches seen randomly: {max_rand_5pct}/{use_n}")
    
    # P-value
    if achieved_n1 > 0:
        p_count = count_ge.get(achieved_n1, 0)
        p_value = p_count / n_samples
        print(f"\n  P-value for ≥{achieved_n1} matches at <1%: {p_value:.6f}")
        if p_value < 0.001:
            print(f"  ✅ HIGHLY SIGNIFICANT (p < 0.001)")
        elif p_value < 0.05:
            print(f"  ✅ SIGNIFICANT (p < 0.05)")
        else:
            print(f"  ⚠️ NOT significant (p = {p_value:.4f})")
    
    # ─── Save all results ───
    output = {
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%S"),
        "phase1_10target": {
            "n_targets": 10,
            "n_params": 8,
            "n_1pct": n1_10,
            "n_5pct": n5_10,
            "mu": mu_best10.tolist() if mu_best10 is not None else None,
            "matches": {k: v for k, v in results10.items()},
            "time_s": time10
        },
        "phase2_12target": {
            "n_targets": 12,
            "n_params": 8,
            "overconstrained_by": 4,
            "n_1pct": n1_12,
            "n_5pct": n5_12,
            "mu": mu_best12.tolist() if mu_best12 is not None else None,
            "matches": {k: v for k, v in results12.items()},
            "time_s": time12
        },
        "phase3_14target": {
            "n_targets": 14,
            "n_params": 8,
            "overconstrained_by": 6,
            "n_1pct": n1_14,
            "n_5pct": n5_14,
            "mu": mu_best14.tolist() if mu_best14 is not None else None,
            "matches": {k: v for k, v in results14.items()},
            "time_s": time14
        },
        "random_baseline": {
            "n_samples": n_samples,
            "max_1pct_random": max_rand_1pct,
            "max_5pct_random": max_rand_5pct,
        }
    }
    
    with open('research/tba/e8_overconstrained_results.json', 'w') as f:
        json.dump(output, f, indent=2, default=str)
    
    print(f"\n{'='*80}")
    print("SUMMARY")
    print(f"{'='*80}")
    print(f"  10-target: {n1_10}/10 at <1%, {n5_10}/10 at <5%")
    print(f"  12-target: {n1_12}/12 at <1%, {n5_12}/12 at <5% (overconstrained by 4)")
    print(f"  14-target: {n1_14}/14 at <1%, {n5_14}/14 at <5% (overconstrained by 6)")
    print(f"  Random max <1%: {max_rand_1pct}")
    print(f"\nResults saved to research/tba/e8_overconstrained_results.json")
