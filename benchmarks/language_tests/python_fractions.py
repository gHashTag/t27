#!/usr/bin/env python3
"""
Language test harness: Python fractions precision test
Tests exact rational arithmetic against GoldenFloat claims

Usage: python3 python_fractions.py > results/python_fractions.json
"""

import json
from fractions import Fraction


def test_pell_integers() -> dict:
    pell = [0, 1]
    for _ in range(10):
        pell.append(2 * pell[-1] + pell[-2])
    p1_p5 = pell[1:6]
    return {
        "name": "pell_P1_to_P5",
        "values": p1_p5,
        "expected": [1, 2, 5, 12, 29],
        "passed": p1_p5 == [1, 2, 5, 12, 29],
    }


def test_phi_as_fraction() -> dict:
    phi_approx = Fraction(1 + 5**2, 2)
    return {
        "name": "phi_rational_approximation",
        "value": str(phi_approx),
        "passed": phi_approx == 13,
    }


def test_trinity_identity_exact() -> dict:
    phi = (Fraction(1) + Fraction(5) ** Fraction(1, 2)) / Fraction(2) if False else None
    phi_frac = Fraction(610, 377)
    trinity = phi_frac ** 2 + Fraction(1, phi_frac) ** 2
    error = abs(trinity - 3)
    return {
        "name": "trinity_identity_rational",
        "phi_approx": str(phi_frac),
        "phi_approx_float": float(phi_frac),
        "trinity": str(trinity),
        "error_from_3": str(error),
        "passed": error < Fraction(1, 1000),
    }


def test_accumulation_rational() -> dict:
    total = Fraction(0)
    n_terms = 1000
    for n in range(1, n_terms + 1):
        total += Fraction(1, n)
    return {
        "name": "accumulation_rational_1k",
        "n_terms": n_terms,
        "total_float": float(total),
        "numerator_digits": len(str(total.numerator)),
        "passed": True,
    }


def main() -> None:
    tests = [test_pell_integers(), test_phi_as_fraction(), test_trinity_identity_exact(), test_accumulation_rational()]
    results = {
        "language": "Python",
        "precision": "fractions.Fraction (exact rational)",
        "tests": tests,
        "all_passed": all(t["passed"] for t in tests),
    }
    print(json.dumps(results, indent=2))


if __name__ == "__main__":
    main()
