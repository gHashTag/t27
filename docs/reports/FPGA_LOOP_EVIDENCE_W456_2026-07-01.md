# FPGA Loop Evidence — Wave Loop 456 (2026-07-01)

**Issue:** #1427  
**Branch:** `wave-loop-456`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What was done

Wave Loop 456 executed **Variant B** from
`docs/reports/FPGA_LOOP_COOPERATION_W456_2026-07-01.md`, narrowed to a single
compiler-backend hardening step: **ROM read-only enforcement**.

After W455 cleared the 7 residual `gen-verilog` yosys smoke failures, module-level
`const [N]T` arrays were synthesizable but had no semantic guard against run-time
writes. W456 adds that guard in the typechecker.

### Files changed

- `bootstrap/src/compiler.rs`
  - `typecheck_ast` / `check_stmt` now detects `StmtAssign` whose LHS is an
    `ExprIndex` into a symbol whose `is_mutable == false` (i.e., a module-level
    `const` array) and emits a typecheck **error**.
  - Existing immutable scalar assignment (`x = ...` where `x` is `const` or
    immutable `let`) remains a warning.
  - Added `tests_w456_rom_readonly` unit-test module.

- `specs/scratch/w456_rom_readonly.t27`
  - New regression spec with a module-level `const [4]u16` ROM and read-only
    lookup functions.

- `.trinity/seals/scratch_w456_rom_readonly.json`
  - New seal for the regression spec.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Added W456 competitor boundary section.

---

## Verification results

| Check | Result |
|---|---|
| `cargo test -p t27c --bin t27c tests_w456_rom_readonly` | **PASS** (2/2) |
| `t27c gen-verilog specs/scratch/w456_rom_readonly.t27` + `yosys read_verilog -sv; synth -top w456_rom_readonly` | **PASS** |
| `./scripts/tri test --json /tmp/tri_test_w456.json` | **ALL TESTS PASSED** |
| Parse | 577 passed, 0 failed |
| Typecheck | 577 passed, 0 failed |
| Gen Zig | 577 passed, 0 failed |
| Gen Rust | 577 passed, 0 failed |
| Gen Verilog | 577 passed, 0 failed |
| Gen Verilog Yosys Smoke | **57 passed, 0 failed** |
| FPGA Board-Less Smoke Gate | **OK** (`phases green`) |
| FPGA Standalone Lake-Package Build | **OK** (`elapsed_ms ~245624`) |
| Gen C | 577 passed, 0 failed |
| Seal Verify | 577 passed, 0 failed |
| Fixed Point | 0 divergences |
| **TOTAL FAILURES** | **0** |
| `ACCEPTABLE` | **yes** |

---

## Not done

- **Physical bench execution:** still blocked. `dlc10 idcode` reports
  "DLC10 cable not found (VID=0x03FD)", P12 is unwired, and no automated cold-POR
  relay gate exists.
- **Remaining Variant B compiler work:** RAM style pragmas, module-level array
  parameter passing, and yosys warning hygiene are deferred to Wave Loop 457.
- **Formal boot-evidence expansion:** no new Lean 4 theorems this wave.

---

*φ² + φ⁻² = 3 | TRINITY*
