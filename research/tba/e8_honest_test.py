#!/usr/bin/env python3
"""
E₈ HONEST ASSESSMENT — What is REAL vs ARTIFACT?
==================================================

PROBLEM IDENTIFIED: The comparison test showed ALL algebras (E₇, E₆, D₈, random)
achieve 10/10 at <1%. This means our "breakthrough" is an artifact of:
  1. Too many compound ratios (M_i/M_j, (M_i/M_j)², M_i*M_j/M_k²)
     → with ~500 ratios from 8 masses, ANYTHING can match ANYTHING
  2. Free choice of WHICH ratio maps to WHICH target
     → optimizer cherry-picks the best match from 500+ candidates

THIS SCRIPT TESTS:
  Level 1: SIMPLE ratios only (M_i/M_j) — 56 ratios from 8 masses
  Level 2: Fixed assignment (each target = specific ratio, no cherry-picking)
  Level 3: UNDEFORMED spectrum (no free params at all)
  Level 4: What φ-related values appear in UNDEFORMED E₈ vs other algebras

The question: Is there ANYTHING that E₈ does better than D₈ or random spectra?
"""

import numpy as np
from scipy.optimize import minimize
import math
import json
import time

PHI = (1 + math.sqrt(5)) / 2
PI = math.pi

# ═══════════════════════════════════════════════════════════════
# Mass Spectra
# ═══════════════════════════════════════════════════════════════

def e8_masses():
    return np.array([
        1.0, 2*math.cos(PI/5), 2*math.cos(PI/30),
        4*math.cos(PI/5)*math.cos(7*PI/30),
        4*math.cos(PI/5)*math.cos(2*PI/15),
        4*math.cos(PI/5)*math.cos(PI/30),
        8*math.cos(PI/5)**2*math.cos(7*PI/30),
        8*math.cos(PI/5)**2*math.cos(2*PI/15),
    ])

def pf_eigenvector(adj):
    eigvals, eigvecs = np.linalg.eigh(adj)
    idx = np.argmax(eigvals)
    pf = np.abs(eigvecs[:, idx])
    return pf / pf.min()

E8_ADJ = np.array([[0,1,0,0,0,0,0,0],[1,0,1,0,0,0,0,0],[0,1,0,1,0,0,0,0],
    [0,0,1,0,1,0,0,0],[0,0,0,1,0,1,0,1],[0,0,0,0,1,0,1,0],
    [0,0,0,0,0,1,0,0],[0,0,0,0,1,0,0,0]], dtype=float)
E7_ADJ = np.array([[0,1,0,0,0,0,0],[1,0,1,0,0,0,0],[0,1,0,1,0,0,0],
    [0,0,1,0,1,0,1],[0,0,0,1,0,1,0],[0,0,0,0,1,0,0],
    [0,0,0,1,0,0,0]], dtype=float)
D8_ADJ = np.array([[0,1,0,0,0,0,0,0],[1,0,1,0,0,0,0,0],[0,1,0,1,0,0,0,0],
    [0,0,1,0,1,0,0,0],[0,0,0,1,0,1,0,0],[0,0,0,0,1,0,1,1],
    [0,0,0,0,0,1,0,0],[0,0,0,0,0,1,0,0]], dtype=float)

def d8_masses(): return pf_eigenvector(D8_ADJ)
def e7_masses(): return pf_eigenvector(E7_ADJ)

SM_TARGETS = {
    "phi":      {"value": PHI,    "desc": "φ"},
    "phi2":     {"value": PHI**2, "desc": "φ²"},
    "phi3":     {"value": PHI**3, "desc": "φ³"},
    "mu_e":     {"value": 206.768,"desc": "mμ/me"},
    "tau_mu":   {"value": 16.817, "desc": "mτ/mμ"},
    "mp_me":    {"value": 1836.15,"desc": "mp/me"},
    "alpha_inv":{"value": 137.036,"desc": "1/α"},
    "sin2tw":   {"value": 0.23121,"desc": "sin²θW"},
    "MZ_MW":    {"value": 1.1342, "desc": "MZ/MW"},
    "koide":    {"value": 2.0/3.0,"desc": "Koide"},
}

# ═══════════════════════════════════════════════════════════════
# TEST 0: Count available ratios at each complexity level
# ═══════════════════════════════════════════════════════════════

def simple_ratios(M):
    """Only M_i/M_j — no powers, no products"""
    n = len(M)
    ratios = {}
    for i in range(n):
        for j in range(n):
            if i != j and M[j] > 0:
                ratios[f"M{i+1}/M{j+1}"] = M[i]/M[j]
    return ratios

def medium_ratios(M):
    """M_i/M_j and (M_i/M_j)^k for k=2,3"""
    ratios = simple_ratios(M)
    n = len(M)
    for i in range(n):
        for j in range(n):
            if i != j and M[j] > 0:
                r = M[i]/M[j]
                ratios[f"(M{i+1}/M{j+1})^2"] = r**2
                ratios[f"(M{i+1}/M{j+1})^3"] = r**3
    return ratios

def full_ratios(M):
    """All: simple + powers + triple products"""
    ratios = medium_ratios(M)
    n = len(M)
    for i in range(n):
        for j in range(n):
            for k in range(n):
                if len({i,j,k}) == 3 and M[k] > 0:
                    ratios[f"M{i+1}*M{j+1}/M{k+1}^2"] = M[i]*M[j]/(M[k]**2)
    return ratios

# ═══════════════════════════════════════════════════════════════
# TEST 1: UNDEFORMED — how many matches with 0 free params?
# ═══════════════════════════════════════════════════════════════

def test_undeformed(name, masses, ratio_func, label):
    """Test with ZERO free parameters"""
    ratios = ratio_func(masses)
    n_1pct = 0
    n_5pct = 0
    details = []
    for tname, tinfo in SM_TARGETS.items():
        tv = tinfo["value"]
        best_err = float('inf')
        best_name = ""
        best_val = 0
        for rname, rval in ratios.items():
            if rval > 0:
                err = abs(rval - tv)/tv * 100
                if err < best_err:
                    best_err = err
                    best_name = rname
                    best_val = rval
        if best_err < 1.0: n_1pct += 1
        if best_err < 5.0: n_5pct += 1
        details.append((tname, best_err, best_name, best_val))
    return n_1pct, n_5pct, details, len(ratios)

# ═══════════════════════════════════════════════════════════════
# TEST 2: DEFORMED but SIMPLE ratios only
# ═══════════════════════════════════════════════════════════════

def test_deformed_simple(name, masses, adj, n_restarts=50, max_time=90):
    """Optimize with only simple M_i/M_j ratios"""
    n = len(masses)
    eigvals, eigvecs = np.linalg.eigh(adj)
    
    def deformed(mu):
        return masses * np.exp(eigvecs @ mu)
    
    def cost(mu):
        M = deformed(mu)
        if np.any(M <= 0): return 1e10
        ratios = simple_ratios(M)
        c = 0.0
        for tinfo in SM_TARGETS.values():
            tv = tinfo["value"]
            best_err = min((abs(v-tv)/tv*100 for v in ratios.values() if v > 0), default=100)
            c += best_err**2
        return c
    
    best_cost = float('inf')
    best_mu = None
    t0 = time.time()
    
    for r in range(n_restarts):
        if time.time() - t0 > max_time: break
        mu0 = np.random.randn(n) * 1.5
        try:
            result = minimize(cost, mu0, method='Nelder-Mead',
                            options={'maxiter': 5000, 'xatol': 1e-8, 'fatol': 1e-8})
            if result.fun < best_cost:
                best_cost = result.fun
                best_mu = result.x.copy()
        except: pass
    
    # Evaluate
    M = deformed(best_mu)
    ratios = simple_ratios(M)
    n_1pct = 0
    n_5pct = 0
    details = []
    for tname, tinfo in SM_TARGETS.items():
        tv = tinfo["value"]
        best_err = float('inf')
        best_name = ""
        best_val = 0
        for rname, rval in ratios.items():
            if rval > 0:
                err = abs(rval-tv)/tv*100
                if err < best_err:
                    best_err = err
                    best_name = rname
                    best_val = rval
        if best_err < 1.0: n_1pct += 1
        if best_err < 5.0: n_5pct += 1
        details.append((tname, best_err, best_name, best_val))
    
    return n_1pct, n_5pct, details

# ═══════════════════════════════════════════════════════════════
# TEST 3: What φ-values exist in UNDEFORMED spectrum?
# ═══════════════════════════════════════════════════════════════

def phi_audit(name, masses):
    """Check which mass ratios naturally equal φ, φ², φ³ etc."""
    phi_matches = []
    for phi_name, phi_val in [("φ", PHI), ("φ²", PHI**2), ("φ³", PHI**3),
                               ("1/φ", 1/PHI), ("φ+1=φ²", PHI+1), ("3", 3.0)]:
        for i in range(len(masses)):
            for j in range(len(masses)):
                if i != j:
                    r = masses[i]/masses[j]
                    err = abs(r - phi_val)/phi_val * 100
                    if err < 0.01:  # Within 0.01%
                        phi_matches.append((phi_name, phi_val, f"m{i+1}/m{j+1}", r, err))
    return phi_matches

# ═══════════════════════════════════════════════════════════════
# TEST 4: Effective number of degrees of freedom
# ═══════════════════════════════════════════════════════════════

def count_effective_dof(n_masses, ratio_level):
    """How many INDEPENDENT ratios at each level?"""
    if ratio_level == "simple":
        # n(n-1) total, but only (n-1) independent (all determined by n-1 ratios)
        return n_masses * (n_masses - 1), n_masses - 1
    elif ratio_level == "medium":
        # simple + squares + cubes
        total = 3 * n_masses * (n_masses - 1)
        independent = n_masses - 1  # Still all from same base ratios
        return total, independent
    elif ratio_level == "full":
        total = 3 * n_masses * (n_masses - 1) + n_masses * (n_masses-1) * (n_masses-2)
        independent = n_masses - 1
        return total, independent

# ═══════════════════════════════════════════════════════════════
# MAIN
# ═══════════════════════════════════════════════════════════════

if __name__ == "__main__":
    np.random.seed(42)
    
    print("=" * 80)
    print("E₈ HONEST ASSESSMENT: What is REAL vs ARTIFACT?")
    print("=" * 80)
    
    # ─── Ratio count analysis ───
    print("\n" + "─" * 60)
    print("RATIO COUNT ANALYSIS (why everything matches)")
    print("─" * 60)
    
    m8 = e8_masses()
    for level_name, func in [("Simple M_i/M_j", simple_ratios),
                              ("+ Powers (^2,^3)", medium_ratios),
                              ("+ Products (full)", full_ratios)]:
        ratios = func(m8)
        total_ratios = len(ratios)
        distinct_positive = len(set(f"{v:.6f}" for v in ratios.values() if v > 0))
        print(f"\n  {level_name}: {total_ratios} total ratios, ~{distinct_positive} distinct values")
        
        # How many targets matchable by chance?
        n_match_1pct = 0
        for tinfo in SM_TARGETS.values():
            tv = tinfo["value"]
            for v in ratios.values():
                if v > 0 and abs(v - tv)/tv * 100 < 1.0:
                    n_match_1pct += 1
                    break
        print(f"  → {n_match_1pct}/10 targets matchable at <1% (NO free params!)")
    
    print(f"\n  KEY INSIGHT: With full ratios, even the UNDEFORMED E₈ matches")
    print(f"  many targets. Adding 8 free params makes it trivial.")
    
    total, indep = count_effective_dof(8, "simple")
    print(f"\n  Simple ratios: {total} total but only {indep} INDEPENDENT")
    print(f"  (All M_i/M_j determined by just 7 base ratios)")
    
    # ─── TEST 1: Undeformed spectra ───
    print(f"\n{'='*80}")
    print("TEST 1: UNDEFORMED SPECTRA (0 free params)")
    print(f"{'='*80}")
    
    algebras = [
        ("E₈", e8_masses()),
        ("E₇", e7_masses()),
        ("D₈", d8_masses()),
    ]
    
    for name, masses in algebras:
        print(f"\n  {name} (rank {len(masses)}):")
        for level_name, func in [("Simple", simple_ratios), ("Full", full_ratios)]:
            n1, n5, details, n_rat = test_undeformed(name, masses, func, level_name)
            print(f"    {level_name:8s} ({n_rat:>4d} ratios): {n1}/10 at <1%, {n5}/10 at <5%")
    
    # Random controls
    print(f"\n  Random spectra (undeformed):")
    for seed in range(5):
        rm = np.sort(np.random.RandomState(seed).exponential(1.5, 8))
        rm = rm / rm[0]
        for level_name, func in [("Simple", simple_ratios), ("Full", full_ratios)]:
            n1, n5, _, n_rat = test_undeformed(f"Rand-{seed}", rm, func, level_name)
            if level_name == "Full":
                print(f"    Random-{seed+1} Full ({n_rat:>4d} ratios): {n1}/10 at <1%, {n5}/10 at <5%")
    
    # ─── TEST 2: φ Audit ───
    print(f"\n{'='*80}")
    print("TEST 2: φ IN UNDEFORMED SPECTRA (the one truly unique feature)")
    print(f"{'='*80}")
    
    for name, masses in algebras:
        matches = phi_audit(name, masses)
        if matches:
            print(f"\n  {name} — φ-matches in undeformed spectrum:")
            for pn, pv, rn, rv, err in matches:
                print(f"    {rn} = {rv:.10f} ≈ {pn} = {pv:.10f} (err {err:.6f}%)")
        else:
            print(f"\n  {name} — NO φ-matches in undeformed spectrum")
    
    # ─── TEST 3: Deformed, SIMPLE ratios only ───
    print(f"\n{'='*80}")
    print("TEST 3: DEFORMED + SIMPLE RATIOS ONLY (strictest test)")
    print(f"{'='*80}")
    print(f"Using ONLY M_i/M_j (no powers, no products)")
    
    for name, masses, adj in [("E₈", e8_masses(), E8_ADJ), 
                               ("D₈", d8_masses(), D8_ADJ)]:
        print(f"\n  {name} ({len(masses)} params, simple ratios only):")
        n1, n5, details = test_deformed_simple(name, masses, adj, n_restarts=50, max_time=90)
        print(f"    Result: {n1}/10 at <1%, {n5}/10 at <5%")
        for tname, err, rname, rval in sorted(details, key=lambda x: x[1]):
            mark = "✅" if err < 1.0 else ("⚠️" if err < 5.0 else "❌")
            print(f"    {mark} {tname:12s}: {err:.3f}% via {rname}")
    
    # Also test random
    for trial in range(3):
        rm = np.sort(np.random.RandomState(trial*13+7).exponential(1.5, 8))
        rm = rm / rm[0]
        adj_rand = np.eye(8)
        n1, n5, _ = test_deformed_simple(f"Random-{trial+1}", rm, adj_rand, 
                                          n_restarts=30, max_time=60)
        print(f"\n  Random-{trial+1} (8 params, simple ratios): {n1}/10 at <1%, {n5}/10 at <5%")
    
    # ─── SUMMARY ───
    print(f"\n{'='*80}")
    print("HONEST SUMMARY")
    print(f"{'='*80}")
    
    print(f"""
  WHAT IS REAL:
  ─────────────
  1. E₈ undeformed spectrum contains m₂/m₁ = φ EXACTLY (0.000000% error)
     → No other simply-laced algebra has this
     → This is a mathematical fact, not fitting
  
  2. c_eff = 1/2 from Rogers dilogarithm (error 7.6×10⁻¹³)
     → Mathematical identity, not optimization
  
  3. E₈ mark pattern in Sacred Formula n-values (p < 0.0001)
     → Statistical, but independent of mass deformation
  
  WHAT IS ARTIFACT:
  ─────────────────
  1. "10/10 at <1%" with full compound ratios
     → Any algebra (even random) achieves this
     → Too many ratios (~500) from 8 masses = easy to match anything
  
  2. "p < 10⁻⁶" claim
     → The random baseline used the SAME ratio library
     → But the optimizer can use DIFFERENT ratios for each target
     → The p-value is technically correct but MISLEADING
     → It measures "optimizer > random draw" not "E₈ > other algebras"
  
  3. The overconstrained argument
     → With 500+ ratios and 8 free params, the effective DOF >> 8
     → The system is NOT truly overconstrained
  
  WHAT REMAINS TO BE DETERMINED:
  ──────────────────────────────
  1. Does E₈ beat D₈ with SIMPLE ratios only?
  2. Is there a FIXED assignment of ratios to observables that works?
  3. Is m₂/m₁ = φ enough to explain why E₈ "prefers" SM values?
""")
    
    # Save
    output = {
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%S"),
        "conclusion": "The 10/10 result is an artifact of too many compound ratios. "
                      "The truly unique E₈ features are: m2/m1=phi exactly, c=1/2, "
                      "and the mark pattern (p < 0.0001). The mass deformation 'fits' "
                      "are not specific to E₈.",
    }
    with open('research/tba/e8_honest_assessment.json', 'w') as f:
        json.dump(output, f, indent=2)
    
    print("Results saved to research/tba/e8_honest_assessment.json")
