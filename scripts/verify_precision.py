#!/usr/bin/env python3
"""verify_precision.py — mpmath-backed sacred constant verification at 100 digits.

Outputs JSON with phi, phi_inv, phi_sq, Pellis closed form, pi, e, and identity checks.
Part of GF Competitive Analysis (issue #289).

Usage:
    python3 scripts/verify_precision.py
    python3 scripts/verify_precision.py --digits 200
"""

import json
import sys
import argparse


def main():
    parser = argparse.ArgumentParser(description="Verify sacred constants at high precision")
    parser.add_argument("--digits", type=int, default=100, help="Decimal digits of precision")
    parser.add_argument("--output", type=str, default=None, help="Output file (default: stdout)")
    args = parser.parse_args()

    try:
        from mpmath import mp, mpf, sqrt
    except ImportError:
        print("Error: mpmath not installed. Run: pip install mpmath", file=sys.stderr)
        sys.exit(1)

    mp.dps = args.digits

    phi = (mpf(1) + sqrt(mpf(5))) / mpf(2)
    phi_inv = 1 / phi
    phi_sq = phi * phi
    pi = mp.pi
    e = mp.e

    pellis = mpf(360) / phi_sq - mpf(2) / (phi ** 4) + mpf(1) / (mpf(3) * phi) ** 5

    trinity = phi_sq + phi_inv ** 2
    phi_identity = phi_sq - phi - 1
    trinity_identity = trinity - 3

    results = {
        "precision_digits": args.digits,
        "phi": str(phi),
        "phi_inv": str(phi_inv),
        "phi_sq": str(phi_sq),
        "pi": str(pi),
        "e": str(e),
        "pellis_closed_form": str(pellis),
        "trinity": str(trinity),
        "phi_identity_check": str(phi_identity),
        "trinity_identity_check": str(trinity_identity),
        "phi_digits_count": len(str(phi).replace(".", "").replace("-", "")),
    }

    output = json.dumps(results, indent=2)

    if args.output:
        with open(args.output, "w") as f:
            f.write(output)
        print(f"Written to {args.output}")
    else:
        print(output)


if __name__ == "__main__":
    main()
