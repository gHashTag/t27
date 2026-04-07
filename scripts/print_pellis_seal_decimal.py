#!/usr/bin/env python3
"""Print Pellis α⁻¹ = 360/φ² - 2/φ³ + (3φ)⁻⁵ using only stdlib Decimal (no mpmath).

Run: python3 scripts/print_pellis_seal_decimal.py
Optional: python3 scripts/print_pellis_seal_decimal.py 110   # decimal precision

Use to refresh the committed digit snapshot in research/trinity-pellis-paper/FORMULA_TABLE.md.
"""

from __future__ import annotations

import sys
from decimal import Decimal, getcontext


def main() -> int:
    prec = int(sys.argv[1]) if len(sys.argv) > 1 else 90
    # `prec` = desired significant digits in the result (~137.x); extra guard for sqrt/powers.
    getcontext().prec = max(prec + 40, 55)
    sqrt5 = Decimal(5).sqrt()
    phi = (Decimal(1) + sqrt5) / Decimal(2)
    pellis = Decimal(360) / phi**2 - Decimal(2) / phi**3 + Decimal(1) / ((Decimal(3) * phi) ** 5)
    print(format(pellis, "f"))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
