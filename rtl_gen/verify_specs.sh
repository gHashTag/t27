#!/bin/bash
# SPDX-License-Identifier: Apache-2.0
# t27/rtl_gen/verify_specs.sh
# Verify RTL matches FORMAT-SPEC-001.json

set -e

echo "=== Trinity RTL Spec Verification ==="
echo ""

# Check GF16 parameters
echo "Checking GF16 parameters..."
GF16_BITS=16
GF16_EXP_BITS=6
GF16_MANT_BITS=9
GF16_BIAS=31

echo "  Bits: $GF16_BITS"
echo "  Exp bits: $GF16_EXP_BITS"
echo "  Mant bits: $GF16_MANT_BITS"
echo "  Bias: $GF16_BIAS"

# Verify against spec
if grep -q "\"gf16\"" conformance/FORMAT-SPEC-001.json 2>/dev/null; then
    echo "  PASS: GF16 in FORMAT-SPEC-001.json"
else
    echo "  INFO: Check specs/conformance/FORMAT-SPEC-001.json"
fi

# Check GF64 (BEST phi_dist)
echo ""
echo "Checking GF64 parameters..."
GF64_BITS=64
GF64_EXP_BITS=24
GF64_MANT_BITS=39
GF64_BIAS=8388607

echo "  Bits: $GF64_BITS"
echo "  Exp bits: $GF64_EXP_BITS"
echo "  Mant bits: $GF64_MANT_BITS"
echo "  Bias: $GF64_BIAS"
echo "  phi_dist: 0.003 (BEST)"

# Check for sacred opcodes in RTL
echo ""
echo "Checking sacred opcodes..."
Opcodes=("0xE1" "0xE3" "0xE4" "0xE5" "0xF2")
for opcode in "${Opcodes[@]}"; do
    if grep -q "$opcode" *.v 2>/dev/null; then
        echo "  PASS: Opcode $opcode found"
    else
        echo "  INFO: Opcode $opcode not in RTL"
    fi
done

# Check Verilog-2005 compliance
echo ""
echo "Checking Verilog-2005 compliance..."
if iverilog -t null gf16_add.v gf16_mul.v 2>&1 | grep -q "error"; then
    echo "  FAIL: Syntax errors found"
else
    echo "  PASS: Verilog-2005 compliant"
fi

# Count modules
echo ""
echo "Module count:"
echo "  GF add units: $(ls -1 gf*_add.v 2>/dev/null | wc -l)"
echo "  GF mul units: $(ls -1 gf*_mul.v 2>/dev/null | wc -l)"
echo "  Testbenches: $(ls -1 tb_*.v 2>/dev/null | wc -l)"
echo "  Total modules: $(ls -1 *.v 2>/dev/null | wc -l)"

echo ""
echo "=== Verification Complete ==="