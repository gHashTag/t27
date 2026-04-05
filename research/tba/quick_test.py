#!/usr/bin/env python3
"""
Quick test for E₈ TBA module structure.

Validates that all modules are properly structured.
"""

import sys
from pathlib import Path

# Add research/tba to path
sys.path.insert(0, str(Path(__file__).parent))

def test_imports():
    """Test that all modules can be imported."""
    print("Testing module imports...")
    print("-" * 40)

    try:
        from e8_tba_solver import E8TbaSolver, E8YSystemSolution
        print("✓ e8_tba_solver imported")
    except Exception as e:
        print(f"✗ e8_tba_solver failed: {e}")
        return False

    try:
        from e8_mass_optimization import E8MassOptimizer, MassDeformationParams
        print("✓ e8_mass_optimization imported")
    except Exception as e:
        print(f"✗ e8_mass_optimization failed: {e}")
        return False

    try:
        from e8_analyzer import E8ResultsAnalyzer
        print("✓ e8_analyzer imported")
    except Exception as e:
        print(f"✗ e8_analyzer failed: {e}")
        return False

    print("-" * 40)
    return True


def show_module_structure():
    """Display module structure and capabilities."""
    print("\nModule Structure:")
    print("=" * 40)
    print()
    print("e8_tba_solver.py:")
    print("  • E8TbaSolver - Iterative Y-system solver")
    print("  • solve() - Solve Y-system, return c")
    print("  • solve_with_mass_deformation() - Solve with μ params")
    print()
    print("e8_mass_optimization.py:")
    print("  • E8MassOptimizer - Multi-objective optimizer")
    print("  • random_search() - 5000 sample scan")
    print("  • simulated_annealing() - Global optimization")
    print("  • gradient_descent() - Local optimization")
    print("  • find_optimal() - Main entry point")
    print()
    print("e8_analyzer.py:")
    print("  • E8ResultsAnalyzer - Results analysis")
    print("  • find_phi_power_matches() - Find φ-power ratios")
    print("  • find_sm_matches() - Find m_μ/m_e = 206.76")
    print("  • plot_error_landscape() - Visualize errors")
    print("  • plot_parameter_pareto() - Pareto front")
    print()
    print("=" * 40)


def show_optimization_plan():
    """Show the multi-objective optimization plan."""
    print("\nOptimization Plan:")
    print("=" * 40)
    print()
    print("Goal: Find μ₀...μ₇ that match multiple SM ratios")
    print()
    print("Target Ratios:")
    print("  1. m_μ/m_e = 206.76     (muon/electron)")
    print("  2. m_τ/m_e = φ          (tau/electron)")
    print("  3. m_c/m_e = φ²         (charm/electron)")
    print("  4. m_b/m_e = φ³         (bottom/electron)")
    print("  5. m_t/m_e = φ⁴         (top/electron)")
    print("  6. m_s/m_e = φ⁵         (strange/electron)")
    print()
    print("Parameter Space:")
    print("  • 8 parameters: μ₀...μ₇")
    print("  • Range: [0, 10] (adjustable)")
    print("  • Search dimension: 8D")
    print()
    print("Optimization Methods:")
    print("  1. Random Search (5000 samples)")
    print("     - Fast, global coverage")
    print("     - ~5 minutes")
    print()
    print("  2. Simulated Annealing")
    print("     - Global, escapes local minima")
    print("     - ~30 minutes")
    print()
    print("  3. Compare Results")
    print("     - Find best multi-ratio match")
    print()
    print("Success Criteria:")
    print("  • Multiple ratios within 1% tolerance")
    print("  • Minimal weighted error")
    print("  • Reproducible parameters")
    print()
    print("=" * 40)


if __name__ == "__main__":
    print("=" * 40)
    print("E₈ TBA Module - Quick Test")
    print("=" * 40)
    print()

    if test_imports():
        show_module_structure()
        show_optimization_plan()

        print("\n✓ All tests passed!")
        print("\nNext Steps:")
        print("  1. python research/tba/e8_mass_optimization.py")
        print("  2. Review generated JSON results")
        print("  3. python research/tba/e8_analyzer.py")
    else:
        print("\n✗ Import tests failed!")
        sys.exit(1)
