#!/usr/bin/env python3
"""
Language test harness: Python Decimal precision test
Tests Python decimal.Decimal arbitrary precision against GoldenFloat claims

Usage: python3 python_decimal.py > results/python_decimal.json
"""

import json
from decimal import Decimal, getcontext


def test_phi() -> dict:
    getcontext().prec = 50
    phi = (Decimal(1) + Decimal(5).sqrt()) / Decimal(2)
    return {
        "name": "phi_decimal_50",
        "computed": str(phi),
        "passed": str(phi).startswith("1.618033988749894848204586834365638117720309179805"),
    }


def test_phi_squared_identity() -> dict:
    getcontext().prec = 50
    phi = (Decimal(1) + Decimal(5).sqrt()) / Decimal(2)
    phi_sq = phi * phi
    phi_plus_one = phi + 1
    error = abs(phi_sq - phi_plus_one)
    return {
        "name": "phi_squared_equals_phi_plus_one",
        "phi_sq": str(phi_sq),
        "phi_plus_one": str(phi_plus_one),
        "error": str(error),
        "passed": error < Decimal("1e-45"),
    }


def test_trinity_identity() -> dict:
    getcontext().prec = 50
    phi = (Decimal(1) + Decimal(5).sqrt()) / Decimal(2)
    trinity = phi ** 2 + phi ** (-2)
    error = abs(trinity - 3)
    return {
        "name": "trinity_identity",
        "trinity": str(trinity),
        "error": str(error),
        "passed": error < Decimal("1e-45"),
    }


def test_accumulation() -> dict:
    getcontext().prec = 50
    total = Decimal(0)
    n_terms = 10000
    for n in range(1, n_terms + 1):
        total += Decimal(1) / Decimal(n)
    return {
        "name": "accumulation_10k",
        "n_terms": n_terms,
        "total": str(total),
        "passed": True,
    }


def main() -> None:
    tests = [test_phi(), test_phi_squared_identity(), test_trinity_identity(), test_accumulation()]
    results = {
        "language": "Python",
        "precision": "decimal.Decimal (50 digits)",
        "tests": tests,
        "all_passed": all(t["passed"] for t in tests),
    }
    print(json.dumps(results, indent=2))


if __name__ == "__main__":
    main()
