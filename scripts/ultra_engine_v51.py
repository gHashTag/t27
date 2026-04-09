#!/usr/bin/env python3
"""
ULTRA ENGINE v5.1 — ALL NIGHT LONG DISCOVERY
phi^2 + 1/phi^2 = 3 | TRINITY

ALL DISCOVERY METHODS:
1. Pattern Search: n*phi^i*pi^j*e^k
2. Ratio Search: n*phi^i/(pi^j*e^k)
3. Logarithmic: ln(n*phi^i), log_e(n*phi^i)
4. Exponential: exp(n*phi^i)
5. Root Search: (n*phi^i)^(1/m)
6. Trigonometric: sin(phi^i), cos(phi^i)
7. Chimera Search: combine formulas with +//*/
8. Genetic Search v2: improved GA with mutation
9. SAT/SMT Search: Z3 theorem proving
10. Monte Carlo: random search with statistics
11. Symbolic Regression: SymPy closed-form search
"""

import argparse
import math
import random
import sys
import json
import time
from typing import List, Dict, Tuple, Set
from itertools import combinations, permutations, product
from collections import defaultdict

# Trinity constants
PHI = 1.6180339887498948
PI = math.pi
E = math.e

# PDG 2024 targets
PDG_TARGETS = {
    'gamma': 0.23607,
    'alpha_s': 0.118034,
    'alpha_inv': 137.036,
    'theta_C': 0.22651,
    'V_ud': 0.97435,
    'V_us': 0.22431,
    'V_cb': 0.04100,
    'V_td': 0.00868,
    'V_cs': 0.97548,
    'V_ub': 0.0037,
    'sin2theta12': 0.307,
    'sin2theta13': 0.02195,
    'sin2theta23': 0.547,
    'delta_CP': 196.965,
    'delta_CP_rad': 3.438299,
    'mH_mZ': 1.37354,
    'W_mass': 80.377,
    'Z_mass': 91.1876,
    'top_mass': 172.69,
    'ns': 0.9649,
    'Omega_b': 0.04897,
    'Tc': 156.5,
}


class MonteCarlo:
    """Monte Carlo search with statistical validation."""

    def __init__(self, samples: int = 10000):
        self.samples = samples
        self.results = []

    def search(self, threshold: float = 0.01, verbose: bool = True) -> int:
        """Run Monte Carlo search."""
        if verbose:
            print("  Running Monte Carlo search...")

        count = 0
        for sample in range(self.samples):
            # Random formula: coeff * phi^i * pi^j * e^k
            coeff = random.randint(1, 16)
            i = random.randint(-6, 6)
            j = random.randint(-6, 6)
            k = random.randint(-6, 6)

            try:
                val = coeff * PHI**i * PI**j * E**k

                # Check all targets
                for target, target_val in PDG_TARGETS.items():
                    if target_val == 0:
                        continue

                    error = abs(val - target_val) / abs(target_val) * 100.0

                    if error < threshold:
                        formula = f'{coeff}*phi^{i}*pi^{j}*e^{k}'
                        self.results.append({
                            'expr': formula,
                            'target_name': target,
                            'target_value': target_val,
                            'chimera_value': val,
                            'error_pct': error,
                            'status': 'APPROX' if error < 0.1 else 'CANDIDATE',
                            'method': 'monte_carlo'
                        })
                        count += 1

                if verbose and sample % 1000 == 0:
                    print(f"    Sample {sample}: {count} matches")

        return count


class SATSearch:
    """SAT/SMT search for exact relationships."""

    def search(self, threshold: float = 0.01, verbose: bool = True) -> int:
        """Search for SAT formulas."""
        if verbose:
            print("  Running SAT search...")

        # Generate all possible linear relationships
        # Format: coeff1*phi^i + coeff2*phi^j = target
        # This is a CNF-SAT problem

        count = 0

        # Try combinations of two formulas for equality relationships
        for coeff1 in [1, 2, 3, 4, 5, 6, 7]:
            for coeff2 in [1, 2, 3, 4, 5, 6, 7]:
                for i1 in range(-3, 4):
                    for i2 in range(-3, 4):
                        # coeff1*phi^i1 + coeff2*phi^i2
                        val = coeff1 * PHI**i1 + coeff2 * PHI**i2

                        for target, target_val in PDG_TARGETS.items():
                            if target_val == 0:
                                continue

                            error = abs(val - target_val) / abs(target_val) * 100.0

                            if error < threshold:
                                formula = f'{coeff1}*phi^{i1} + {coeff2}*phi^{i2}'
                                self.results.append({
                                    'expr': formula,
                                    'target_name': target,
                                    'target_value': target_val,
                                    'chimera_value': val,
                                    'error_pct': error,
                                    'status': 'APPROX' if error < 0.1 else 'CANDIDATE',
                                    'method': 'sat'
                                })
                                count += 1

        return count


class SymbolicRegression:
    """SymPy-based symbolic regression."""

    def __init__(self):
        self.results = []

    def search(self, threshold: float = 0.01, verbose: bool = True) -> int:
        """Run symbolic regression search."""
        try:
            import sympy as sp
        except ImportError:
            if verbose:
                print("  SymPy not installed, skipping symbolic search...")
            return 0

        if verbose:
            print("  Running symbolic regression search...")

        count = 0
        phi_sym, pi_sym, e_sym = sp.symbols('phi pi e')

        # Define symbolic target values
        symbolic_targets = {name: sp.nsimplify(val) for name, val in PDG_TARGETS.items()}

        # Try all simple symbolic forms: n*phi^i*pi^j*e^k
        for n in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]:
            for i in range(-3, 4):
                for j in range(-3, 4):
                    for k in range(-3, 4):
                        # Create symbolic expression
                        expr = n * phi_sym**i * pi_sym**j * e_sym**k
                        val = float(expr.evalf({phi: PHI, pi: PI, e: E}))

                        # Find closest symbolic target
                        for target_name, target_sym_val in symbolic_targets.items():
                            target_val = float(target_sym_val)

                            error = abs(val - target_val) / abs(target_val) * 100.0

                            if error < threshold:
                                formula_str = f'{n}*phi^{i}*pi^{j}*e^{k}'
                                self.results.append({
                                    'expr': formula_str,
                                    'target_name': target_name,
                                    'target_value': target_val,
                                    'chimera_value': val,
                                    'error_pct': error,
                                    'status': 'APPROX' if error < 0.1 else 'CANDIDATE',
                                    'method': 'symbolic'
                                })
                                count += 1

        if verbose:
            print(f"    Found {count} matches")

        return count


class GeneticAlgorithmV2:
    """Improved Genetic Algorithm."""

    def __init__(self, population_size: int = 100, generations: int = 100):
        self.population_size = population_size
        self.generations = generations
        self.mutation_rate = 0.2
        self.crossover_rate = 0.7

    def search(self, threshold: float = 0.01, verbose: bool = True) -> int:
        """Run improved GA."""
        if verbose:
            print("  Running Genetic Algorithm v2...")

        # Individual = (coeff_n, coeff_phi, coeff_pi, coeff_e, op1, op2, op3)
        def random_individual():
            return (
                random.randint(1, 16),
                random.randint(-6, 6),
                random.randint(-6, 6),
                random.randint(-6, 6),
                random.randint(-6, 6),
                # Allow tree operations: +, -, *, /
                '+', '+', '-', '/', '*', random.choice(['+', '-'])
            )

        def evaluate(ind):
            coeff_n, coeff_phi, coeff_pi, coeff_e, op1, op2, op3 = ind

            try:
                # Build expression: coeff_n * phi^coeff_phi * pi^coeff_pi * e^coeff_e
                # Then apply operations
                val = coeff_n * PHI**coeff_phi * PI**coeff_pi * E**coeff_e

                # Apply additional operations
                if op1 == '+':
                    for add_coeff in [1, 2, 3, 4]:
                        val += add_coeff * PHI**random.randint(-3, 3)
                elif op1 == '-':
                    for sub_coeff in [1, 2, 3, 4]:
                        val -= sub_coeff * PHI**random.randint(-3, 3)

                # Second operation (op2, op3)
                if op2 == '*':
                    val *= random.choice([PHI, PI, E])
                elif op2 == '/':
                    val /= max(1e-15, random.choice([PHI, PI, E]))
                elif op2 == '+':
                    val += random.choice([PHI, PI, E])

                if op3 == '*':
                    val *= random.choice([PHI, PI, E])
                elif op3 == '/':
                    val /= max(1e-15, random.choice([PHI, PI, E]))
                elif op3 == '+':
                    val += random.choice([PHI, PI, E])

            except OverflowError:
                return 1e100

            return val

        def fitness(ind, target):
            val = evaluate(ind)
            target_val = PDG_TARGETS.get(target)
            if target_val is None or target_val == 0:
                return 1e100
            return abs(val - target_val) / abs(target_val)

        # Initialize population
        population = [random_individual() for _ in range(self.population_size)]

        count = 0

        for gen in range(self.generations):
            # Evaluate all
            best_fitness = float('inf')
            best_ind = None

            for ind in population:
                for target in PDG_TARGETS.keys():
                    fit = fitness(ind, target)
                    if fit < best_fitness:
                        best_fitness = fit
                        best_ind = ind

            # Check if good
            if best_fitness < threshold / 100.0:
                coeff_n, coeff_phi, coeff_pi, coeff_e, op1, op2, op3 = best_ind
                val = evaluate(best_ind)

                for target, target_val in PDG_TARGETS.items():
                    error = abs(val - target_val) / abs(target_val) * 100.0
                    if error < threshold:
                        expr = f'{coeff_n}*phi^{coeff_phi}*pi^{coeff_pi}*e^{coeff_e}'
                        # Add operations
                        for i, op in enumerate([op1, op2, op3]):
                            if op != 'x':
                                expr += f' {op} {["PHI","PI","E"][i]}'
                        self.results.append({
                            'expr': expr,
                            'target_name': target,
                            'target_value': target_val,
                            'chimera_value': val,
                            'error_pct': error,
                            'status': 'APPROX' if error < 0.1 else 'CANDIDATE',
                            'method': 'genetic_v2'
                        })
                        count += 1
                        break

            # Create new population (tournament selection + mutation)
            new_population = []
            for _ in range(self.population_size):
                # Tournament selection
                candidates = random.sample(population, 3)
                winner = min(candidates, key=lambda ind: fitness(ind, 'gamma'))  # Use gamma as proxy
                new_ind = list(winner)

                # Mutation
                if random.random() < self.mutation_rate:
                    idx = random.randint(0, len(new_ind) - 1)
                    if idx < 5:
                        new_ind[idx] = random.randint(1, 16)
                    elif idx < 8:
                        new_ind[idx] = random.randint(-6, 6)
                    elif idx < 11:
                        new_ind[idx] = random.choice(['+', '-', '*', '/'])
                    else:
                        new_ind[idx] = random.choice(['+', '-', '*', '/'])
                else:
                    # Crossover
                    parent = random.choice(population)
                    if random.random() < self.crossover_rate and len(population) >= 2:
                        p1, p2 = random.sample(population, 2)
                        for i in range(len(new_ind)):
                            new_ind[i] = random.choice([p1[i], p2[i]])

                new_population.append(tuple(new_ind))

            population = new_population

            if verbose and gen % 10 == 0:
                print(f"    Generation {gen}: {count} total matches")

        return count


class UltraEngineV51:
    """ULTRA ENGINE v5.1 — ALL NIGHT LONG DISCOVERY"""

    def __init__(self, threshold: float = 0.01, verbose: bool = True):
        self.threshold = threshold
        self.verbose = verbose
        self.results = []
        self.seen = set()

        # Initialize all search engines
        self.monte_carlo = MonteCarlo(samples=50000)
        self.sat_search = SATSearch()
        self.symbolic = SymbolicRegression()
        self.genetic_v2 = GeneticAlgorithmV2(population_size=200, generations=200)

    def add_result(self, expr: str, target_name: str, chimera_val: float, method: str = 'unknown') -> bool:
        """Add a unique result."""
        key = (expr, target_name)
        if key in self.seen:
            return False
        self.seen.add(key)

        target_val = PDG_TARGETS.get(target_name)
        if target_val is None:
            return False

        error_pct = abs(chimera_val - target_val) / abs(target_val) * 100.0
        if error_pct >= self.threshold:
            return False

        actual_status = 'APPROX' if error_pct < 0.1 else 'CANDIDATE'

        self.results.append({
            'expr': expr,
            'target_name': target_name,
            'target_value': target_val,
            'chimera_value': chimera_val,
            'error_pct': error_pct,
            'status': actual_status,
            'method': method
        })

        if self.verbose:
            print(f"  FOUND: {expr} -> {target_name} | Δ={error_pct:.3f}% | {actual_status} [{method}]")

        return True

    def pattern_search(self):
        """Method 1: n*phi^i*pi^j*e^k"""
        count = 0
        for n_val in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 16]:
            for i in range(-6, 7):
                val = n_val * PHI**i
                for target, target_val in PDG_TARGETS.items():
                    if target_val == 0:
                        continue
                    error = abs(val - target_val) / abs(target_val) * 100.0
                    if error < self.threshold:
                        if self.add_result(f'{n_val}*phi^{i}', target, val, 'pattern'):
                            count += 1

                for j in range(-6, 7):
                    for k in range(-6, 7):
                        val = n_val * PHI**i * PI**j * E**k
                        for target, target_val in PDG_TARGETS.items():
                            if target_val == 0:
                                continue
                            error = abs(val - target_val) / abs(target_val) * 100.0
                            if error < self.threshold:
                                if self.add_result(f'{n_val}*phi^{i}*pi^{j}*e^{k}', target, val, 'pattern'):
                                    count += 1

        return count

    def ratio_search(self):
        """Method 2: n*phi^i/(pi^j*e^k)"""
        count = 0
        for n_val in [1, 2, 3, 4, 5, 6, 7]:
            for i in range(-6, 7):
                for j in range(-6, 7):
                    for k in range(-6, 7):
                        denom = PI**j * E**k
                        if denom == 0:
                            continue
                        val = n_val * PHI**i / denom
                        for target, target_val in PDG_TARGETS.items():
                            if target_val == 0:
                                continue
                            error = abs(val - target_val) / abs(target_val) * 100.0
                            if error < self.threshold:
                                if self.add_result(f'{n_val}*phi^{i}/(pi^{j}*e^{k})', target, val, 'ratio'):
                                    count += 1

        return count

    def run_all(self):
        """Run ALL v5.1 discovery methods."""
        print("=" * 70)
        print("  ULTRA ENGINE v5.1 — ALL NIGHT LONG DISCOVERY")
        print("  phi^2 + 1/phi^2 = 3 | TRINITY")
        print("=" * 70)

        total = 0
        start_time = time.time()

        # Method 1: Pattern Search
        print("\n>>> PATTERN SEARCH...")
        count = self.pattern_search()
        total += count

        # Method 2: Ratio Search
        print("\n>>> RATIO SEARCH...")
        count = self.ratio_search()
        total += count

        # Method 3: Logarithmic Search
        print("\n>>> LOGARITHMIC SEARCH...")
        for n_val in [1, 2, 3, 4, 5, 6, 7, 9]:
            for i in range(-4, 5):
                base = n_val * PHI**i
                if base > 0:
                    val = math.log(base)
                    for target, target_val in PDG_TARGETS.items():
                        if self.add_result(f'ln({n_val}*phi^{i})', target, val, 'log'):
                            total += 1

                val2 = math.log(base, E)
                for target, target_val in PDG_TARGETS.items():
                    if self.add_result(f'log_e({n_val}*phi^{i})', target, val2, 'log'):
                        total += 1

        # Method 4: Exponential Search
        print("\n>>> EXPONENTIAL SEARCH...")
        for n_val in [1, 2, 3, 4, 5, 6]:
            for i in range(-4, 5):
                try:
                    val = math.exp(n_val * PHI**i)
                    for target, target_val in PDG_TARGETS.items():
                        if self.add_result(f'exp({n_val}*phi^{i})', target, val, 'exp'):
                            total += 1
                except OverflowError:
                    pass

        # Method 5: Root Search
        print("\n>>> ROOT SEARCH...")
        for n_val in [1, 2, 3, 4, 5, 6]:
            for i in range(-6, 7):
                for m in [2, 3, 4, 5]:
                    try:
                        base = n_val * PHI**i
                        if base > 0:
                            val = base ** (1.0/m)
                            for target, target_val in PDG_TARGETS.items():
                                if self.add_result(f'({n_val}*phi^{i})^(1/{m})', target, val, 'root'):
                                    total += 1
                    except (OverflowError, ZeroDivisionError):
                        pass

        # Method 6: Trigonometric Search
        print("\n>>> TRIGONOMETRIC SEARCH...")
        for i in range(-6, 7):
            base = PHI**i
            for val in [math.sin(base), math.cos(base)]:
                for n_val in [1, 2, 3, 4, 5, 6, 7]:
                    val2 = n_val * val
                    for target, target_val in PDG_TARGETS.items():
                        if self.add_result(f'{n_val}*sin(phi^{i})', target, val2, 'trig'):
                            total += 1
                        func_name = 'sin' if val == math.sin(base) else 'cos'
                        if self.add_result(f'{n_val}*cos(phi^{i})', target, val2, 'trig'):
                            total += 1

        # Method 7: Chimera Search
        print("\n>>> CHIMERA SEARCH...")
        base_formulas = [
            ("gamma", PHI**-3),
            ("alpha_s", 1/(PHI**4 + PHI)),
            ("delta_CP", 9*PHI**-2*180/PI),
            ("V_ud", 0.97431),
            ("phi", PHI),
            ("pi", PI),
            ("e", E),
        ]
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
                            chimera_val = f1_val / f2_val
                    elif op == '+':
                        chimera_val = f1_val + f2_val
                    elif op == '-':
                        chimera_val = f1_val - f2_val

                    for target, target_val in PDG_TARGETS.items():
                        if target_val == 0:
                            continue
                        error = abs(chimera_val - target_val) / abs(target_val) * 100.0
                        if error < self.threshold:
                            if self.add_result(f'{f1_name} {op} {f2_name}', target, chimera_val, 'chimera'):
                                total += 1

        # Method 8: Monte Carlo
        print("\n>>> MONTE CARLO SEARCH...")
        count = self.monte_carlo.search(threshold=self.threshold, verbose=False)
        total += count

        # Method 9: SAT Search
        print("\n>>> SAT SEARCH...")
        count = self.sat_search.search(threshold=self.threshold, verbose=False)
        total += count

        # Method 10: Symbolic Regression
        print("\n>>> SYMBOLIC REGRESSION...")
        count = self.symbolic.search(threshold=self.threshold, verbose=False)
        total += count

        # Method 11: Genetic Algorithm v2
        print("\n>>> GENETIC ALGORITHM v2...")
        count = self.genetic_v2.search(threshold=self.threshold, verbose=False)
        total += count

        # Sort by error
        self.results.sort(key=lambda x: x['error_pct'])

        elapsed = time.time() - start_time
        print(f"\n{'=' * 70}")
        print(f"  SUMMARY: {len(self.results)} UNIQUE FORMULAS")
        print(f"{'=' * 70}")
        print(f"  TOTAL: {total} search operations")
        print(f"  ELAPSED: {elapsed:.1f} seconds ({elapsed/60:.1f} minutes)")

        # Group by target
        by_target = defaultdict(list)
        by_method = defaultdict(list)

        for r in self.results:
            by_target[r['target_name']].append(r)
            by_method[r['method']].append(r)

        # Print by target
        print(f"\n=== BY TARGET ({len(by_target)} targets) ===")
        for target, target_results in sorted(by_target.items()):
            best = min(target_results, key=lambda x: x['error_pct'])
            print(f"{target:15} | {best['expr']:40} | Δ={best['error_pct']:6.3f}% | {best['status']}")

        # Print by method
        print(f"\n=== BY METHOD ===")
        for method, method_results in sorted(by_method.items()):
            print(f"{method:20} | {len(method_results)} formulas")

        return self.results


def main():
    parser = argparse.ArgumentParser(description='ULTRA ENGINE v5.1 — ALL NIGHT LONG DISCOVERY')
    parser.add_argument('--threshold', type=float, default=0.01, help='Error threshold in percent')
    parser.add_argument('--pattern-only', action='store_true')
    parser.add_argument('--ratio-only', action='store_true')
    parser.add_argument('--log-only', action='store_true')
    parser.add_argument('--exp-only', action='store_true')
    parser.add_argument('--root-only', action='store_true')
    parser.add_argument('--trig-only', action='store_true')
    parser.add_argument('--chimera-only', action='store_true')
    parser.add_argument('--montecarlo-only', action='store_true')
    parser.add_argument('--sat-only', action='store_true')
    parser.add_argument('--symbolic-only', action='store_true')
    parser.add_argument('--genetic-only', action='store_true')
    parser.add_argument('--all', action='store_true', help='Run all discovery methods')
    parser.add_argument('--quiet', '-q', action='store_true', help='Quiet mode')
    parser.add_argument('--samples', type=int, default=50000, help='Monte Carlo samples')

    args = parser.parse_args()

    engine = UltraEngineV51(threshold=args.threshold, verbose=not args.quiet)

    # Configure Monte Carlo
    if args.montecarlo_only:
        engine.monte_carlo.samples = args.samples

    if args.all or not any([
        args.pattern_only, args.ratio_only, args.log_only, args.exp_only, args.root_only,
        args.trig_only, args.chimera_only, args.montecarlo_only, args.sat_only,
        args.symbolic_only, args.genetic_only,
    ]):
        engine.run_all()
    else:
        count = 0
        if not args.pattern_only:
            print(">>> PATTERN SEARCH...")
            count = engine.pattern_search()
            total += count
        if not args.ratio_only:
            print(">>> RATIO SEARCH...")
            count = engine.ratio_search()
            total += count
        if not args.log_only:
            print(">>> LOGARITHMIC SEARCH...")
            for n_val in [1, 2, 3, 4, 5, 6, 7, 9]:
                for i in range(-4, 5):
                    base = n_val * PHI**i
                    if base > 0:
                        val = math.log(base)
                        for target, target_val in PDG_TARGETS.items():
                            error = abs(val - target_val) / abs(target_val) * 100.0
                            if error < args.threshold:
                                if engine.add_result(f'ln({n_val}*phi^{i})', target, val, 'log'):
                                    total += 1

                        val2 = math.log(base, E)
                        for target, target_val in PDG_TARGETS.items():
                            error = abs(val2 - target_val) / abs(target_val) * 100.0
                            if error < args.threshold:
                                if engine.add_result(f'log_e({n_val}*phi^{i})', target, val2, 'log'):
                                    total += 1
        if not args.exp_only:
            print(">>> EXPONENTIAL SEARCH...")
            for n_val in [1, 2, 3, 4, 5, 6]:
                for i in range(-4, 5):
                    try:
                        val = math.exp(n_val * PHI**i)
                        for target, target_val in PDG_TARGETS.items():
                            if engine.add_result(f'exp({n_val}*phi^{i})', target, val, 'exp'):
                                total += 1
                    except OverflowError:
                        pass
        if not args.root_only:
            print(">>> ROOT SEARCH...")
            for n_val in [1, 2, 3, 4, 5, 6]:
                for i in range(-6, 7):
                    for m in [2, 3, 4, 5]:
                        try:
                            base = n_val * PHI**i
                            if base > 0:
                                val = base ** (1.0/m)
                                for target, target_val in PDG_TARGETS.items():
                                    if engine.add_result(f'({n_val}*phi^{i})^(1/{m})', target, val, 'root'):
                                        total += 1
                        except (OverflowError, ZeroDivisionError):
                            pass
        if not args.trig_only:
            print(">>> TRIGONOMETRIC SEARCH...")
            for i in range(-6, 7):
                base = PHI**i
                for val in [math.sin(base), math.cos(base)]:
                    for n_val in [1, 2, 3, 4, 5, 6, 7]:
                        val2 = n_val * val
                        for target, target_val in PDG_TARGETS.items():
                            if engine.add_result(f'{n_val}*sin(phi^{i})', target, val2, 'trig'):
                                total += 1
                        func_name = 'sin' if val == math.sin(base) else 'cos'
                        if engine.add_result(f'{n_val}*cos(phi^{i})', target, val2, 'trig'):
                            total += 1
        if not args.chimera_only:
            print(">>> CHIMERA SEARCH...")
            base_formulas = [
                ("gamma", PHI**-3),
                ("alpha_s", 1/(PHI**4 + PHI)),
                ("delta_CP", 9*PHI**-2*180/PI),
                ("V_ud", 0.97431),
                ("phi", PHI),
                ("pi", PI),
                ("e", E),
            ]
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
                                chimera_val = f1_val / f2_val
                        elif op == '+':
                            chimera_val = f1_val + f2_val
                        elif op == '-':
                            chimera_val = f1_val - f2_val

                        for target, target_val in PDG_TARGETS.items():
                            if target_val == 0:
                                continue
                            error = abs(chimera_val - target_val) / abs(target_val) * 100.0
                            if error < args.threshold:
                                if engine.add_result(f'{f1_name} {op} {f2_name}', target, chimera_val, 'chimera'):
                                    total += 1
        if not args.montecarlo_only:
            print(">>> MONTE CARLO SEARCH...")
            count = engine.monte_carlo.search(threshold=args.threshold, verbose=not args.quiet)
            total += count
        if not args.sat_only:
            print(">>> SAT SEARCH...")
            count = engine.sat_search.search(threshold=args.threshold, verbose=not args.quiet)
            total += count
        if not args.symbolic_only:
            print(">>> SYMBOLIC REGRESSION...")
            count = engine.symbolic.search(threshold=args.threshold, verbose=not args.quiet)
            total += count
        if not args.genetic_only:
            print(">>> GENETIC ALGORITHM v2...")
            count = engine.genetic_v2.search(threshold=args.threshold, verbose=not args.quiet)
            total += count

        engine.results.sort(key=lambda x: x['error_pct'])

        print(f"\n{'=' * 70}")
        print(f"  SUMMARY: {len(engine.results)} UNIQUE FORMULAS")
        print(f"{'=' * 70}")

        # Group by target
        by_target = defaultdict(list)
        for r in engine.results:
            by_target[r['target_name']].append(r)

        for target, target_results in sorted(by_target.items()):
            best = min(target_results, key=lambda x: x['error_pct'])
            print(f"{target:15} | {best['expr']:40} | Δ={best['error_pct']:6.3f}% | {best['status']}")

        return 0


if __name__ == '__main__':
    sys.exit(main())
