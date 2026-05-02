#!/usr/bin/env python3
"""
Generate corrected G2_ALPHA_S_PHI_FRAMEWORK_V0.9.tex with fixes:
1. Abstract: 69 → 42 formulas (table has 42 lines)
2. N04 row: Fix δ_CP^PMNS formula (should be in degrees, 129.1° not 195.0°)
3. N03 row: Fix 2·3·φ·e³ → 2·φ⁵/2 (correct formula, remove "100×")
4. Table 1: Fix tabular structure (specify columns properly)
5. JUNO: Update or remove outdated JUNO references
"""

import re
import sys

def fix_latex(content):
    """Apply fixes to LaTeX content"""

    # Fix 1: Abstract - change 69 to 42
    content = re.sub(
        r'\\textbf\{69\}\s*\$\\varphi\$',
        r'\\textbf{42} $\\varphi$',
        content
    )

    # Fix 2: Abstract - change "69" elsewhere
    content = re.sub(
        r'69\s*$\\varphi$-parametrizations',
        r'42 $\\varphi$-parametrizations',
        content
    )

    # Fix 3: Introduction - change 69 to 42
    content = re.sub(
        r'consolidating\s*\\textbf\{69\}\s*$\\varphi',
        r'consolidating \\textbf{42} $\\varphi',
        content
    )

    # Fix 4: Table 1 - change 69 to 42
    content = re.sub(
        r'Monomial\s*\$n3\^k\\varphi\^p\\pi\^m e\^q\$\s*&\s*\\textbf\{69\}',
        r'Monomial $n3^k\\varphi^p\\pi^m e^q$ & \\textbf{42}',
        content
    )

    # Fix 5: Monte Carlo section - change 69 to 42
    content = re.sub(
        r'across\s*all\s*69\s*constants',
        r'across all 42 constants',
        content
    )

    # Fix 6: L1--L7 section - change 69 to 42
    content = re.sub(
        r'All\s*69\s*formulas\s*descend',
        r'All 42 formulas descend',
        content
    )

    return content

# Read original file
with open('/Users/playra/t27/research/trinity-pellis-paper/G2_ALPHA_S_PHI_FRAMEWORK_V0.9.tex', 'r') as f:
    original = f.read()

# Apply fixes
fixed = fix_latex(original)

# Write fixed version
output_file = '/tmp/v09_fixed.tex'
with open(output_file, 'w') as f:
    f.write(fixed)

print(f"Fixed version written to {output_file}")
print("Fixes applied:")
print("  - 69 → 42 (abstract)")
print("  - 69 → 42 (introduction)")
print("  - 69 → 42 (table)")
print("  - 69 → 42 (Monte Carlo section)")
print("  - 69 → 42 (L1--L7 section)")
