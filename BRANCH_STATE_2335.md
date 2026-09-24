# Branch State Documentation for gHashTag/t27#2335

## Issue Summary
This document records the state of undefined symbols on branch `feat/rename-tef-to-tnf` at commit `592ba4c2da1becdad87e7861985890f7a3841ba1`, as documented in issue #2335.

## Symbols Previously Missing
According to issue #2335, the following six symbols were unresolvable on the branch `feat/rename-tef-to-tnf`:

| Symbol | Use sites in `cli/tri/src/fpga.rs` | Outside `#[cfg(test)]` | Status on queen-2335 |
|--------|-----------------------------------|------------------------|---------------------|
| `verify_lean` (function) | 6181, 6278, 9681, 9689, 9696, 9722, 9749, 9776 | 2 | ✅ PRESENT |
| `pvt_context_inside_envelope` | 2728, 6190, 6291 | 3 | ✅ PRESENT |
| `synthetic_pvt_context` | 6092, 7151 | 1 | ✅ PRESENT |
| `cclk_period_ns` | 6102 | 1 | ✅ PRESENT |
| `SmokeGateReport` | 6063 | 1 | ✅ PRESENT |
| `assert_report_superset` | 10060 | 0 | ✅ PRESENT |

## Verification Results
All six symbols mentioned in issue #2335 are **present** in the current `queen-2335` branch.

### Symbol Locations in Current Branch
- **`SmokeGateReport` struct**: `cli/tri/src/fpga.rs:2732`
- **`synthetic_pvt_context` function**: `cli/tri/src/fpga.rs:2753` 
- **`pvt_context_inside_envelope` function**: `cli/tri/src/fpga.rs:2765`
- **`cclk_period_ns` function**: `cli/tri/src/fpga.rs:2774`
- **`verify_lean` function**: `cli/tri/src/fpga.rs:2789`
- **`assert_report_superset` function**: `cli/tri/src/fpga.rs:10623` (in test module)

## Key Observations
1. **Complete Recovery**: All six symbols that were reportedly missing from branch `feat/rename-tef-to-tnf` are present in the current `queen-2335` branch.

2. **Contiguous Block**: Five of the six symbols (`SmokeGateReport`, `synthetic_pvt_context`, `pvt_context_inside_envelope`, `cclk_period_ns`, `verify_lean`) are located in a contiguous block at lines 2730-2787 in `cli/tri/src/fpga.rs`, suggesting they were part of a single missing section.

3. **Test Module Location**: `assert_report_superset` is located in the test module (`fpga.rs:10623+`), which aligns with the issue's note that it had 0 use sites outside `#[cfg(test)]`.

4. **SmokeGateReport Accessibility**: The `SmokeGateReport` struct, while present, was noted in the issue as potentially inaccessible due to lack of `bootstrap` dependency in `cli/tri/Cargo.toml`. However, the struct definition itself is available.

## Conclusion
The branch state documented in issue #2335 appears to be recovered in the current `queen-2335` branch. All previously undefined symbols now have definitions present in the codebase.

## Files Verified
- `bootstrap/src/suite.rs`: Contains `SmokeGateReport` definition at line 1601
- `cli/tri/src/fpga.rs`: Contains all six symbol definitions

## T27 Compiler Status
- `compiler/parser/parser.t27`: Parse error fixed, compilation successful
- Other `.t27` files: Compilation status varies, but core symbols are present

Generated on: 2025-06-17
Branch: queen-2335
Commit: $(git rev-parse HEAD)