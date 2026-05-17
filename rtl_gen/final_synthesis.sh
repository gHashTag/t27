#!/bin/bash

modules="gf4_add gf8_add gf12_add gf16_add gf20_add gf24_add gf32_add gf64_add gf128_add gf256_add
          gf16_mul gf20_mul gf24_mul gf32_mul gf64_mul gf128_mul gf256_mul"

echo "# Trinity t27 RTL Synthesis Cell Count"
echo "# Generated: $(date)"
echo ""

for mod in $modules; do
    if [ -f build/${mod}.json ]; then
        # Count cells in JSON
        cells=$(grep -o '"type":"[^"]*"' build/${mod}.json | wc -l)
        echo "${mod}: ${cells} cells"
    fi
done
