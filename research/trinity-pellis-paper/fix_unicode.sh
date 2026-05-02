#!/bin/bash
# Fix Unicode characters in LaTeX files for proper LaTeX syntax

# Files to fix
FILES=(
    "banks_zaks_fixed_point.tex"
    "a5_coxeter_characteristic.tex"
    "h3_e8_projection.tex"
    "koide_trinity_approx.tex"
    "phi4_theory_fixed_points.tex"
    "l_function_alpha_s.tex"
)

for f in "${FILES[@]}"; do
    if [ -f "$f" ]; then
        echo "=== Fixing $f ==="

        # Simple sed replacements
        sed -i '' -e 's/φ⁻³\/2\$/\\varphi^{-3}\/2\$/g' "$f"
        sed -i '' -e 's/φ⁻³\$/\\varphi^{-3}\$/g' "$f"
        sed -i '' -e 's/φ⁻¹\$/\\varphi^{-1}\$/g' "$f"
        sed -i '' -e 's/φ⁻²\$/\\varphi^{-2}\$/g' "$f"
        sed -i '' -e 's/φ⁵\$/\varphi^5\$/g' "$f"
        sed -i '' -e 's/φ⁴/\$^4\$/g' "$f"
        sed -i '' -e 's/₅/\$_5$/g' "$f"
        sed -i '' -e 's/₃/\$_3$/g' "$f"
        sed -i '' -e 's/₈/\$_8$/g' "$f"
        sed -i '' -e 's/₇/\$_7$/g' "$f"
        sed -i '' -e 's/₆/\$_6$/g' "$f"
        sed -i '' -e 's/₄/\$_4$/g' "$f"
        sed -i '' -e 's/₁/\$_1$/g' "$f"
        sed -i '' -e 's/₂/\$_2$/g' "$f"

        echo "✓ Fixed $f"
    else
        echo "✗ File not found: $f"
    fi
done

echo "=== Unicode fixes complete ==="
