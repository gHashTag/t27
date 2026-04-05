#!/usr/bin/env python3
"""
ALGEBRA COMPARISON: E₈ vs E₇ vs E₆ vs D₈ vs Random
=====================================================

Falsifiability test: if E₈ is truly special, the same mass-deformation
procedure applied to OTHER Lie algebras should give WORSE results.

For each algebra:
1. Compute the exact mass spectrum (Zamolodchikov-type for integrable cases)
2. Apply eigenvector-based deformation (same procedure)
3. Optimize against SM targets
4. Compare: how many targets match at <1%?

Algebras tested:
- E₈ (rank 8, h=30): 8 masses — our claim
- E₇ (rank 7, h=18): 7 masses (E₇ Toda has 7 particles)
- E₆ (rank 6, h=12): 6 masses
- D₈ (rank 8, h=14): 8 masses (same rank as E₈!)
- Random 8×8 symmetric: control

The key comparison is E₈ vs D₈: same number of parameters (8),
but D₈ is a classical algebra without φ in its spectrum.
"""

import numpy as np
from scipy.optimize import minimize
import math
import json
import time

PHI = (1 + math.sqrt(5)) / 2
PI = math.pi

# ═══════════════════════════════════════════════════════════════
# Mass Spectra for Different Algebras
# ═══════════════════════════════════════════════════════════════

def e8_masses():
    """E₈ Toda: 8 particles, m₂/m₁ = φ"""
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

def e7_masses():
    """E₇ Toda: 7 particles
    Mass ratios from Braden et al. (1990), Freeman (1991)
    m_a/m_1 = sin(a*pi/h)/sin(pi/h) for simply-laced
    with h = 18 for E₇.
    Actually the exact masses follow from E₇ Perron-Frobenius."""
    # E₇ Dynkin diagram: 1-2-3-4-5-6 with 7 branching from 4
    # Adjacency matrix
    adj = np.array([
        [0,1,0,0,0,0,0],
        [1,0,1,0,0,0,0],
        [0,1,0,1,0,0,0],
        [0,0,1,0,1,0,1],
        [0,0,0,1,0,1,0],
        [0,0,0,0,1,0,0],
        [0,0,0,1,0,0,0],
    ], dtype=float)
    # Perron-Frobenius eigenvector = mass ratios
    eigvals, eigvecs = np.linalg.eigh(adj)
    # PF eigenvector corresponds to largest eigenvalue
    idx = np.argmax(eigvals)
    pf = np.abs(eigvecs[:, idx])
    return pf / pf.min()

def e6_masses():
    """E₆ Toda: 6 particles
    E₆ Dynkin: 1-2-3-4-5 with 6 branching from 3"""
    adj = np.array([
        [0,1,0,0,0,0],
        [1,0,1,0,0,0],
        [0,1,0,1,0,1],
        [0,0,1,0,1,0],
        [0,0,0,1,0,0],
        [0,0,1,0,0,0],
    ], dtype=float)
    eigvals, eigvecs = np.linalg.eigh(adj)
    idx = np.argmax(eigvals)
    pf = np.abs(eigvecs[:, idx])
    return pf / pf.min()

def d8_masses():
    """D₈ Toda: 8 particles (same rank as E₈!)
    D₈ Dynkin: 1-2-3-4-5-6-7 with 8 branching from 6
    (linear chain 1-...-6, then 7 and 8 both connect to 6)"""
    adj = np.array([
        [0,1,0,0,0,0,0,0],
        [1,0,1,0,0,0,0,0],
        [0,1,0,1,0,0,0,0],
        [0,0,1,0,1,0,0,0],
        [0,0,0,1,0,1,0,0],
        [0,0,0,0,1,0,1,1],
        [0,0,0,0,0,1,0,0],
        [0,0,0,0,0,1,0,0],
    ], dtype=float)
    eigvals, eigvecs = np.linalg.eigh(adj)
    idx = np.argmax(eigvals)
    pf = np.abs(eigvecs[:, idx])
    return pf / pf.min()

def random_masses(n=8, seed=None):
    """Random positive spectrum with n masses"""
    rng = np.random.RandomState(seed)
    m = np.sort(rng.exponential(1.5, size=n))
    return m / m[0]  # Normalize to m_1 = 1

# ═══════════════════════════════════════════════════════════════
# Generic Deformation + Optimization
# ═══════════════════════════════════════════════════════════════

SM_TARGETS = {
    "phi": PHI,
    "phi2": PHI**2,
    "phi3": PHI**3,
    "mu_e": 206.768,
    "tau_mu": 16.817,
    "mp_me": 1836.15,
    "alpha_inv": 137.036,
    "sin2tw": 0.23121,
    "MZ_MW": 1.1342,
    "koide": 2.0/3.0,
}

def compute_ratios_generic(M):
    """Compute all mass ratios for any spectrum"""
    n = len(M)
    ratios = {}
    # Simple ratios
    for i in range(n):
        for j in range(n):
            if i != j and M[j] > 0:
                ratios[f"M{i+1}/M{j+1}"] = M[i] / M[j]
    # Compound ratios: products and powers
    for i in range(n):
        for j in range(n):
            if i != j and M[j] > 0:
                ratios[f"(M{i+1}/M{j+1})^2"] = (M[i] / M[j])**2
                ratios[f"(M{i+1}/M{j+1})^3"] = (M[i] / M[j])**3
    # Triple products
    for i in range(n):
        for j in range(n):
            for k in range(n):
                if i != j and j != k and i != k and M[k] > 0:
                    ratios[f"M{i+1}*M{j+1}/M{k+1}^2"] = M[i] * M[j] / (M[k]**2)
    return ratios

def best_match(ratios, target_val):
    best_err = float('inf')
    for name, val in ratios.items():
        if val > 0:
            err = abs(val - target_val) / target_val * 100
            if err < best_err:
                best_err = err
    return best_err

def run_algebra_test(name, base_masses, adjacency_matrix=None, n_restarts=50, max_time=120):
    """Run the full optimization test for a given algebra"""
    n = len(base_masses)
    
    # Eigenvectors for deformation
    if adjacency_matrix is not None:
        eigenvalues, eigenvectors = np.linalg.eigh(adjacency_matrix)
    else:
        # For random: use identity or random orthogonal
        eigenvectors = np.eye(n)
    
    def deformed(mu):
        log_shift = eigenvectors @ mu
        M = base_masses * np.exp(log_shift)
        return M
    
    target_names = list(SM_TARGETS.keys())
    
    def cost(mu):
        M = deformed(mu)
        if np.any(M <= 0):
            return 1e10
        ratios = compute_ratios_generic(M)
        total = 0.0
        for tname in target_names:
            tv = SM_TARGETS[tname]
            err = best_match(ratios, tv)
            total += err**2
        return total
    
    best_cost = float('inf')
    best_mu = None
    start_time = time.time()
    
    for restart in range(n_restarts):
        if time.time() - start_time > max_time:
            break
        mu0 = np.random.randn(n) * 1.5
        try:
            result = minimize(cost, mu0, method='Nelder-Mead',
                            options={'maxiter': 5000, 'xatol': 1e-8, 'fatol': 1e-8})
            if result.fun < best_cost:
                best_cost = result.fun
                best_mu = result.x.copy()
        except:
            continue
    
    elapsed = time.time() - start_time
    
    # Evaluate
    if best_mu is not None:
        M = deformed(best_mu)
        ratios = compute_ratios_generic(M)
        n_1pct = 0
        n_5pct = 0
        details = {}
        for tname in target_names:
            tv = SM_TARGETS[tname]
            err = best_match(ratios, tv)
            details[tname] = err
            if err < 1.0: n_1pct += 1
            if err < 5.0: n_5pct += 1
    else:
        n_1pct, n_5pct, details = 0, 0, {}
    
    return {
        "name": name,
        "rank": n,
        "n_params": n,
        "n_targets": len(target_names),
        "n_1pct": n_1pct,
        "n_5pct": n_5pct,
        "time_s": elapsed,
        "details": details,
    }

# ═══════════════════════════════════════════════════════════════
# MAIN
# ═══════════════════════════════════════════════════════════════

if __name__ == "__main__":
    np.random.seed(42)
    
    print("=" * 80)
    print("ALGEBRA COMPARISON: E₈ vs E₇ vs E₆ vs D₈ vs Random")
    print("=" * 80)
    
    # Define adjacency matrices
    E8_ADJ = np.array([
        [0,1,0,0,0,0,0,0],
        [1,0,1,0,0,0,0,0],
        [0,1,0,1,0,0,0,0],
        [0,0,1,0,1,0,0,0],
        [0,0,0,1,0,1,0,1],
        [0,0,0,0,1,0,1,0],
        [0,0,0,0,0,1,0,0],
        [0,0,0,0,1,0,0,0],
    ], dtype=float)
    
    E7_ADJ = np.array([
        [0,1,0,0,0,0,0],
        [1,0,1,0,0,0,0],
        [0,1,0,1,0,0,0],
        [0,0,1,0,1,0,1],
        [0,0,0,1,0,1,0],
        [0,0,0,0,1,0,0],
        [0,0,0,1,0,0,0],
    ], dtype=float)
    
    E6_ADJ = np.array([
        [0,1,0,0,0,0],
        [1,0,1,0,0,0],
        [0,1,0,1,0,1],
        [0,0,1,0,1,0],
        [0,0,0,1,0,0],
        [0,0,1,0,0,0],
    ], dtype=float)
    
    D8_ADJ = np.array([
        [0,1,0,0,0,0,0,0],
        [1,0,1,0,0,0,0,0],
        [0,1,0,1,0,0,0,0],
        [0,0,1,0,1,0,0,0],
        [0,0,0,1,0,1,0,0],
        [0,0,0,0,1,0,1,1],
        [0,0,0,0,0,1,0,0],
        [0,0,0,0,0,1,0,0],
    ], dtype=float)
    
    algebras = [
        ("E₈", e8_masses(), E8_ADJ),
        ("E₇", e7_masses(), E7_ADJ),
        ("E₆", e6_masses(), E6_ADJ),
        ("D₈", d8_masses(), D8_ADJ),
    ]
    
    # Print mass spectra
    for name, masses, _ in algebras:
        print(f"\n  {name} masses: {[f'{m:.4f}' for m in masses]}")
        if len(masses) >= 2:
            print(f"    m₂/m₁ = {masses[1]/masses[0]:.6f}" + 
                  (f" = φ" if abs(masses[1]/masses[0] - PHI) < 0.001 else ""))
    
    # Run tests
    results = []
    
    for name, masses, adj in algebras:
        print(f"\n{'='*60}")
        print(f"Testing {name} (rank {len(masses)}, {len(masses)} params)...")
        print(f"{'='*60}")
        
        r = run_algebra_test(name, masses, adj, n_restarts=50, max_time=120)
        results.append(r)
        
        print(f"  Result: {r['n_1pct']}/10 at <1%, {r['n_5pct']}/10 at <5% ({r['time_s']:.1f}s)")
        for tname, err in sorted(r['details'].items(), key=lambda x: x[1]):
            mark = "✅" if err < 1.0 else ("⚠️" if err < 5.0 else "❌")
            print(f"    {mark} {tname:12s}: {err:.3f}%")
    
    # Random controls (5 trials)
    print(f"\n{'='*60}")
    print(f"Random 8×8 controls (5 trials)...")
    print(f"{'='*60}")
    
    random_results = []
    for trial in range(5):
        rm = random_masses(8, seed=trial*17+3)
        r = run_algebra_test(f"Random-{trial+1}", rm, None, n_restarts=30, max_time=60)
        random_results.append(r)
        print(f"  Random-{trial+1}: {r['n_1pct']}/10 at <1%, {r['n_5pct']}/10 at <5%")
    
    avg_random_1pct = np.mean([r['n_1pct'] for r in random_results])
    max_random_1pct = max(r['n_1pct'] for r in random_results)
    
    # ─── Summary ───
    print(f"\n{'='*80}")
    print("COMPARISON SUMMARY")
    print(f"{'='*80}")
    print(f"\n  {'Algebra':12s} {'Rank':>5s} {'<1%':>5s} {'<5%':>5s}")
    print(f"  {'─'*32}")
    for r in results:
        marker = " ★" if r['name'] == 'E₈' else ""
        print(f"  {r['name']:12s} {r['rank']:>5d} {r['n_1pct']:>5d} {r['n_5pct']:>5d}{marker}")
    print(f"  {'Random avg':12s} {'8':>5s} {avg_random_1pct:>5.1f} {np.mean([r['n_5pct'] for r in random_results]):>5.1f}")
    print(f"  {'Random max':12s} {'8':>5s} {max_random_1pct:>5.0f}")
    
    # Check if E₈ is best
    e8_result = [r for r in results if r['name'] == 'E₈'][0]
    others_max = max(r['n_1pct'] for r in results if r['name'] != 'E₈')
    
    print(f"\n  E₈: {e8_result['n_1pct']}/10 at <1%")
    print(f"  Best non-E₈: {others_max}/10 at <1%")
    
    if e8_result['n_1pct'] > others_max:
        print(f"  ✅ E₈ IS THE BEST ALGEBRA — exceeds all others by {e8_result['n_1pct'] - others_max}")
    else:
        print(f"  ⚠️ E₈ is NOT uniquely best — {[r['name'] for r in results if r['n_1pct'] >= e8_result['n_1pct']]}")
    
    # Save
    output = {
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%S"),
        "results": results,
        "random_results": [{"name": r["name"], "n_1pct": r["n_1pct"], "n_5pct": r["n_5pct"]} for r in random_results],
        "conclusion": {
            "e8_1pct": e8_result['n_1pct'],
            "best_non_e8_1pct": others_max,
            "e8_is_best": e8_result['n_1pct'] > others_max,
        }
    }
    
    with open('research/tba/algebra_comparison_results.json', 'w') as f:
        json.dump(output, f, indent=2, default=str)
    
    print(f"\nResults saved to research/tba/algebra_comparison_results.json")
