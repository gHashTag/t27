#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# t27/rtl_gen/check_rsi1.py
# R-SI-1 Compliance Checker — Zero * operators

import sys
import glob
import os

RTL_SOURCES = [
    "gf_formats.v",
    "gf4_add.v", "gf8_add.v", "gf12_add.v", "gf16_add.v",
    "gf20_add.v", "gf24_add.v", "gf32_add.v", "gf64_add.v",
    "gf128_add.v", "gf256_add.v",
    "gf16_mul.v", "gf20_mul.v", "gf24_mul.v", "gf32_mul.v",
    "gf64_mul.v", "gf128_mul.v", "gf256_mul.v",
    "nf4_quantizer.v", "int4_quantizer.v", "int8_quantizer.v",
    "fp8_e4m3_quantizer.v", "fp8_e5m2_quantizer.v", "posit16_quantizer.v",
    "lane_l_precheck.v", "purkinje_thermal_gate.v",
    "gf16_to_fp16.v", "gf16_to_posit16.v", "gf32_to_fp32.v",
]

def check_no_mul_operators(filename: str) -> tuple[bool, list[str]]:
    """Check for forbidden * multiplication operators.

    R-SI-1: Zero * operators in RTL
    Allowed:
    - always @(*) (combinational blocks)
    - * in comments
    - * in case wildcards (case(*))
    - * in string literals

    Forbidden:
    - Actual arithmetic multiplication: variable * variable
    - constant * variable for arithmetic
    """
    issues = []

    try:
        with open(filename, 'r') as f:
            lines = f.readlines()
    except FileNotFoundError:
        return False, [f"File not found: {filename}"]

    for i, line in enumerate(lines, 1):
        original = line
        line = line.strip()

        # Skip empty lines
        if not line:
            continue

        # Remove comments
        if '//' in line:
            line = line.split('//')[0]
            if not line:
                continue

        # Skip always @(*) blocks - this is standard Verilog, not R-SI-1
        if '@(*)' in line or 'always @(*)' in line:
            continue

        # Skip case wildcards
        if 'case' in line and '*' in line:
            continue

        # Check for actual multiplication operators
        # Pattern: identifier or number * identifier or number
        if '*' in line:
            # Skip if it's just in a '*' character or something not arithmetic
            # Look for actual multiplication: operand * operand
            import re

            # Match patterns like: a * b, 2 * c, mant_a * mant_b, etc.
            # But NOT: always @(*), case (*), /*
            mul_pattern = r'[a-zA-Z0-9_]\[\d+:[0-9]+\]\s*\*\s*[a-zA-Z0-9_]|' \
                         r'[a-zA-Z0-9_]+\s*\*\s*[a-zA-Z0-9_]|' \
                         r'[0-9]+[\'bhdBH]\s*\*\s*[a-zA-Z0-9_]'

            matches = re.findall(mul_pattern, line)
            for match in matches:
                # Skip if it's part of a comment (already removed)
                # Skip if it's a bit-width declaration like [1:0]
                if ' [* ' in line or '[*]' in line:
                    continue
                # Report the violation
                issues.append(f"Line {i}: {original.rstrip()}")

    return len(issues) == 0, issues


def check_dsp_cells(build_dir: str = "build") -> dict:
    """Check synthesized netlists for DSP cells."""
    results = {}

    if not os.path.exists(build_dir):
        return {"info": f"Build directory {build_dir} not found. Run synthesis first."}

    for json_file in glob.glob(os.path.join(build_dir, "*.json")):
        with open(json_file, 'r') as f:
            content = f.read()

        has_dsp = "dsp" in content.lower() or "DSP" in content or "DSP_" in content
        basename = os.path.basename(json_file)
        results[basename] = has_dsp

    return results


def main():
    print("=" * 60)
    print("R-SI-1 Compliance Check")
    print("=" * 60)
    print()

    # Check each source file
    all_pass = True
    pass_count = 0
    fail_count = 0

    for src in RTL_SOURCES:
        if os.path.exists(src):
            passed, issues = check_no_mul_operators(src)
            if passed:
                print(f"PASS: {src}")
                pass_count += 1
            else:
                print(f"FAIL: {src}")
                for issue in issues:
                    print(f"  → {issue}")
                fail_count += 1
                all_pass = False
        else:
            print(f"SKIP: {src} (not found)")

    print()
    print(f"Summary: {pass_count} PASS, {fail_count} FAIL")
    print()

    # Check for DSP cells in synthesized netlists
    print("=" * 60)
    print("DSP Cell Check (Synthesized Netlists)")
    print("=" * 60)
    print()

    dsp_results = check_dsp_cells()

    if "info" in dsp_results:
        print(dsp_results["info"])
    else:
        dsp_pass = True
        for netlist, has_dsp in dsp_results.items():
            if has_dsp:
                print(f"FAIL: {netlist} contains DSP cells (R-SI-2 violation)")
                dsp_pass = False
                all_pass = False
            else:
                print(f"PASS: {netlist} - No DSP cells")

        if dsp_pass:
            print()
            print("All netlists are DSP-free (R-SI-2 compliant)")

    print()
    print("=" * 60)

    if all_pass:
        print("✅ R-SI COMPLIANT: All checks passed")
        return 0
    else:
        print("❌ R-SI VIOLATIONS DETECTED")
        return 1


if __name__ == "__main__":
    sys.exit(main())