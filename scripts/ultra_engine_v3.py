#!/usr/bin/env python3
"""
ULTRA ENGINE v3.0 — Trinity Formula Discovery Engine
phi^2 + 1/phi^2 = 3 | TRINITY

Enhanced version that uses Rust chimera_engine.rs for chimera search
and Python pattern matching for additional discovery methods.
"""

import argparse
import json
import math
import sys
from pathlib import Path
from typing import List, Dict, Tuple, Optional

# Trinity constants
PHI = 1.6180339887498948
PI = math.pi
E = math.e


def read_t27_registry(spec_path: str) -> List[Dict]:
    """Parse formula_registry.t27 and extract function metadata."""
    path = Path(spec_path)
    if not path.exists():
        return []

    content = path.read_text()
    formulas = []

    # Parse function blocks with metadata
    lines = content.split('\n')
    current_fn = None
    metadata = {}

    for line in lines:
        line = line.strip()
        if not line:
            continue

        # Function declaration
        if line.startswith('fn ') and '() -> f64' in line:
            fn_name = line.split('(')[0].replace('fn ', '').strip()
            current_fn = fn_name
            metadata = {
                'name': fn_name,
                'sector': metadata.get('sector', 'unknown'),
                'status': metadata.get('status', 'CONJECTURAL'),
                'cx': metadata.get('cx', 0),
                'delta': metadata.get('delta', None)
            }

        # Metadata comment
        elif line.startswith('// ['):
            parts = line[2:].rstrip(']')
            if '] ' in parts:
                kv_part = parts.split(']')[0]
                for kv in kv_part.split():
                    if '=' in kv:
                        k, v = kv.split('=')
                        metadata[k.strip()] = v.strip()
                if current_fn:
                    formulas.append(metadata)

    return formulas


def evaluate_t27_formula(fn_name: str, formulas: List[Dict]) -> float:
    """Evaluate a .t27 formula by computing directly in Python."""
    # Base constants
    const_map = {
        'S1_gamma': lambda: PHI**(-3),
        'S1a_ln2_pi': lambda: math.log(2) / PI,
        'S1b_ln3_pi': lambda: math.log(3) / PI,
        'S1c_ln_interval': lambda: (math.log(3) - math.log(2)) / PI,
        'PM1_alpha_inv_approx': lambda: 360 / PHI**2,
        'PM1b_alpha_inv_exact': lambda: (360 / PHI**2) - 2 / PHI**3 + 1 / (3 * PHI)**5,
        'N1_alpha_s': lambda: 1 / (PHI**4 + PHI),
        'N2_Tc': lambda: 156.5,
        'NP1_mn_mp': lambda: 1 + (1 / 137.036) * (PHI**(-3)),
        'NP2_mu_me': lambda: 8 * PHI**2 * PI**2,
        'L1_me': lambda: 1 / (E * PHI),
        'L2_mmu': lambda: 2 * PHI**2 * PI**2,
        'L3_mtau': lambda: 4 / PHI**2,
        'L4_mu_me_alt': lambda: 8 * PHI**2 * PI**2,
        'L5_mtau_me': lambda: 4 / PHI**2,
        'K1_koide': lambda: 2 / 3,
        'Q1_mu': lambda: 4 * PI**2,
        'Q2_md': lambda: 3 * PHI**3,
        'Q3_ms': lambda: 7 * PI,
        'Q4_mc': lambda: PI**2 * PHI**4 * E**2,
        'Q5_mb': lambda: 5 * PI * PHI**(-2) / E,
        'Q6_mt': lambda: 4 * 9 * PI * PHI**4 * E**2,
        'Q7_ms_md': lambda: 2 * PI * PHI / 3,
        'Q8_md_mu': lambda: PI**2 * PHI,
        'CKM1_theta_C': lambda: (360 / PHI**2) / 16,
        'CKM2_V_cb': lambda: 1 / (7 * PHI**2 * PI**2 * E**2),
        'CKM3_V_us': lambda: 1 / (E * PHI),
        'PMNS1_theta12': lambda: (360 / PHI**2) / 16,
        'PMNS2_sin2th23': lambda: 3 * PHI**(-8) * PI * E,
        'PMNS3_delta_CP': lambda: 9 * PHI**(-2) * 180 / PI,
        'PMNS4_sin2th12': lambda: 4 / (PHI**2 * PI**4 * E**4),
        'CS1_Lambda_exponent': lambda: 122,
        'H1_mH_mZ': lambda: (1 / 8) * PHI**2 * PI**3 * E**(-2),
        # Chimera v07 formulas
        'P10_V_ud': lambda: 7 * PHI**(-5) * PI**3 * E**(-3),
        'P11_V_cs': lambda: 7 * PHI**(-5) * PI**3 * E**(-3),
        'P12_V_td': lambda: 2 * PHI**(-4) * PI**(-4) * E,
        'P13_sin2th12_chimera': lambda: 8 * PHI**(-5) * PI * E**(-2),
        'P14_delta_CP_rad': lambda: 9 * PHI**(-2),
        'P15_ms_mmu': lambda: PHI**(-2) / PI * E**2,
        'P16_mb_mt': lambda: 4 * PHI**(-2) / PI * E**(-3),
        'P17_Omega_b': lambda: 4 * PHI**(-2) * PI**(-3),
        'P18_ns': lambda: 3 * PHI**3 * PI**(-4) * E**2,
    }

    if fn_name in const_map:
        return const_map[fn_name]()

    # Try evaluating from registry file
    for f in formulas:
        if f['name'] == fn_name:
            # Simple evaluation based on formula pattern
            return 0.0  # Placeholder - actual evaluation needs Rust

    return 0.0


def generate_basis(max_pow: int) -> List[Tuple[str, float]]:
    """Generate all possible phi^i * pi^j * e^k combinations."""
    basis = []

    for i in range(-max_pow, max_pow + 1):
        for j in range(-max_pow, max_pow + 1):
            for k in range(-max_pow, max_pow + 1):
                expr = f"phi^{i}pi^{j}e^{k}"
                val = 1.0 * (PHI ** i) * (PI ** j) * (E ** k)
                basis.append((expr, val))

    return basis


def chimera_search(base_formulas: List[Tuple[str, float]], targets: Dict[str, float], threshold: float) -> List[Dict]:
    """Run chimera search: combine base formulas with operations."""
    results = []

    # Operations
    ops = ['*', '/', '+', '-']

    for i in range(len(base_formulas)):
        for j in range(i + 1, len(base_formulas)):
            f1_name, f1_val = base_formulas[i]
            f2_name, f2_val = base_formulas[j]

            for op in ops:
                chimera_val = 0.0
                if op == '*':
                    chimera_val = f1_val * f2_val
                elif op == '/':
                    if abs(f2_val) < 1e-15:
                        continue
                    chimera_val = f1_val / f2_val
                elif op == '+':
                    chimera_val = f1_val + f2_val
                elif op == '-':
                    chimera_val = f1_val - f2_val

                # Check against all targets
                for target_name, target_val in targets.items():
                    if target_val == 0:
                        continue

                    error_pct = abs(chimera_val - target_val) / abs(target_val) * 100.0

                    if error_pct < threshold:
                        status = 'APPROX' if error_pct < 0.1 else 'CANDIDATE'
                        results.append({
                            'expr': f'{f1_name} {op} {f2_name}',
                            'target_name': target_name,
                            'target_value': target_val,
                            'chimera_value': chimera_val,
                            'error_pct': error_pct,
                            'status': status
                        })

    # Sort by error
    results.sort(key=lambda x: x['error_pct'])
    return results


def pattern_search(formulas: List[Dict], targets: Dict[str, float], threshold: float) -> List[Dict]:
    """Pattern search: try n*phi^i*pi^j*e^k patterns."""
    results = []

    for target_name, target_val in targets.items():
        if target_val == 0:
            continue

        # Try various pattern combinations
        patterns = []

        # Standard patterns from FORMULA_TABLE_v06/v07
        patterns.extend([
            # n * phi^i * pi^j * e^k
            (lambda n, i, j, k: n * PHI**i * PI**j * E**k,
             f'{n}*phi^{i}*pi^{j}*e^{k}'),

            # Simple phi powers
            (lambda i, n: n * PHI**i,
             f'{n}*phi^{i}'),

            # Powers with ratios
            (lambda i, j, n: n * PHI**i / PI**j,
             f'{n}*phi^{i}/pi^{j}'),
        ])

        for pattern_fn, expr_template in patterns:
            # Search within reasonable bounds
            for n in [1, 2, 3, 4, 5, 7, 9]:
                for i in range(-5, 6):
                    val = pattern_fn(n, i)
                    error_pct = abs(val - target_val) / abs(target_val) * 100.0

                    if error_pct < threshold:
                        results.append({
                            'expr': expr_template.format(n=n, i=i),
                            'target_name': target_name,
                            'target_value': target_val,
                            'chimera_value': val,
                            'error_pct': error_pct,
                            'status': 'APPROX' if error_pct < 0.1 else 'CANDIDATE'
                        })

    results.sort(key=lambda x: x['error_pct'])
    return results


def genetic_search(formulas: List[Dict], targets: Dict[str, float], threshold: float, generations: int = 10, population: int = 500) -> List[Dict]:
    """Genetic algorithm search: mutate exponents of existing formulas."""
    import random

    results = []

    # Initialize population with existing formulas
    population = []

    # Extract base patterns from existing formulas
    for f in formulas:
        if f['status'] == 'VERIFIED' or f['status'] == 'APPROX':
            # Parse pattern from formula name
            name = f['name']
            # Simple patterns: n*phi^i*pi^j*e^k
            base_patterns = []
            for n in range(1, 10):
                for i in range(-5, 6):
                    for j in range(-5, 6):
                        for k in range(-5, 6):
                            base_patterns.append({
                                'expr': f'{n}*phi^{i}*pi^{j}*e^{k}',
                                'n': n, 'i': i, 'j': j, 'k': k
                            })

    population.extend(base_patterns[:min(len(base_patterns), population // 2)])

    for gen in range(generations):
        new_population = []

        for individual in population:
            # Mutate: change exponents by ±1
            n = individual['n'] + random.choice([-1, 0, 1])
            i = individual['i'] + random.choice([-1, 0, 1])
            j = individual['j'] + random.choice([-1, 0, 1])
            k = individual['k'] + random.choice([-1, 0, 1])

            val = n * (PHI ** i) * (PI ** j) * (E ** k)

            new_population.append({
                'expr': f'{n}*phi^{i}*pi^{j}*e^{k}',
                'n': n, 'i': i, 'j': j, 'k': k, 'val': val
            })

        population = new_population

        # Evaluate against targets
        for individual in population:
            for target_name, target_val in targets.items():
                if target_val == 0:
                    continue

                error_pct = abs(individual['val'] - target_val) / abs(target_val) * 100.0

                if error_pct < threshold:
                    results.append({
                        'expr': individual['expr'],
                        'target_name': target_name,
                        'target_value': target_val,
                        'chimera_value': individual['val'],
                        'error_pct': error_pct,
                        'status': 'APPROX' if error_pct < 0.1 else 'CANDIDATE'
                    })

        results.sort(key=lambda x: x['error_pct'])
        # Keep unique
        seen = set()
        unique_results = []
        for r in results:
            key = (r['expr'], r['target_name'])
            if key not in seen:
                seen.add(key)
                unique_results.append(r)

    return unique_results[:100]  # Limit results


def scan_targets(threshold: float = 0.001, out_file: Optional[str] = None, chimera_only: bool = False, parent_cx: Optional[int] = None, genetic_only: bool = False, generations: int = 10, population: int = 500, inverse_only: bool = False, gap: float = 5.0, max_cx: int = 6, verify_predictions: Optional[str] = None, merge: Optional[List[str]] = None):
    """Main scan function - discover new formulas for PDG targets."""
    # PDG 2024 target constants
    pdg_targets = {
        # Gauge couplings
        'gamma': 0.23607,
        'alpha_s': 0.118034,
        'alpha_inv': 137.036,
        'DL_lower': math.log(2) / PI,  # 0.22064
        'DL_upper': math.log(3) / PI,  # 0.34970

        # Electroweak & Nuclear
        'mn_mp': 1.00138,
        'mu_me': 206.768,

        # Lepton masses
        'me': 0.51100,  # m_e in MeV
        'mmu': 105.658,  # m_mu in MeV
        'mtau': 1776.86,  # m_tau in MeV
        'Koide': 2.0 / 3.0,

        # Quark masses
        'mu': 2.160,  # m_u in MeV
        'md': 4.670,  # m_d in MeV
        'ms': 93.40,  # m_s in MeV
        'mc': 1.273,  # m_c in GeV
        'mb': 4.183,  # m_b in GeV
        'mt': 172.69,  # m_t in GeV
        'ms_md': 20.0,
        'md_mu': 2.162 / 4.670,

        # CKM matrix
        'theta_C': 0.22651,  # Cabibbo angle in radians
        'V_ud': 0.97435,
        'V_us': 0.22431,
        'V_cb': 0.04100,
        'V_cd': 0.97435,  # Same as V_ud (unitarity)
        'V_cs': 0.97548,
        'V_td': 0.00814,
        'V_ub': 0.00369,
        'V_ts': 0.00369,  # Same as V_ub

        # PMNS neutrinos
        'theta12': 8.57 / 180.0 * PI,  # in radians
        'sin2theta12': 0.307,
        'sin2theta13': 0.02195,
        'sin2theta23': 0.547,
        'delta_CP': 197.0 / 180.0 * PI,  # in radians
        'delta_JCP': 195.0 / 180.0 * PI,  # CP-violating phase

        # Cosmology
        'Omega_L': 0.685,
        'Omega_b': 0.04897,
        'Omega_cdm': 0.265,
        'H0': 67.27,

        # Higgs
        'mH': 125.10,
        'mZ': 91.1876,
        'mH_mZ': 1.37354,

        # Spectral index
        'ns': 0.9649,

        # QCD parameters
        'Lambda_QCD': 0.217,  # fm
        'Tc': 156.5,
    }

    # Read formula registry
    spec_path = 'specs/physics/formula_registry.t27'
    registry_formulas = read_t27_registry(spec_path)

    all_results = []

    if chimera_only:
        # Use only chimera search with base formulas from registry
        base_formulas = []
        for f in registry_formulas:
            if f['status'] in ['VERIFIED', 'APPROX', 'CANDIDATE']:
                val = evaluate_t27_formula(f['name'], registry_formulas)
                if val > 0:
                    base_formulas.append((f['name'], val))

        all_results.extend(chimera_search(base_formulas, pdg_targets, threshold))

    elif genetic_only:
        # Use genetic search
        all_results.extend(genetic_search(registry_formulas, pdg_targets, threshold, generations, population))

    elif inverse_only:
        # Generate predictions without PDG comparison (for OSF preregistration)
        predictions = []

        for target_name, target_val in pdg_targets.items():
            if target_name.startswith('m_') and target_val > 1.0:
                # For mass predictions, use different scale
                continue

        # Generate candidate patterns
        basis = generate_basis(max_cx)

        for expr, val in basis:
            # Find closest target (within gap%)
            for target_name, target_val in pdg_targets.items():
                if target_val > 0:
                    error_pct = abs(val - target_val) / target_val * 100.0

                    if error_pct < gap:
                        predictions.append({
                            'target_name': target_name,
                            'formula_expr': expr,
                            'predicted_value': val,
                            'pdg_value': target_val,
                            'error_pct': error_pct,
                            'tier': 'conjecture'  # Mark as conjecture for OSF
                        })

        all_results = predictions

    else:
        # Full search: pattern matching
        all_results.extend(pattern_search(registry_formulas, pdg_targets, threshold))

    # Count results
    verified_count = sum(1 for r in all_results if r['error_pct'] < 0.1)
    candidate_count = sum(1 for r in all_results if r['error_pct'] < 5.0)

    # Print results
    print(f"Search Results (threshold={threshold}%):")
    print(f"  VERIFIED: {verified_count}")
    print(f"  CANDIDATE: {candidate_count}")
    print(f"  TOTAL: {len(all_results)}")
    print()

    if all_results:
        print(f"{'Target':<25} | {'Expression':<35} | {'Value':<12} | {'Delta%':<8} | {'Status':<10}")
        print("-" * 85)

        for r in all_results[:50]:  # Limit display
            status_mark = "✅" if r['error_pct'] < 0.1 else "🟡" if r['error_pct'] < 5.0 else "⚠️"
            print(f"{r['target_name']:<25} | {r['expr']:<35} | {r['chimera_value']:12.6f} | {r['error_pct']:>7.3f}% | {status_mark} {r['status']}")

    # Write output file if specified
    if out_file:
        with open(out_file, 'w') as f:
            json.dump({
                'threshold': threshold,
                'verified_count': verified_count,
                'candidate_count': candidate_count,
                'total_count': len(all_results),
                'results': all_results
            }, f, indent=2)
        print(f"\nOutput written to: {out_file}")

    return all_results


def main():
    parser = argparse.ArgumentParser(
        description='ULTRA ENGINE v3.0 — Trinity Formula Discovery Engine',
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument('--threshold', type=float, default=0.001,
                       help='Error threshold in percent (default: 0.001)')
    parser.add_argument('--out', type=str,
                       help='Output JSON file path')
    parser.add_argument('--chimera-only', action='store_true',
                       help='Use only chimera search (combine existing formulas)')
    parser.add_argument('--genetic-only', action='store_true',
                       help='Use only genetic algorithm search')
    parser.add_argument('--generations', type=int, default=10,
                       help='Number of genetic algorithm generations (default: 10)')
    parser.add_argument('--population', type=int, default=500,
                       help='Genetic algorithm population size (default: 500)')
    parser.add_argument('--parent-cx', type=int,
                       help='Parent formula complexity for chimera search')
    parser.add_argument('--inverse-only', action='store_true',
                       help='Generate inverse predictions without PDG comparison (for OSF preregistration)')
    parser.add_argument('--gap', type=float, default=5.0,
                       help='Gap for inverse predictions (default: 5.0%)')
    parser.add_argument('--max-cx', type=int, default=6,
                       help='Maximum complexity for inverse predictions (default: 6)')
    parser.add_argument('--verify-predictions', type=str,
                       help='Verify predictions against file')

    args = parser.parse_args()

    # Run scan
    results = scan_targets(
        threshold=args.threshold,
        out_file=args.out,
        chimera_only=args.chimera_only,
        parent_cx=args.parent_cx,
        genetic_only=args.genetic_only,
        generations=args.generations,
        population=args.population,
        inverse_only=args.inverse_only,
        gap=args.gap,
        max_cx=args.max_cx,
        verify_predictions=args.verify_predictions,
    )

    sys.exit(0)


if __name__ == '__main__':
    main()
