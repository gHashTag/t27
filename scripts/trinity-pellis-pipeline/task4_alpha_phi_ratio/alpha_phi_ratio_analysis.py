#!/usr/bin/env python3
"""α_φ/α Ratio Analysis (Priority 4).

Compute ε = (α_φ/α - 10φ)/(10φ):

  alpha_phi = φ⁻³/2 ≈ 0.118034
  alpha_meas = 1/137.035999084 ≈ 0.007297
  ratio_expected = 10φ ≈ 16.18034

  ε = (alpha_phi/alpha_meas - ratio_expected) / ratio_expected
  ε ≈ -0.047% (from user)

CODATA uncertainty: δα/α = 1.5 × 10⁻¹⁰

Result: Compare ε with CODATA uncertainty → conjecture "not excluded" if ε < δα/α
"""

from __future__ import annotations

import sys
from pathlib import Path

# Add parent directory to path for core module
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    from mpmath import mp, mpf, nstr, sqrt
except ImportError:
    print("mpmath is required: pip install mpmath")
    sys.exit(1)

from core.formula_evaluator import FormulaEvaluator


def main() -> int:
    """Main execution."""
    mp.dps = 100

    evaluator = FormulaEvaluator(dps=100)
    phi = evaluator.phi

    print("=== α_φ/α Ratio Analysis ===\n")

    # Define values
    # α_φ = φ⁻³ / 2
    alpha_phi = (phi ** (-3)) / mpf(2)
    print(f"α_φ = φ⁻³ / 2")
    print(f"  = {evaluator.format_50_digit(alpha_phi)}")

    # α_measured (CODATA 2022)
    alpha_measured = mpf(1) / mpf("137.035999084")
    print(f"\nα (CODATA 2022) = 1 / 137.035999084")
    print(f"  = {evaluator.format_50_digit(alpha_measured)}")

    # CODATA 2024 update (more precise)
    alpha_codata_2024 = mpf(1) / mpf("137.035999084")
    alpha_uncertainty_2024 = mpf("1.5e-10")  # δα/α = 1.5×10⁻¹⁰
    alpha_uncertainty_float = float(alpha_uncertainty_2024)  # For formatting

    print(f"α (CODATA 2024) = {evaluator.format_50_digit(alpha_codata_2024)}")
    print(f"Uncertainty (δα/α) = {nstr(alpha_uncertainty_2024, 20)}")

    # Ratio α_φ / α
    ratio_computed = alpha_phi / alpha_codata_2024
    print(f"\nComputed ratio α_φ / α = {evaluator.format_50_digit(ratio_computed)}")

    # Expected ratio: 10φ
    ratio_expected = mpf(10) * phi
    print(f"\nExpected ratio 10φ = {evaluator.format_50_digit(ratio_expected)}")

    # Compute ε = (α_φ/α - 10φ) / (10φ)
    epsilon = (ratio_computed - ratio_expected) / ratio_expected
    epsilon_float = float(epsilon)

    print(f"\n=== Deviation Analysis ===")
    print(f"ε = (α_φ/α - 10φ) / (10φ)")
    print(f"ε = {evaluator.format_50_digit(epsilon)}")
    print(f"ε = {epsilon_float * 100:.15f}%")

    # Compare with CODATA uncertainty
    print(f"\n=== Comparison with CODATA Uncertainty ===")
    print(f"|ε| = {abs(epsilon_float) * 100:.15f}%")
    print(f"CODATA δα/α = {alpha_uncertainty_float * 100:.15f}%")
    print(f"|ε| / (δα/α) = {abs(epsilon_float) / alpha_uncertainty_float:.2e}")

    # Determine if conjecture is excluded
    is_excluded = abs(epsilon_float) > alpha_uncertainty_float
    print(f"\nConjecture status: {'EXCLUDED' if is_excluded else 'NOT EXCLUDED'}")
    print(f"  Criteria: |ε| < δα/α for non-exclusion")
    print(f"  |ε| < δα/α: {abs(epsilon_float) < alpha_uncertainty_float}")

    # Additional calculations
    print(f"\n=== Additional Information ===")

    # α_φ in SI units (for reference)
    alpha_phi_si = float(alpha_phi)
    print(f"α_φ = {alpha_phi_si:.15f}")

    # α_φ / α in percent terms
    ratio_percent = float(ratio_computed) * 100
    expected_percent = float(ratio_expected) * 100
    print(f"\nα_φ/α = {ratio_percent:.15f}%")
    print(f"10φ = {expected_percent:.15f}%")

    # Delta between computed and expected
    delta_ratio_pct = abs((ratio_computed - ratio_expected) / ratio_expected) * 100
    delta_ratio_float = float(delta_ratio_pct)
    print(f"Δ% = |α_φ/α - 10φ| / (10φ) × 100% = {delta_ratio_float:.15f}%")

    # Prepare output
    output_dir = Path(__file__).parent.parent / "output"
    output_dir.mkdir(exist_ok=True)

    import json

    result = {
        "alpha_phi": str(evaluator.format_50_digit(alpha_phi)),
        "alpha_formula": "phi^(-3) / 2",
        "alpha_codata_2024": str(evaluator.format_50_digit(alpha_codata_2024)),
        "alpha_uncertainty": str(alpha_uncertainty_2024),
        "alpha_uncertainty_percent": float(alpha_uncertainty_float * 100),
        "computed_ratio": str(evaluator.format_50_digit(ratio_computed)),
        "expected_ratio_10phi": str(evaluator.format_50_digit(ratio_expected)),
        "epsilon": str(evaluator.format_50_digit(epsilon)),
        "epsilon_percent": float(epsilon_float * 100),
        "delta_ratio_percent": delta_ratio_float,
        "is_excluded": is_excluded,
        "exclusion_criteria": "|epsilon| < alpha_uncertainty",
        "interpretation": (
            f"The deviation ε = {epsilon_float * 100:.15f}% is "
            f"{'GREATER than' if is_excluded else 'LESS than'} "
            f"the CODATA uncertainty δα/α = {alpha_uncertainty_float * 100:.15f}%. "
            f"Therefore, the α_φ conjecture is {'EXCLUDED' if is_excluded else 'NOT EXCLUDED'} "
            f"by current experimental precision."
        )
    }

    # Write JSON output
    json_path = output_dir / "alpha_phi_ratio.json"
    with open(json_path, 'w') as f:
        json.dump(result, f, indent=2, default=str)
    print(f"\nSaved: {json_path}")

    # Write Markdown table for paper
    md_path = output_dir / "alpha_phi_ratio_table.md"
    with open(md_path, 'w') as f:
        f.write("# α_φ/α Ratio Analysis\n\n")

        f.write("## Methodology\n\n")
        f.write("We compare the Trinity prediction for the fine-structure constant\n")
        f.write("with the CODATA 2024 measured value:\n\n")

        f.write("### Trinity Prediction\n\n")
        f.write("$$\\alpha_\\phi = \\frac{\\phi^{-3}}{2}$$\n\n")
        f.write(f"$$\\alpha_\\phi = {evaluator.format_50_digit(alpha_phi)}$$\n\n")

        f.write("### CODATA 2024 Measurement\n\n")
        f.write("$$\\alpha_{\\text{CODATA 2024}} = \\frac{1}{137.035999084}$$\n\n")
        f.write(f"$$\\alpha_{{\\text{{CODATA 2024}}}} = {evaluator.format_50_digit(alpha_codata_2024)}$$\\n")
        f.write(f"**Uncertainty:** $\\frac{{\\delta\\alpha}}{{\\alpha}} = 1.5 \\times 10^{{-10}}$$\\n")

        f.write("### Ratio Comparison\n\n")
        f.write("$$\\frac{\\alpha_\\phi}{\\alpha} = {evaluator.format_50_digit(ratio_computed)}$$\n\n")
        f.write("### Expected Ratio\n\n")
        f.write("$$10\\phi = {evaluator.format_50_digit(ratio_expected)}$$\n\n")

        f.write("## Deviation Analysis\n\n")
        f.write("$$\\epsilon = \\frac{{\\alpha_\\phi/\\alpha - 10\\phi}}{{10\\phi}}$$\n\n")
        f.write(f"$$\\epsilon = {evaluator.format_50_digit(epsilon)}$$\\n")
        f.write(f"$$\\epsilon = {epsilon_float * 100:.15f}\\%$$\\n")

        f.write("## Comparison with CODATA Uncertainty\\n")
        f.write("| Quantity | Value |\\n")
        f.write("|----------|-------|\\n")
        f.write(f"| $|\\epsilon|$ | {abs(epsilon_float) * 100:.15f}% |\\n")
        f.write(f"| $\\delta\\alpha/\\alpha$ (CODATA 2024) | {alpha_uncertainty_float * 100:.15f}% |\\n")
        f.write(f"| Ratio $|\\epsilon| / (\\delta\\alpha/\\alpha)$ | {abs(epsilon_float) / alpha_uncertainty_float:.2e} |\\n")

        f.write("## Conjecture Status\\n")
        f.write(f"**Status:** {'~~EXCLUDED~~' if is_excluded else '**NOT EXCLUDED**'}\\n")
        f.write(f"**Criteria:** $|\\epsilon| < \\delta\\alpha/\\alpha$\\n")
        f.write(f"**Result:** $|{epsilon_float * 100:.15f}\\%| {'>' if is_excluded else '<'} {alpha_uncertainty_float * 100:.15f}\\%$\\n")

        f.write("## Interpretation\n\n")
        f.write(result["interpretation"])
        f.write("\n\n")

        if not is_excluded:
            f.write("### Conclusion for Paper\n\n")
            f.write("The α_φ conjecture is **not excluded** by current experimental data.\n")
            f.write("The deviation from the expected $10\\phi$ ratio is within the\n")
            f.write("CODATA 2024 uncertainty bounds. This warrants further investigation\n")
            f.write("of the Trinity framework's predictive power for fundamental constants.\n")

    print(f"Saved: {md_path}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
