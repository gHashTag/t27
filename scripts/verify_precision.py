#!/usr/bin/env python3
"""High-precision replay of Trinity / Pellis formulas (mpmath).

NOT on the math verification critical path (see docs/nona-02-organism/TZ-T27-001-NO-PYTHON-CRITICAL-PATH.md).
Use for research, pre-registered checkpoints, and reviewer-facing digit dumps. SSOT for release gates
remains specs/*.t27 + tri / t27c.

Dependencies:
  pip install -r scripts/requirements-verify-precision.txt

No-deps Pellis-only one-liner (stdlib Decimal): scripts/print_pellis_seal_decimal.py

Run from repo root:
  python3 scripts/verify_precision.py
  python3 scripts/verify_precision.py --dps 200

FORMULA_TABLE row coverage (this script):
  Computed from phi / integers: 1, 2, 3, 5, 22, 23, 24, 25, 27, 28, 29, 30, 31.
  Not computed here: 4 (CODATA tag), 6–21 (hybrid maps + SM refs in CLI), 10, 26 (need inputs),
  11–13, 14–20 (PDG-only references). Extend with explicit mpf inputs if you need full-row dumps.
"""

from __future__ import annotations

import argparse
import sys


def main() -> int:
    try:
        from mpmath import atan, degrees, fabs, mp, mpf, nstr, sqrt
    except ImportError:
        print(
            "mpmath is required: pip install -r scripts/requirements-verify-precision.txt",
            file=sys.stderr,
        )
        return 1

    p = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    p.add_argument(
        "--dps",
        type=int,
        default=100,
        help="decimal places (mpmath working precision)",
    )
    p.add_argument(
        "--pellis-digits",
        type=int,
        default=0,
        metavar="N",
        help="if >0, print only row-31 Pellis alpha^-1 with N digits after decimal (no label noise)",
    )
    args = p.parse_args()

    if args.pellis_digits > 0:
        mp.dps = max(args.pellis_digits + 25, 80)
        phi = (1 + sqrt(5)) / 2
        pellis_alpha_inv = 360 / phi**2 - 2 / phi**3 + 1 / (3 * phi) ** 5
        # Total significant digits ~ integer part + fractional
        print(nstr(pellis_alpha_inv, args.pellis_digits + 6))
        return 0

    if args.dps < 30:
        print("--dps below 30 is not useful for pre-registration", file=sys.stderr)
    mp.dps = args.dps

    phi = (1 + sqrt(5)) / 2
    inv_phi = 1 / phi

    # --- Row 1: L5 Trinity sum
    l5 = phi**2 + inv_phi**2
    l5_residual = l5 - 3

    # --- Row 2: phi^2 = phi + 1
    golden_residual = phi**2 - phi - 1

    # --- Row 3: Pell P_1..P_5 (exact integers via recurrence)
    pell = [0, 1]
    for _ in range(10):
        pell.append(2 * pell[-1] + pell[-2])
    # P_1..P_5 in 1-indexed naming used in repo: values 1,2,5,12,29
    p1_p5 = [pell[1], pell[2], pell[3], pell[4], pell[5]]

    # --- Row 5
    phi5 = phi**5

    # --- Row 22/23
    phi_inv_cubed = inv_phi**3

    # --- Row 24
    phi_pow_m65 = phi ** mpf("-6.5")

    # --- Row 25
    phi_pow_m115 = phi ** mpf("-11.5")

    # --- Row 27
    theta12_rad = atan(1 / phi)
    theta12_deg = degrees(theta12_rad)

    # --- Row 28-30
    phi17 = phi**17
    phi11 = phi**11
    phi8 = phi**8

    # --- Row 31: Pellis closed form for alpha^-1
    term1 = 360 / phi**2
    term2 = 2 / phi**3
    term3 = 1 / (3 * phi) ** 5
    pellis_alpha_inv = term1 - term2 + term3

    print(f"mpmath dps = {mp.dps}")
    print(f"phi = {nstr(phi, mp.dps)}")
    print()
    print("=== FORMULA_TABLE row mapping (computable from phi / integers) ===")
    print()
    print(f"Row 1  L5: phi^2 + phi^-2 = {nstr(l5, 50)}")
    print(f"       residual (L5 - 3) = {nstr(l5_residual, 50)}")
    print(f"Row 2  residual (phi^2 - phi - 1) = {nstr(golden_residual, 50)}")
    print(f"Row 3  Pell P_1..P_5 (integers) = {p1_p5}")
    print(f"Row 5  phi^5 = {nstr(phi5, 50)}")
    print(f"Row 22/23  phi^-3 = {nstr(phi_inv_cubed, 50)}")
    print(f"Row 24  phi^-6.5 = {nstr(phi_pow_m65, 50)}")
    print(f"Row 25  phi^-11.5 = {nstr(phi_pow_m115, 50)}")
    print(f"Row 27  theta_12 = arctan(1/phi) rad = {nstr(theta12_rad, 50)}")
    print(f"Row 27  theta_12 deg = {nstr(theta12_deg, 50)}")
    print(f"Row 28  phi^17 = {nstr(phi17, 50)}")
    print(f"Row 29  phi^11 = {nstr(phi11, 50)}")
    print(f"Row 30  phi^8 = {nstr(phi8, 50)}")
    print()
    print("Row 31  Pellis alpha^-1 = 360/phi^2 - 2/phi^3 + (3*phi)^-5")
    print(f"       = {nstr(pellis_alpha_inv, mp.dps)}")
    print()
    print("Terms (row 31):")
    print(f"  360/phi^2     = {nstr(term1, 40)}")
    print(f"  2/phi^3       = {nstr(term2, 40)}")
    print(f"  (3*phi)^-5    = {nstr(term3, 40)}")
    print()
    print(f"|L5 - 3| = {nstr(fabs(l5_residual), 10)}")
    print(f"|golden residual| = {nstr(fabs(golden_residual), 10)}")
    print()
    print(
        "Rows not printed (need PDG/hybrid/CLI inputs): "
        "4, 6-21, 10, 11-13, 14-20, 26 — see module docstring."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
