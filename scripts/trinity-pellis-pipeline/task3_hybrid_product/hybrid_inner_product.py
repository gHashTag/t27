#!/usr/bin/env python3
"""Hybrid Inner Product Analysis (Priority 3).

For 4 Pellis constants {α⁻¹, μ, Ω_Λ, α_s}:

  ⟨M_Trinity, P_Pellis⟩ = (M·P) / (|M||P|)

  Trinity monomial: M = n·φ^k·π^m·e^p
  Pellis polynomial: P(φ) = Σ c_k·φ^{-k}

Output: 4×2 table for section 10.2
"""

from __future__ import annotations

import sys
from pathlib import Path
from typing import List, Tuple, Dict

# Add parent directory to path for core module
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    from mpmath import mp, mpf, nstr, sqrt, pi, e, exp
except ImportError:
    print("mpmath is required: pip install mpmath")
    sys.exit(1)

from core.formula_evaluator import FormulaEvaluator, TrinityMonomial


# 4 Pellis constants from FORMULA_TABLE
PELLIS_CONSTANTS = {
    "alpha_inv": {
        "name": r"\alpha^{-1}",
        "value": mpf("360") / mpf("137.035999084"),  # Normalized value
        "description": "Inverse fine-structure constant",
        "trinity_formula": TrinityMonomial(n=360, k=-2, m=3, p=0),  # 360φ⁻²π³
    },
    "mu_ratio": {
        "name": r"\mu_m / m_e",
        "value": mpf("206.7682830"),  # μ/m_e ratio
        "description": "Muon-electron mass ratio",
        "trinity_formula": TrinityMonomial(n=17, k=0, m=2, p=5),  # 17π²e⁵
    },
    "omega_lambda": {
        "name": r"\Omega_\Lambda",
        "value": mpf("0.685"),
        "description": "Dark energy density parameter",
        "trinity_formula": TrinityMonomial(n=1, k=0, m=0, p=0),  # Simple approximation
    },
    "alpha_s": {
        "name": r"\alpha_s",
        "value": mpf("0.1181"),  # Strong coupling constant at Z pole
        "description": "Strong interaction coupling",
        "trinity_formula": TrinityMonomial(n=4, k=-2, m=-2, p=2),  # 4φ⁻²π⁻²e²
    },
}


# Pellis polynomial approximations
# P(φ) = Σ c_k·φ^{-k}
PELLIS_POLYNOMIALS = {
    "alpha_inv_pellis": {
        "coefficients": [(360, -2), (-2, -3), (1/3, -5)],
        "name": "Pellis α⁻¹",
        "formula": "360φ⁻² - 2φ⁻³ + (3φ)⁻⁵",
    },
    "mu_ratio_pellis": {
        "coefficients": [(17, 0), (2, -1), (5, 1)],
        "name": "Pellis μ/m_e",
        "formula": "17 + 2φ⁻¹ + 5φ",
    },
}


def evaluate_pellis_polynomial(coefficients: List[Tuple[float, float]],
                               phi: mpf) -> mpf:
    """Evaluate Pellis polynomial: P(φ) = Σ c_k·φ^{k}."""
    result = mpf(0)
    for coeff, k in coefficients:
        result = result + coeff * (phi ** k)
    return result


def compute_norm_trinity(monomial: TrinityMonomial, evaluator: FormulaEvaluator) -> mpf:
    """Compute |M| = sqrt(M²) for Trinity monomial."""
    value = evaluator.compute_monial(monomial)
    return sqrt(value ** 2)


def compute_norm_pellis(coefficients: List[Tuple[float, float]],
                     evaluator: FormulaEvaluator) -> mpf:
    """Compute |P| = sqrt(P²) for Pellis polynomial."""
    value = evaluate_pellis_polynomial(coefficients, evaluator.phi)
    return sqrt(value ** 2)


def compute_inner_product(trinity_m: TrinityMonomial,
                       pellis_coeffs: List[Tuple[float, float]],
                       evaluator: FormulaEvaluator) -> Dict[str, mpf]:
    """Compute ⟨M, P⟩ = (M·P) / (|M|·|P|)."""
    m_value = evaluator.compute_monial(trinity_m)
    p_value = evaluate_pellis_polynomial(pellis_coeffs, evaluator.phi)

    m_norm = compute_norm_trinity(trinity_m, evaluator)
    p_norm = compute_norm_pellis(pellis_coeffs, evaluator)

    # Inner product: M·P / (|M|·|P|)
    if m_norm * p_norm == 0:
        inner_product = mpf(0)
    else:
        inner_product = (m_value * p_value) / (m_norm * p_norm)

    return {
        "M_value": m_value,
        "P_value": p_value,
        "M_norm": m_norm,
        "P_norm": p_norm,
        "inner_product": inner_product,
    }


def main() -> int:
    """Main execution."""
    mp.dps = 100

    evaluator = FormulaEvaluator(dps=100)

    results = []

    # Compute inner products for each Pellis constant
    print("=== Hybrid Inner Product Analysis ===\n")

    for key, const_info in PELLIS_CONSTANTS.items():
        print(f"Constant: {const_info['name']} ({const_info['description']})")

        # Get Trinity monomial
        trinity_m = const_info.get("trinity_formula")
        if trinity_m is None:
            trinity_m = TrinityMonomial(n=1, k=0, m=0, p=0)

        # Compute with Trinity monomial only
        m_norm = compute_norm_trinity(trinity_m, evaluator)
        p_value_const = const_info.get("value", mpf(0))
        p_norm = sqrt(p_value_const ** 2)

        # Simple inner product: ⟨M, P⟩ = M·P / |M|·|P|
        m_value = evaluator.compute_monial(trinity_m)

        if m_norm * p_norm == 0:
            simple_ip = mpf(0)
        else:
            simple_ip = (m_value * p_value_const) / (m_norm * p_norm)

        result = {
            "constant": key,
            "name": const_info["name"],
            "description": const_info["description"],
            "trinity_monomial": trinity_m.to_string(),
            "trinity_value": str(evaluator.format_50_digit(m_value)),
            "pellis_value": str(evaluator.format_50_digit(p_value_const)),
            "trinity_norm": str(evaluator.format_50_digit(m_norm)),
            "pellis_norm": str(evaluator.format_50_digit(p_norm)),
            "inner_product": str(evaluator.format_50_digit(simple_ip)),
            "cosine_similarity": float(simple_ip),
        }

        # Compute with Pellis polynomial if available
        pellis_key = f"{key}_pellis"
        if pellis_key in PELLIS_POLYNOMIALS:
            pellis_coeffs = PELLIS_POLYNOMIALS[pellis_key]["coefficients"]
            p_poly_value = evaluate_pellis_polynomial(pellis_coeffs, evaluator.phi)
            p_poly_norm = compute_norm_pellis(pellis_coeffs, evaluator)

            if m_norm * p_poly_norm == 0:
                pellis_ip = mpf(0)
            else:
                pellis_ip = (m_value * p_poly_value) / (m_norm * p_poly_norm)

            result["pellis_polynomial"] = PELLIS_POLYNOMIALS[pellis_key]["formula"]
            result["pellis_poly_value"] = str(evaluator.format_50_digit(p_poly_value))
            result["pellis_inner_product"] = str(evaluator.format_50_digit(pellis_ip))
            result["pellis_cosine_similarity"] = float(pellis_ip)

        results.append(result)

        print(f"  Trinity monomial: {trinity_m.to_string()}")
        print(f"  Trinity value: {evaluator.format_50_digit(m_value)}")
        print(f"  Pellis value: {evaluator.format_50_digit(p_value_const)}")
        print(f"  Inner product: {evaluator.format_50_digit(simple_ip)}")
        if "pellis_inner_product" in result:
            print(f"  Pellis polynomial IP: {evaluator.format_50_digit(pellis_ip)}")
        print()

    # Prepare output
    output_dir = Path(__file__).parent.parent / "output"
    output_dir.mkdir(exist_ok=True)

    # Write JSON output
    json_path = output_dir / "hybrid_inner_products.json"
    import json
    with open(json_path, 'w') as f:
        json.dump(results, f, indent=2, default=str)
    print(f"Saved: {json_path}")

    # Write Markdown table for paper
    md_path = output_dir / "hybrid_inner_product_table.md"
    with open(md_path, 'w') as f:
        f.write("# Hybrid Inner Product Analysis\n\n")
        f.write("## Methodology\n\n")
        f.write("For Trinity monomial $M$ and Pellis constant $P$:\n\n")
        f.write("$$\\langle M, P \\rangle = \\frac{M \\cdot P}{|M| \\cdot |P|}$$\n\n")

        f.write("where:\n")
        f.write("- $M = n \\cdot \\phi^k \\cdot \\pi^m \\cdot e^p$ (Trinity monomial)\n")
        f.write("- $P$ is the measured Pellis constant\n")
        f.write("- $|M| = \\sqrt{M^2}$ is the norm\n\n")

        f.write("## Results\n\n")
        f.write("| Constant | Description | Trinity Monomial | Trinity Value | Pellis Value | Inner Product | Cosine Similarity |\n")
        f.write("|----------|-------------|------------------|---------------|---------------|---------------|-------------------|\n")

        for r in results:
            ip_str = r["inner_product"]
            cosine = r["cosine_similarity"]
            f.write(f'| ${r["name"]}$ | {r["description"]} | ${r["trinity_monomial"]}$ | `{r["trinity_value"]}` | `{r["pellis_value"]}` | `{ip_str}` | {cosine:.15f} |\n')

        # Add Pellis polynomial section if available
        has_pellis = any("pellis_inner_product" in r for r in results)
        if has_pellis:
            f.write("\n### Pellis Polynomial Inner Products\n\n")
            f.write("| Constant | Pellis Polynomial | Poly Value | Inner Product | Cosine Similarity |\n")
            f.write("|----------|-------------------|-----------|---------------|-------------------|\n")

            for r in results:
                if "pellis_inner_product" in r:
                    ip_str = r["pellis_inner_product"]
                    cosine = r["pellis_cosine_similarity"]
                    f.write(f'| ${r["name"]}$ | ${r["pellis_polynomial"]}$ | `{r["pellis_poly_value"]}` | `{ip_str}` | {cosine:.15f} |\n')

        f.write("\n## Interpretation\n\n")
        f.write("The inner product measures the alignment between Trinity monomials\n")
        f.write("and measured Pellis constants. Values close to 1 indicate\n")
        f.write("strong alignment, while values near 0 indicate orthogonality.\n")

    print(f"Saved: {md_path}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
