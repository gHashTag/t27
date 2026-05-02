#!/usr/bin/env python3
"""
E₈ Y-System Multi-Objective Mass Optimization

Finds optimal 8-parameter mass deformations that match multiple SM ratios simultaneously.

Targets:
- m_μ/m_e = 206.76 (muon/electron)
- m_τ/m_e = φ (tau/electron)
- m_c/m_e = φ² (charm/electron)
- m_b/m_e = φ³ (bottom/electron)
- m_t/m_e = φ⁴ (top/electron)

Optimization: Minimize weighted sum of squared errors across all targets.

Status: Week 4 Development
Date: 2026-04-06
"""

from __future__ import annotations
import numpy as np
from dataclasses import dataclass, asdict
from typing import List, Tuple, Callable, Dict, Any
import json
from pathlib import Path
import random
from mpmath import mp, mpf

from e8_tba_solver import E8TbaSolver, E8YSystemSolution

mp.dps = 50


@dataclass
class StandardModelRatios:
    """Target Standard Model mass ratios."""
    # Experimental values (PDG 2024)
    m_mu_over_me: float = 206.76       # m_μ/m_e
    m_tau_over_me: float = 1.61803399  # φ ≈ 1.618
    m_c_over_me: float = 2.61803399   # φ² ≈ 2.618
    m_b_over_me: float = 4.23606798   # φ³ ≈ 4.236
    m_t_over_me: float = 6.85410197   # φ⁴ ≈ 6.854
    m_s_over_me: float = 11.0901699  # φ⁵ ≈ 11.090

    # Relative uncertainties (PDG)
    m_mu_err: float = 0.000003      # ±3×10⁻⁶
    m_tau_err: float = 0.001          # ±0.1%
    m_c_err: float = 0.1               # ±10% (theoretical)


@dataclass
class MassDeformationParams:
    """8-parameter mass deformation for E₈ Y-system."""
    # Parameters μ₀...μ₇ map to E₈ simple roots
    mu: np.ndarray  # shape: (8,)

    def to_dict(self) -> Dict[str, float]:
        return {f"mu_{i}": float(self.mu[i]) for i in range(8)}


@dataclass
class OptimizationResult:
    """Result of mass deformation optimization."""
    params: MassDeformationParams
    predicted_ratios: np.ndarray  # shape: (6,)
    target_ratios: np.ndarray
    errors: np.ndarray  # shape: (6,)
    total_error: float
    weighted_error: float
    n_matches: int  # Number of ratios within tolerance


class E8MassOptimizer:
    """
    Multi-objective optimizer for E₈ Y-system mass deformation.

    Searches 8-dimensional parameter space for μ that best matches
    Standard Model mass ratios.
    """

    def __init__(
        self,
        solver: E8TbaSolver,
        targets: StandardModelRatios = None,
        tolerance: float = 0.01,  # 1% tolerance for "match"
        weights: np.ndarray = None
    ):
        self.solver = solver
        self.targets = targets or StandardModelRatios()
        self.tolerance = tolerance

        # Weights for each ratio (higher = more important)
        if weights is None:
            self.weights = np.array([
                1.0,   # m_μ/m_e
                0.5,   # m_τ/m_e (φ is easier)
                0.5,   # m_c/m_e
                0.5,   # m_b/m_e
                0.5,   # m_t/m_e
                0.5    # m_s/m_e
            ])
        else:
            self.weights = weights

        # Target ratios as array
        self.target_array = np.array([
            self.targets.m_mu_over_me,
            self.targets.m_tau_over_me,
            self.targets.m_c_over_me,
            self.targets.m_b_over_me,
            self.targets.m_t_over_me,
            self.targets.m_s_over_me,
        ])

    def evaluate(self, mu: np.ndarray) -> OptimizationResult:
        """
        Evaluate mass deformation parameters.

        Solves Y-system with deformation μ and computes errors.
        """
        # Solve Y-system with mass deformation
        solution, predicted = self.solver.solve_with_mass_deformation(mu)

        # Compute relative errors
        errors = np.abs(predicted - self.target_array) / self.target_array

        # Total weighted error
        weighted_error = np.sum(self.weights * errors ** 2)
        total_error = np.sum(errors ** 2)

        # Count matches within tolerance
        n_matches = np.sum(errors < self.tolerance)

        return OptimizationResult(
            params=MassDeformationParams(mu=mu),
            predicted_ratios=predicted,
            target_ratios=self.target_array,
            errors=errors,
            total_error=float(total_error),
            weighted_error=float(weighted_error),
            n_matches=int(n_matches)
        )

    def random_search(
        self,
        n_samples: int = 5000,
        mu_range: Tuple[float, float] = (0.0, 10.0),
        seed: int = None
    ) -> Tuple[OptimizationResult, List[OptimizationResult]]:
        """
        Random search over 8-dimensional parameter space.

        Returns: (best_result, all_results)
        """
        if seed is not None:
            random.seed(seed)
            np.random.seed(seed)

        all_results = []
        best_error = float('inf')
        best_result = None

        for i in range(n_samples):
            # Random 8 parameters
            mu = np.random.uniform(mu_range[0], mu_range[1], 8)

            # Evaluate
            result = self.evaluate(mu)
            all_results.append(result)

            # Track best
            if result.weighted_error < best_error:
                best_error = result.weighted_error
                best_result = result

            # Progress update
            if i % 500 == 0:
                print(f"  Sample {i}/{n_samples}: best_error = {best_error:.6e}, matches = {best_result.n_matches if best_result else 0}")

        return best_result, all_results

    def gradient_descent(
        self,
        initial_mu: np.ndarray = None,
        learning_rate: float = 0.1,
        max_iter: int = 1000,
        tolerance: float = 1e-10
    ) -> OptimizationResult:
        """
        Gradient descent optimization (simplified).

        Uses finite differences for gradient approximation.
        """
        if initial_mu is None:
            mu = np.random.uniform(0, 5, 8)
        else:
            mu = initial_mu.copy()

        best_mu = mu.copy()
        best_result = self.evaluate(mu)

        for i in range(max_iter):
            # Approximate gradient using finite differences
            gradient = np.zeros(8)
            h = 1e-6

            for j in range(8):
                mu_plus = mu.copy()
                mu_plus[j] += h
                result_plus = self.evaluate(mu_plus)

                mu_minus = mu.copy()
                mu_minus[j] -= h
                result_minus = self.evaluate(mu_minus)

                gradient[j] = (result_plus.weighted_error - result_minus.weighted_error) / (2 * h)

            # Update parameters
            mu_new = mu - learning_rate * gradient

            # Ensure parameters are positive
            mu_new = np.maximum(mu_new, 0)

            # Evaluate new point
            result_new = self.evaluate(mu_new)

            # Track best
            if result_new.weighted_error < best_result.weighted_error:
                best_mu = mu_new.copy()
                best_result = result_new

            # Check convergence
            if np.max(np.abs(mu_new - mu)) < tolerance:
                break

            mu = mu_new

            if i % 100 == 0:
                print(f"  Iter {i}: error = {best_result.weighted_error:.6e}, matches = {best_result.n_matches}")

        best_result.params = MassDeformationParams(mu=best_mu)
        return best_result

    def simulated_annealing(
        self,
        initial_temp: float = 1.0,
        cooling_rate: float = 0.995,
        min_temp: float = 1e-6,
        n_steps_per_temp: int = 100,
        seed: int = None
    ) -> OptimizationResult:
        """
        Simulated annealing for global search.

        Helps escape local minima in 8D space.
        """
        if seed is not None:
            random.seed(seed)
            np.random.seed(seed)

        # Random initial parameters
        mu = np.random.uniform(0, 10, 8)
        current_result = self.evaluate(mu)
        best_result = current_result

        temp = initial_temp

        while temp > min_temp:
            for _ in range(n_steps_per_temp):
                # Propose new parameters (Gaussian perturbation)
                mu_new = mu + np.random.normal(0, temp, 8)
                mu_new = np.maximum(mu_new, 0)  # Ensure non-negative

                # Evaluate
                new_result = self.evaluate(mu_new)

                # Metropolis criterion
                delta = new_result.weighted_error - current_result.weighted_error
                if delta < 0 or random.random() < np.exp(-delta / temp):
                    mu = mu_new
                    current_result = new_result

                    # Update best
                    if current_result.weighted_error < best_result.weighted_error:
                        best_result = current_result

            temp *= cooling_rate

        best_result.params = MassDeformationParams(mu=mu)
        return best_result

    def find_optimal(
        self,
        method: str = "annealing",
        **kwargs
    ) -> OptimizationResult:
        """
        Find optimal mass deformation parameters.

        Methods:
        - "random": Random search (fast, global)
        - "descent": Gradient descent (fast, local)
        - "annealing": Simulated annealing (slow, global)
        """
        print(f"Finding optimal parameters using {method}...")
        print(f"Target ratios: {self.target_array}")
        print(f"Tolerance: {self.tolerance * 100}%")
        print("-" * 60)

        if method == "random":
            result, _ = self.random_search(**kwargs)
        elif method == "descent":
            result = self.gradient_descent(**kwargs)
        elif method == "annealing":
            result = self.simulated_annealing(**kwargs)
        else:
            raise ValueError(f"Unknown method: {method}")

        print("-" * 60)
        print(f"Optimization complete!")
        print(f"Best weighted error: {result.weighted_error:.6e}")
        print(f"Matches within tolerance: {result.n_matches}/6")

        print("\nOptimal parameters μ:")
        for i in range(8):
            print(f"  μ_{i} = {result.params.mu[i]:.8f}")

        print("\nPredicted vs Target ratios:")
        names = ["m_μ/m_e", "m_τ/m_e", "m_c/m_e", "m_b/m_e", "m_t/m_e", "m_s/m_e"]
        for i, name in enumerate(names):
            pred = result.predicted_ratios[i]
            target = result.target_ratios[i]
            err = result.errors[i] * 100
            status = "✓" if err < self.tolerance * 100 else "✗"
            print(f"  {status} {name}: {pred:.6f} vs {target:.6f} ({err:+.2f}%)")

        return result

    def save_results(self, result: OptimizationResult, path: Path) -> None:
        """Save optimization results to JSON."""
        output = {
            "best_result": {
                "params": result.params.to_dict(),
                "weighted_error": result.weighted_error,
                "total_error": result.total_error,
                "n_matches": result.n_matches,
                "predicted_ratios": [float(x) for x in result.predicted_ratios],
                "target_ratios": [float(x) for x in result.target_ratios],
                "errors": [float(x) for x in result.errors],
            },
            "targets": asdict(self.targets),
        }

        with open(path, "w") as f:
            json.dump(output, f, indent=2, default=str)

        print(f"\nResults saved to: {path}")


def run_multi_objective_search():
    """
    Run multi-objective search for SM mass ratios.

    Target: Find 8 parameters that match multiple ratios simultaneously.
    """
    # Initialize solver
    solver = E8TbaSolver(tolerance=1e-25, max_iter=3000)

    # Initialize optimizer
    optimizer = E8MassOptimizer(
        solver=solver,
        tolerance=0.01  # 1% tolerance
    )

    # Method 1: Random search (5000 samples)
    print("=" * 60)
    print("Method 1: Random Search (5000 samples)")
    print("=" * 60)
    best_random = optimizer.find_optimal(method="random", n_samples=5000, seed=42)
    optimizer.save_results(best_random, Path("research/tba/e8_mass_random_results.json"))

    # Method 2: Simulated annealing (global optimization)
    print("\n" + "=" * 60)
    print("Method 2: Simulated Annealing")
    print("=" * 60)
    best_annealing = optimizer.find_optimal(
        method="annealing",
        initial_temp=1.0,
        cooling_rate=0.995,
        min_temp=1e-6,
        n_steps_per_temp=50,
        seed=42
    )
    optimizer.save_results(best_annealing, Path("research/tba/e8_mass_annealing_results.json"))

    # Compare results
    print("\n" + "=" * 60)
    print("Comparison of Methods")
    print("=" * 60)
    print(f"Random search: {best_random.n_matches}/6 matches, error = {best_random.weighted_error:.6e}")
    print(f"Simulated annealing: {best_annealing.n_matches}/6 matches, error = {best_annealing.weighted_error:.6e}")

    # Use best result
    best = best_annealing if best_annealing.n_matches > best_random.n_matches else best_random

    print(f"\nBest overall: {best.n_matches}/6 matches")
    print(f"Parameters: {best.params.mu}")


if __name__ == "__main__":
    print("=" * 60)
    print("E₈ Y-System Multi-Objective Mass Optimization")
    print("=" * 60)
    print("\nObjective: Find 8 parameters μ that match multiple SM mass ratios")
    print("\nTargets:")
    print("  m_μ/m_e = 206.76 (muon/electron)")
    print("  m_τ/m_e = φ ≈ 1.618 (tau/electron)")
    print("  m_c/m_e = φ² ≈ 2.618 (charm/electron)")
    print("  m_b/m_e = φ³ ≈ 4.236 (bottom/electron)")
    print("  m_t/m_e = φ⁴ ≈ 6.854 (top/electron)")
    print()

    run_multi_objective_search()
