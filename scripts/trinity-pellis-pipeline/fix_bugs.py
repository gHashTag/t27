#!/usr/bin/env python3
"""Fix BUG-1 and BUG-2 in G2_ALPHA_S_PHI_FRAMEWORK_V0.9.tex

BUG-1: Poisson exact p-value uses 10^{-53} instead of 10^{-4}
BUG-2: CKM unitarity formula has approx 1$ instead of = 1$
"""

import sys
from pathlib import Path

tex_file = Path(__file__).parent.parent.parent / "research" / "trinity-pellis-paper" / "G2_ALPHA_S_PHI_FRAMEWORK_V0.9.tex"

with open(tex_file, 'r') as f:
    content = f.read()

print(f"Read {len(content)} lines")

# Fix BUG-1: Change 10^{-53} to 10^{-4} in summary table
# Two occurrences: line 167 (Poisson exact section) and line 180 (summary table)

content = content.replace("1.47 \\times 10^{-53}", "1.47 \\times 10^{-4}")

# Fix BUG-2: Change CKM unitarity approx 1$ to = 1$
# Find the CKM unitarity formula in Block Permutation section
content = content.replace("|V_{ud}|^2 + |V_{us}|^2 + |V_{ub}|^2 \\approx 1$",
                              "|V_{ud}|^2 + |V_{us}|^2 + |V_{ub}|^2 = 1$")

with open(tex_file, 'w') as f:
    f.write(content)

print(f"Fixed BUG-1 and BUG-2")
print(f"Lines modified: {len(content) - len(content.split(chr(10)))}")

# Verify fixes
print("\nVerification:")
if "1.47 \\times 10^{-4}" in content:
    print("  ✓ BUG-1 fixed: 10^{-53} → 10^{-4}")
else:
    print("  ✗ BUG-1 not found")

if "|V_{ud}|^2 + |V_{us}|^2 + |V_{ub}|^2 = 1$" in content:
    print("  ✓ BUG-2 fixed: CKM unitarity")
else:
    print("  ✗ BUG-2 not found")
