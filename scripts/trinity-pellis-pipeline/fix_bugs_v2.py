#!/usr/bin/env python3
"""Fix BUG-1 and BUG-2 in G2_ALPHA_S_PHI_FRAMEWORK_V0.9.tex

BUG-1: Poisson exact p-value uses 10^{-53} instead of 10^{-4}
BUG-2: CKM unitarity formula has approx 1$ instead of = 1$ (appears in TWO places)
"""

import sys
from pathlib import Path

tex_file = Path(__file__).parent.parent.parent / "research" / "trinity-pellis-paper" / "G2_ALPHA_S_PHI_FRAMEWORK_V0.9.tex"

with open(tex_file, 'r') as f:
    content = f.read()

print(f"Read {len(content)} lines")

# Fix BUG-1: Change 10^{-53} to 10^{-4} in summary table (line 180)
content = content.replace("1.47 \\times 10^{-53}", "1.47 \\times 10^{-4}")

# Fix BUG-2: Change approx 1$ to = 1$ in BOTH places
# This requires finding the exact formula occurrences and replacing them

# First, let's find all occurrences of the buggy CKM unitarity formula
buggy_formula = "$|V_{ud}|^2 + |V_{us}|^2 + |V_{ub}|^2 \\approx 1$"

# Count occurrences before replacement
count_before = content.count(buggy_formula)
print(f"Found {count_before} occurrences of buggy formula")

# Replace all occurrences with correct formula
correct_formula = "$|V_{ud}|^2 + |V_{us}|^2 + |V_{ub}|^2 = 1$"
content = content.replace(buggy_formula, correct_formula)

# Count occurrences after replacement
count_after = content.count(correct_formula)
print(f"Replaced {count_before} -> {count_after} occurrences")

with open(tex_file, 'w') as f:
    f.write(content)

print(f"Lines modified: {len(content) - len(content.split(chr(10)))}")

# Verify
print("\nVerification:")
if "1.47 \\times 10^{-4}" in content:
    print("  ✓ BUG-1 fixed: 10^{-53} → 10^{-4}")
else:
    print("  ✗ BUG-1 not found")

# Check for buggy CKM formula (should NOT be found anymore)
if buggy_formula in content:
    print("  ✗ BUG-2 still present - should be fixed")
else:
    print("  ✓ BUG-2 fixed: all CKM unitarity formulas corrected")
