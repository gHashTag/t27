#!/usr/bin/env python3
"""CKM Unitarity Check (Priority 6).

Compute: |V_ud|² + |V_us|² + |V_ub|² + |V_cb|² = 1

  V_ud = 7φ⁻⁵π³e⁻³ (from FORMULA_TABLE)
  V_us = 3γ/π (from sacred catalog)
  V_cb = γ³π (from sacred catalog)

Unitarity sum: Σ|V_ij|² should equal 1

Output: Deviation percentage from unity.
"""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Dict, List, Tuple

# Add parent directory to path for core module
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    from mpmath import mp, mpf, nstr, sqrt, pi, e
except ImportError:
    print("mpmath is required: pip install mpmath")
    sys.exit(1)

from core.formula_evaluator import FormulaEvaluator, TrinityMonomial


# CKM matrix elements with Trinity formulas
# Based on FORMULA_TABLE and sacred_formula_catalog.json
CKM_ELEMENTS = {
    "V_ud": {
        "name": "V_ud",
        "description": "CKM element u→d",
        "trinity_formula": TrinityMonomial(n=7, k=-5, m=3, p=-3),
        "formula_str": "7φ⁻⁵π³e⁻³",
        "experimental_value": mpf("0.97449"),  # PDG 2024
    },
    "V_us": {
        "name": "V_us",
        "description": "CKM element u→s",
        "trinity_formula": TrinityMonomial(n=3, k=0, m=-1, p=0),
        "formula_str": "3/π",
        "experimental_value": mpf("0.22431"),  # PDG 2024
    },
    "V_ub": {
        "name": "V_ub",
        "description": "CKM element u→b",
        "trinity_formula": TrinityMonomial(n=1, k=0, m=0, p=0),  # No Trinity formula - use experimental
        "formula_str": "Experimental (no Trinity formula)",
        "experimental_value": mpf("0.00369"),  # PDG 2024 upper limit
    },
    "V_cb": {
        "name": "V_cb",
        "description": "CKM element c→b",
        "trinity_formula": TrinityMonomial(n=1, k=0, m=1, p=0),  # γ³π requires gamma
        "gamma_formula": TrinityMonomial(n=1, k=0, m=1, p=0, q=3),
        "formula_str": "γ³π",
        "experimental_value": mpf("0.0411"),  # PDG 2024
    },
}


def compute_ckm_unitarity(elements: Dict, evaluator: FormulaEvaluator) -> Dict:
    """Compute CKM unitarity: Σ|V_ij|² = 1."""
    results = {}

    # Compute each element value
    for key, elem in elements.items():
        trinity_formula = elem.get("trinity_formula")
        gamma_formula = elem.get("gamma_formula")

        if gamma_formula:
            # Use gamma formula
            value = evaluator.compute_monial(gamma_formula)
            formula_str = f"{elem['formula_str']} (with γ={evaluator.format_50_digit(evaluator.gamma)})"
        elif trinity_formula:
            value = evaluator.compute_monial(trinity_formula)
            formula_str = elem['formula_str']
        else:
            value = elem['experimental_value']
            formula_str = elem['formula_str']

        results[key] = {
            "name": elem['name'],
            "description": elem['description'],
            "formula": formula_str,
            "computed_value": str(evaluator.format_50_digit(value)),
            "experimental_value": str(evaluator.format_50_digit(elem['experimental_value'])),
            "delta_pct": float(abs((value - elem['experimental_value']) / elem['experimental_value']) * 100),
            "squared": float(value ** 2),
        }

    # Compute unitarity sum
    unitarity_sum = sum(r['squared'] for r in results.values())

    # Deviation from unity
    deviation = unitarity_sum - 1
    deviation_pct = abs(deviation) * 100

    results['unitarity'] = {
        "sum": float(unitarity_sum),
        "sum_formatted": str(evaluator.format_50_digit(unitarity_sum)),
        "deviation": float(deviation),
        "deviation_formatted": str(evaluator.format_50_digit(deviation)),
        "deviation_pct": deviation_pct,
        "is_unitary": abs(deviation) < mpf("1e-10"),
    }

    return results


def main() -> int:
    """Main execution."""
    mp.dps = 100

    evaluator = FormulaEvaluator(dps=100)

    print("=== CKM Unitarity Check ===\n")
    print("CKM Matrix (first row):\n")
    print("  V_ud  V_us  V_ub")
    print("  |V_ud|² + |V_us|² + |V_ub|² ≈ 1\n\n")

    # Compute unitarity
    results = compute_ckm_unitarity(CKM_ELEMENTS, evaluator)

    # Print individual elements
    for key in ["V_ud", "V_us", "V_ub", "V_cb"]:
        elem = results[key]
        print(f"{elem['name']} ({elem['description']}):")
        print(f"  Formula: {elem['formula']}")
        print(f"  Computed: {elem['computed_value']}")
        print(f"  Experimental: {elem['experimental_value']}")
        print(f"  Δ%: {elem['delta_pct']:.15f}%")
        print(f"  |V|²: {evaluator.format_50_digit(mp.mpf(str(elem['squared'])))}")
        print()

    # Print unitarity results
    unitarity = results['unitarity']
    print("=== Unitarity Results ===")
    print(f"Σ|V|² = {unitarity['sum_formatted']}")
    print(f"Deviation from unity: {unitarity['deviation_formatted']}")
    print(f"Deviation %: {unitarity['deviation_pct']:.15f}%")
    print(f"Is unitary: {unitarity['is_unitary']}")

    # Prepare output
    output_dir = Path(__file__).parent.parent / "output"
    output_dir.mkdir(exist_ok=True)

    import json

    result = {
        "ckm_elements": {
            key: {
                "name": results[key]['name'],
                "description": results[key]['description'],
                "formula": results[key]['formula'],
                "computed_value": results[key]['computed_value'],
                "experimental_value": results[key]['experimental_value'],
                "delta_pct": results[key]['delta_pct'],
                "squared": results[key]['squared'],
            }
            for key in ["V_ud", "V_us", "V_ub", "V_cb"]
        },
        "unitarity": {
            "sum": unitarity['sum_formatted'],
            "deviation": unitarity['deviation_formatted'],
            "deviation_pct": unitarity['deviation_pct'],
            "is_unitary": unitarity['is_unitary'],
        },
        "interpretation": (
            f"The unitarity deviation is {unitarity['deviation_pct']:.15f}%. "
            f"{'Unitarity holds within experimental precision' if abs(unitarity['deviation']) < mpf('0.01') else 'Significant deviation from unity observed'}."
        )
    }

    # Write JSON output
    json_path = output_dir / "ckm_unitarity.json"
    with open(json_path, 'w') as f:
        json.dump(result, f, indent=2, default=str)
    print(f"\nSaved: {json_path}")

    # Write Markdown table for paper
    md_path = output_dir / "ckm_unitarity_table.md"
    with open(md_path, 'w') as f:
        f.write("# CKM Unitarity Check\n\n")

        f.write("## Methodology\n\n")
        f.write("The Cabibbo-Kobayashi-Maskawa (CKM) matrix must satisfy unitarity:\n\n")
        f.write("$$\\sum_{{j=1}}^{{3}} |V_{{uj}}|^2 = 1$$\n\n")

        f.write("### CKM Matrix First Row\n\n")
        f.write("|  | $V_{{ud}}$ | $V_{{us}}$ | $V_{{ub}}$ |\n")
        f.write("|--|---------------|---------------|--------------|\n")
        f.write("| Experimental | 0.97449 | 0.22431 | 0.00369 |\n")

        f.write("\n## Trinity Predictions\n\n")
        f.write("| Element | Trinity Formula | Computed Value | Experimental | Δ% |\n")
        f.write("|---------|-----------------|----------------|-------------|-----|\n")

        for key in ["V_ud", "V_us", "V_cb"]:
            elem = results[key]
            f.write(f'| $V_{{{elem["name"].replace("_", "").replace("V", "").replace("ub", "u{{b}}").replace("us", "u{{s}}").replace("ud", "u{{d}}")}}}$ | ${elem["formula"]}$ | `{elem["computed_value"]}` | `{elem["experimental_value"]}` | {elem["delta_pct"]:.15f}% |\n')

        f.write("\n| Element | $|V|^2$ |\n")
        f.write("|---------|----------|\n")
        for key in ["V_ud", "V_us", "V_ub", "V_cb"]:
            elem = results[key]
            f.write(f'| $V_{{{elem["name"].replace("_", "")}}}$ | `{evaluator.format_50_digit(mp.mpf(str(elem["squared"])))}` |\n')

        f.write("\n## Unitarity Check\n\n")
        f.write(f"$$\\sum |V_{{ij}}|^2 = {unitarity['sum_formatted']}$$\n\n")

        f.write(f"$$\\Delta = |\\sum |V|^2 - 1| = {unitarity['deviation_formatted']}$$\n\n")
        f.write(f"$$\\Delta_{{\\%}} = {unitarity['deviation_pct']:.15f}\\%$$\n\n")

        f.write("## Interpretation\n\n")
        f.write(result["interpretation"])
        f.write("\n\n")

        if unitarity['is_unitary']:
            f.write("### Conclusion\n\n")
            f.write("The Trinity-predicted CKM elements satisfy unitarity\n")
            f.write("within the expected experimental uncertainty bounds.\n")
            f.write("This supports the consistency of the Trinity framework\n")
            f.write("with the Standard Model.\n")
        else:
            f.write("### Conclusion\n\n")
            f.write("The unitarity deviation exceeds experimental precision.\n")
            f.write("This may indicate limitations in the current Trinity\n")
            f.write("formulation for these matrix elements.\n")

    print(f"Saved: {md_path}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
