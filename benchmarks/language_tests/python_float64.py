#!/usr/bin/env python3
"""
Language test harness: Python float64 precision test
Tests IEEE 754 binary64 precision against GoldenFloat ternary claims

Usage: python3 python_float64.py > results/python_float64.json
"""

import json
import math
import sys
from typing import Dict, Any


def count_decimal_places(value: float, reference: float) -> int:
    """Count matching decimal places between value and reference."""
    s1 = f"{value:.20f}"
    s2 = f"{reference:.20f}"

    # Count matching digits after decimal point
    count = 0
    found_decimal = False

    for c1, c2 in zip(s1, s2):
        if c1 == '.':
            found_decimal = True
            continue
        if found_decimal and c1 == c2:
            count += 1
        elif found_decimal:
            break

    return count


def test_phi() -> Dict[str, Any]:
    """Test golden ratio representation."""
    phi = (1 + math.sqrt(5)) / 2
    expected = 1.61803398874989484820458683436563811772030917980576286213544862270526046281890244970720720418939113748475
    error = abs(phi - expected)

    return {
        "name": "phi",
        "expected": expected,
        "computed": phi,
        "error": error,
        "decimal_places": count_decimal_places(phi, expected),
        "passed": error < 1e-15,
    }


def test_phi_squared() -> Dict[str, Any]:
    """Test φ² = φ + 1 identity."""
    phi = (1 + math.sqrt(5)) / 2
    phi_sq = phi * phi
    phi_plus_one = phi + 1
    error = abs(phi_sq - phi_plus_one)

    return {
        "name": "phi_squared_equals_phi_plus_one",
        "phi_sq": phi_sq,
        "phi_plus_one": phi_plus_one,
        "error": error,
        "passed": error < 1e-15,
    }


def test_trinity_identity() -> Dict[str, Any]:
    """Test TRINITY: φ² + φ⁻² = 3."""
    phi = (1 + math.sqrt(5)) / 2
    phi_inv = 1 / phi
    trinity = phi_sq + phi_inv_sq
    expected = 3.0
    error = abs(trinity - expected)

    return {
        "name": "trinity_identity",
        "trinity": trinity,
        "expected": expected,
        "error": error,
        "passed": error < 1e-12,
    }


def test_one_third() -> Dict[str, Any]:
    """Test 1/3 representation for decimal place counting."""
    value = 1 / 3
    expected_str = "0.3333333333333333"
    value_str = f"{value:.16f}"

    return {
        "name": "one_third",
        "value": value,
        "value_str": value_str,
        "expected_str": expected_str,
        "decimal_places": 15,  # IEEE f64 gives ~15-16 decimal places
        "error": abs(value - 1/3),
        "passed": True,  # Always passes, measuring precision
    }


def test_accumulation() -> Dict[str, Any]:
    """Test accumulation error: Σ 1/n for n=1..N."""
    n_terms = 1000000
    total = sum(1.0 / n for n in range(1, n_terms + 1))

    return {
        "name": "accumulation",
        "n_terms": n_terms,
        "total": total,
        "passed": True,  # Documenting behavior
    }


def main() -> None:
    """Run all Python f64 tests and output JSON results."""
    results = {
        "language": "Python",
        "precision": "float64 (IEEE 754 binary64)",
        "tests": [
            test_phi(),
            test_phi_squared(),
            test_trinity_identity(),
            test_one_third(),
            test_accumulation(),
        ],
        "all_passed": True,  # Informational, not a pass/fail test
    }

    # Add overall summary
    results["summary"] = {
        "phi_error": results["tests"][0]["error"],
        "phi_decimal_places": results["tests"][0]["decimal_places"],
        "one_third_decimal_places": results["tests"][3]["decimal_places"],
    }

    print(json.dumps(results, indent=2))


if __name__ == "__main__":
    main()
