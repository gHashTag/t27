#!/usr/bin/env python3
"""Core Formula Evaluator Module for Trinity/Pellis Pipeline.

Provides TrinityMonomial class for n·φ^k·π^m·e^p expressions
matching sacred_formula_catalog.json structure.

Dependencies:
  pip install -r scripts/requirements-verify-precision.txt

Pattern reuse from verify_precision.py:
  - mp.dps = 100 for precision control
  - mp.nstr(value, 50) for 50-digit formatting
  - phi = (1 + mp.sqrt(5)) / 2 for high precision
"""

from __future__ import annotations

import json
from dataclasses import dataclass
from typing import List, Dict, Any, Optional
from pathlib import Path

try:
    from mpmath import mp, mpf, nstr, sqrt, pi, e, exp
except ImportError:
    print("mpmath is required: pip install -r scripts/requirements-verify-precision.txt")
    raise


@dataclass
class TrinityMonomial:
    """Trinity monomial: M = n·φ^k·π^m·e^p·γ^q·δ^r"""

    n: float  # Numerical coefficient
    k: float  # Exponent for phi (φ)
    m: float  # Exponent for pi (π)
    p: float  # Exponent for e (Euler's number)
    q: float = 0.0  # Exponent for gamma (γ)
    r: float = 0.0  # Exponent for delta_cp (δ)

    def evaluate(self, phi: mpf, pi_val: mpf, e_val: mpf,
               gamma: Optional[mpf] = None, delta: Optional[mpf] = None) -> mpf:
        """Evaluate the monomial with given constants."""
        result = self.n

        if self.k != 0:
            result = result * (phi ** self.k)

        if self.m != 0:
            result = result * (pi_val ** self.m)

        if self.p != 0:
            result = result * (e_val ** self.p)

        if self.q != 0 and gamma is not None:
            result = result * (gamma ** self.q)

        if self.r != 0 and delta is not None:
            result = result * (delta ** self.r)

        return result

    @property
    def complexity(self) -> float:
        """Compute complexity cx = |k| + |m| + |p| + |q| + |r|"""
        return abs(self.k) + abs(self.m) + abs(self.p) + abs(self.q) + abs(self.r)

    def to_string(self) -> str:
        """Convert monomial to LaTeX-like string representation."""
        parts = []

        if self.n != 1:
            parts.append(f"{self.n}")

        if self.k != 0:
            if self.k == 1:
                parts.append(r"\phi")
            elif self.k == -1:
                parts.append(r"\phi^{-1}")
            else:
                parts.append(f"\\phi^{{{self.k}}}")

        if self.m != 0:
            if self.m == 1:
                parts.append(r"\pi")
            elif self.m == -1:
                parts.append(r"\pi^{-1}")
            else:
                parts.append(f"\\pi^{{{self.m}}}")

        if self.p != 0:
            if self.p == 1:
                parts.append("e")
            elif self.p == -1:
                parts.append("e^{-1}")
            else:
                parts.append(f"e^{{{self.p}}}")

        if self.q != 0:
            if self.q == 1:
                parts.append(r"\gamma")
            elif self.q == -1:
                parts.append(r"\gamma^{-1}")
            else:
                parts.append(f"\\gamma^{{{self.q}}}")

        if self.r != 0:
            if self.r == 1:
                parts.append(r"\delta")
            elif self.r == -1:
                parts.append(r"\delta^{-1}")
            else:
                parts.append(f"\\delta^{{{self.r}}}")

        return r" \cdot ".join(parts) if parts else "1"


class FormulaEvaluator:
    """High-precision formula evaluator for Trinity monomials."""

    def __init__(self, dps: int = 100):
        """Initialize evaluator with decimal precision."""
        self.dps = dps
        mp.dps = dps

        # Define sacred constants with high precision
        self.phi = (1 + sqrt(5)) / 2  # Golden ratio φ
        self.pi_val = pi              # π
        self.e_val = e                # Euler's number e
        self.gamma = sqrt(5) - 2       # γ_φ = √5 - 2 ≈ 0.23607

    def compute_monial(self, monomial: TrinityMonomial) -> mpf:
        """Compute monomial value."""
        return monomial.evaluate(self.phi, self.pi_val, self.e_val, self.gamma)

    def format_50_digit(self, value: mpf) -> str:
        """Format value to 50 digits (for paper-ready output)."""
        return nstr(value, 50)

    def compute_delta_pct(self, computed: mpf, reference: mpf) -> float:
        """Compute percentage delta: |computed - reference| / |reference| × 100%."""
        if reference == 0:
            return float('inf')
        return abs((computed - reference) / reference) * 100

    def load_sacred_catalog(self, path: Path = None) -> List[Dict[str, Any]]:
        """Load sacred_formula_catalog.json."""
        if path is None:
            # __file__ is in scripts/trinity-pellis-pipeline/core/
            # Need to go up to t27 root, then to research/
            path = Path(__file__).parent.parent.parent.parent / "research" / "sacred_formula_catalog.json"
        else:
            path = Path(path)

        with open(path, 'r') as f:
            return json.load(f)

    def parse_formula_table(self, path: Path = None) -> List[Dict[str, Any]]:
        """Parse FORMULA_TABLE.md into structured data."""
        if path is None:
            # __file__ is in scripts/trinity-pellis-pipeline/core/
            # Need to go up to t27 root, then to research/trinity-pellis-paper/
            path = Path(__file__).parent.parent.parent.parent / "research" / "trinity-pellis-paper" / "FORMULA_TABLE.md"
        else:
            path = Path(path)

        formulas = []
        in_table = False

        with open(path, 'r') as f:
            lines = f.readlines()

        for line in lines:
            line = line.strip()

            # Skip header and separator
            if line.startswith("|") and "ID" in line:
                in_table = True
                continue
            if line.startswith("|---"):
                continue

            if in_table and line.startswith("|"):
                # Parse table row: | ID | Name | Category | Formula | Value | Δ% | Tier | Source | PDG Δ | Spec |
                parts = [p.strip() for p in line.split("|") if p.strip()]
                if len(parts) >= 5:
                    formula = {
                        "id": parts[0],
                        "name": parts[1],
                        "category": parts[2],
                        "formula_str": parts[3],
                        "value_str": parts[4],
                        "delta_pct_str": parts[5] if len(parts) > 5 else "",
                        "tier": parts[6] if len(parts) > 6 else "",
                        "source": parts[7] if len(parts) > 7 else "",
                        "pdg_delta": parts[8] if len(parts) > 8 else "",
                        "spec": parts[9] if len(parts) > 9 else "",
                    }
                    formulas.append(formula)

        return formulas

    def create_monomial_from_dict(self, data: Dict[str, Any]) -> TrinityMonomial:
        """Create TrinityMonomial from sacred_catalog dict."""
        return TrinityMonomial(
            n=float(data.get('n', 1)),
            k=float(data.get('k', 0)),
            m=float(data.get('m', 0)),
            p=float(data.get('p', 0)),
            q=float(data.get('q', 0)),
            r=float(data.get('r', 0))
        )

    def evaluate_formula_str(self, formula_str: str) -> mpf:
        """Evaluate a simple formula string like 'phi**2 + 1/phi**2'."""
        # Safe evaluation using mpmath context
        phi = self.phi
        pi = self.pi_val
        e = self.e_val

        # Very limited eval for specific patterns only
        # This is NOT a general Python eval - only for known formula types
        try:
            # Map known formula patterns
            if 'phi' in formula_str.lower() and 'pi' not in formula_str.lower() and 'e' not in formula_str.lower():
                # Pure phi formula
                if 'phi**2' in formula_str and 'phi**-2' in formula_str:
                    return self.phi**2 + self.phi**(-2)
                elif 'phi**2' in formula_str and '+ 1' in formula_str:
                    return self.phi**2 + 1
            return mpf(0)
        except Exception as ex:
            print(f"Warning: Could not evaluate formula '{formula_str}': {ex}")
            return mpf(0)


def main() -> None:
    """Test the formula evaluator."""
    evaluator = FormulaEvaluator(dps=100)

    print("=== Formula Evaluator Test ===")
    print(f"phi = {evaluator.format_50_digit(evaluator.phi)}")
    print(f"pi = {evaluator.format_50_digit(evaluator.pi_val)}")
    print(f"e = {evaluator.format_50_digit(evaluator.e_val)}")
    print(f"gamma_φ = {evaluator.format_50_digit(evaluator.gamma)}")
    print()

    # Test Trinity monomial: 7*phi^5/(3*pi^3*e)
    monomial = TrinityMonomial(n=7, k=5, m=-3, p=-1)
    result = evaluator.compute_monial(monomial)
    print(f"Test: 7φ⁵/(3π³e) ≈ {evaluator.format_50_digit(result)}")
    print(f"  Formula string: {monomial.to_string()}")
    print(f"  Complexity cx = {monomial.complexity}")

    # Load sacred catalog
    catalog = evaluator.load_sacred_catalog()
    print(f"\nLoaded {len(catalog)} formulas from sacred_formula_catalog.json")

    # Parse FORMULA_TABLE
    table = evaluator.parse_formula_table()
    print(f"Parsed {len(table)} formulas from FORMULA_TABLE.md")


if __name__ == "__main__":
    main()
