#!/usr/bin/env python3
"""Benchmark φ as fixed-point attractor - verifies convergence rate.

Theorem 3: φ is the unique fixed point of balancing recursion
    f(x) = (x + x⁻¹ + 1) / 2

From any positive starting point x₀ > 0, iteration converges
exponentially to φ with rate λ = (√5 - 1) / 4 ≈ 0.309.

This script verifies the theorem numerically.
"""

import numpy as np

# φ = (1 + √5) / 2
PHI = (1 + np.sqrt(5)) / 2
PHI_SQ = PHI * PHI

# Theoretical convergence rate λ = (√5 - 1) / 4
LAMBDA = (np.sqrt(5) - 1) / 4

def balancing_recursion(x):
    """f(x) = (x + x⁻¹ + 1) / 2"""
    return (x + 1/x + 1) / 2

def iterate_to_convergence(x0, max_iter=50, tol=1e-14):
    """Iterate from x0 to convergence.

    Returns:
        x_final: Final value after convergence
        iterations: Number of iterations taken
        errors: List of errors at each iteration
    """
    x = x0
    errors = []

    for i in range(max_iter):
        x_next = balancing_recursion(x)
        error = abs(x_next - PHI)
        errors.append(error)

        if error < tol:
            return x_next, i + 1, errors

        x = x_next

    return x, max_iter, errors

def print_header():
    """Print the theorem header."""
    print("=" * 65)
    print("Theorem 3 Verification: φ as Universal Fixed-Point Attractor")
    print("=" * 65)
    print()
    print(f"φ           = {PHI:.15f}")
    print(f"φ²         = {PHI_SQ:.15f}")
    print(f"λ (theoretical convergence rate) = {LAMBDA:.15f}")
    print()

def print_convergence_table():
    """Print convergence results from various starting points."""
    test_starts = [0.01, 0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 50.0, 100.0]

    print("Convergence from arbitrary starting points:")
    print(f"{'x₀':>12} | {'iters':>6} | {'final error':>20} | {'λ^iter':>15}")
    print("-" * 60)

    for x0 in test_starts:
        x_final, iters, errors = iterate_to_convergence(x0)
        final_error = errors[-1] if errors else 1.0
        predicted = LAMBDA ** iters

        print(f"{x0:12.3f} | {iters:6d} | {final_error:.2e} | {predicted:.2e}")

    print()

def print_fixed_point_verification():
    """Verify that φ is indeed a fixed point of f."""
    print("Fixed point verification:")
    print("-" * 40)
    f_phi = balancing_recursion(PHI)
    error = abs(f_phi - PHI)

    print(f"f(φ) = {f_phi:.15f}")
    print(f"φ     = {PHI:.15f}")
    print(f"diff   = {error:.2e}")
    print()

    if error < 1e-14:
        print("✓ PASS: φ is a fixed point of f(x)")
    else:
        print("✗ FAIL: φ is not a fixed point")

def print_convergence_rate_analysis():
    """Analyze the convergence rate."""
    print("Convergence rate analysis:")
    print("-" * 40)

    # Test error decay over iterations
    x0 = 0.5
    x_final, iters, errors = iterate_to_convergence(x0)

    print(f"Starting from x₀ = {x0}")
    print(f"Error decay:")
    for i, err in enumerate(errors[:10]):
        predicted = (0.5 - PHI) * (LAMBDA ** (i + 1))
        print(f"  iter {i+1:2d}: |xₙ - φ| = {err:.4e}, predicted: {predicted:.4e}")

    print()

def print_theoretical_explanation():
    """Print the theoretical explanation."""
    print("Theoretical explanation:")
    print("-" * 40)
    print("The balancing recursion f(x) = (x + x⁻¹ + 1)/2 represents a")
    print("fundamental dynamic: allocate a component while maintaining balance")
    print("with its complement.")
    print()
    print(f"Convergence rate λ = {LAMBDA:.15f} means error decays as:")
    print(f"  |xₙ - φ| ≈ λⁿ × |x₀ - φ|")
    print()
    print("For λ ≈ 0.309, the error roughly decays by:")
    print("  ~70% every iteration")
    print()

def main():
    """Main benchmark execution."""
    print_header()
    print_convergence_table()
    print_fixed_point_verification()
    print_convergence_rate_analysis()
    print_theoretical_explanation()

    print("=" * 65)
    print("Theorem 3: Zero-Parameter Generative Mechanism")
    print("=" * 65)
    print()
    print("Key distinction from 'fitting':")
    print("  • No free parameters were tuned")
    print("  • The recursion f(x) is defined independently of GF formats")
    print("  • φ emerges as the inevitable outcome of the dynamics")
    print("  • This is a first-principles mechanism, not a narrative fit")
    print()

if __name__ == "__main__":
    main()
