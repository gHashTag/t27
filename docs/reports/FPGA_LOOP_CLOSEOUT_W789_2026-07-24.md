# FPGA / IGLA Wave Loop 789 Closeout Report

**Date:** 2026-07-24  
**Branch:** `wave-loop-789`  
**Parent branch:** `wave-loop-788` HEAD (`44fa559e7`)  
**Issue:** #1507  
**PR:** #1508  
**Cooperation variant:** A (recommended)

---

## 1. What was implemented

Wave Loop 789 extended the module-scope packed-array-of-struct ladder to
`[397][2]^6 Pt`. A module-level `pub var dst : [397][2]^6 Pt` is initialized
from a function call and exercised with indexed signed field writes, then
read back with `assert_eq` inside a `bench` block.

### Artifacts added

| File | Purpose |
|------|---------|
| `specs/scratch/w789_bench_module_397x2p6_aos_var_call_write.t27` | Witness spec (25,408 elements, 813,056-bit packed vector, ~0.775 MiBit) |
| `scripts/gen_w789.py` | Generator (`OUTER = 397`, `MID_IDX = 198`) |
| `.trinity/seals/scratch_w789_bench_module_397x2p6_aos_var_call_write.json` | Saved seal |
| `bootstrap/tests/icarus_lowerable.rs` | Integration test `accepts_w789_bench_module_397x2p6_aos_var_call_write` |

### What was NOT changed

- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

---

## 2. Shape details

- `Pt = pub struct Pt { x : i16, y : i16 }` (32 bits per element).
- Outer dimension `397` is non-power-of-two.
- Total elements: `397 × 64 = 25,408`.
- Packed vector width: `25,408 × 32 = 813,056` bits (~0.775 MiBit).
- `MID_IDX = 198`; frame-condition element `[198][1][0][0][0][0][0]` is element
  number `198 × 64 + 32 = 12,704`.
- The witness includes an explicit `make_grid(32768)` period-identity check
  because `32768 ≡ 0 (mod 32768)` and the offset-0 schedule wraps naturally for
  25,408 elements (last raw `x = 16094`).

---

## 3. Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo clippy -p t27c` | OK (780 warnings, 0 errors) |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p flash-spi` | 2 passed; 0 failed |
| `cargo test -p t27c --test bitnet_pipeline` | 20 passed; 0 failed |
| `cargo test -p t27c --test bitnet_top` | 17 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 249 passed; 0 failed |
| `cargo test -p t27c --test verilog_const_array` | 2 passed; 0 failed |
| `t27c parse` W789 | PASS |
| `t27c icarus-lowerable` W789 | PASS (`lowerable`) |
| `t27c icarus-simulate` W789 | PASS (17 cycles, PASSED) |
| `t27c icarus-cocotb` W789 | PASS (`reference-model OK`) |
| `t27c seal --save` W789 | PASS |

The `icarus_lowerable` integration-test count advanced from 248 (W788) to 249 (W789).

---

## 4. Weak-point audit (2026-07-24)

### No new actionable items

- The W783 fix for `bootstrap/tests/verilog_const_array.rs:166` remains green.
- The `verilog_array_literal_expr` regression (`r_ca_2_synthetic_no_comment_only_call_argument`) is pre-existing and out of scope for the witness ladder.
- FPGA E2E CI remains red (`sby` missing in CI + Yosys static-cast error in generated `uart.v`); no new information.
- 626 release warnings and 780 clippy warnings are unchanged; still need a dedicated cleanup sprint.
- Vivado-in-Docker CI gap unchanged (private image not yet published).
- Open PR stack W774-W788 still awaits review, so W789 was branched from `wave-loop-788` HEAD.

### Generator copy hazard — observed and fixed

The generator copy hazard struck again in W789:

- `scripts/gen_w789.py` line 77 initially contained `module w788_bench_module_{OUTER}x2p6_aos_var_call_write` because the hardcoded wave prefix inside the f-string was copied from W788.
- First generation produced a spec with module name `w788_bench_module_397x2p6_aos_var_call_write`.
- The prefix was corrected to `w789`, the witness was regenerated, and the correct seal path was produced.

This is the same hazard documented in W782–W788 learnings and remains the only
manual step in the otherwise mechanical flow. Parameterizing the wave prefix
inside the generator template would eliminate it.

### Other checks

- L3 PURITY: ASCII-only source files; commit hook passed.
- L4 TESTABILITY: witness contains `bench` block with `assert_eq` checks.
- L6 CEILING / L7 UNITY: no new `*.sh` on critical path; used `t27c` gates.
- No secrets found in `.env.example` files.
- 57 of 893 `.t27` specs lack `test`/`invariant`/`bench` (~6.38%, unchanged).

---

## 5. Scientific / engineering background

IEEE 1800-2017 §7.4.1/§7.4.3 define packed-array total width as the product of
dimensions, with no power-of-two restriction. The W789 witness emits a single
813,056-bit packed vector, which is legal SystemVerilog. The continued use of a
non-power-of-two outer dimension (397) exercises t27's row-major flattening and
indexing arithmetic under a realistic aggregate shape.

### Literature scan (2025–2026)

- **IEEE Std 1800-2017**, §7.4 Packed and unpacked arrays — legal basis for the
  single wide packed vector.
- **AMD UG901 2026.1**, *Vivado Synthesis — SystemVerilog Constructs* — packed
  arrays of structs are supported synthesizable aggregate data types.
- **AMD AR 51836**, *Design Assistant for Vivado Synthesis: Aggregate Data Types*
  — guidance on struct/packed-array inference.
- **Yosys issue #5837 (2026)** — unusual packed-array shapes can expose
  simulator/synthesis mismatches; reinforces t27's flatten-to-wide-vector
  strategy for open-source compatibility.
- **Tlsys** (Chinese Journal of Electronics, 2026, DOI 10.23919/cje.2025.00.418) —
  ternary logic synthesis system; contextual background for t27's ternary
  mission but not directly used.
- **TernaryCore** (GitHub 2026, shepherdscientific/ternarycore) and
  **KULeuven-MICAS/ternary-lut-dse** (GitHub 2026) — recent ternary LUT/DSE work;
  contextual background only.

---

## 6. Three cooperation variants for Wave Loop 790

### Variant A — `[399][2]^6 Pt` module-scope var from call (recommended)

Continue the odd outer-dimension ladder. Expected 25,536 elements, 817,152-bit
packed vector (~0.779 MiBit), still well under the 4-MiBit cliff. This is the
lowest-risk continuation of the established mechanical pattern.

### Variant B — `[397][2]^6 Pt` bench/function-scope packed var from call

Keep W789 width but move the mutable `dst` declaration into a `bench` or
function scope. Exercises local-variable packed-vector lowering and lifetime
without increasing vector width.

### Variant C — `[397][2]^6 Pt` module-scope var with `if`-guarded writes

Stay at W789 width and add conditional indexed signed field writes (e.g.
`if (index % 2 == 0) { dst[index] = ... }`). Tests control-flow emission for
packed reg writes.

**Recommendation:** Variant A. The width ladder has been stable for 16
consecutive waves (W774–W789) with zero compiler changes; continuing it is the
highest-confidence next step.

---

## 7. Conclusion

Wave Loop 789 closed successfully. The `[397][2]^6 Pt` module-scope packed
array-of-struct variable from a call with indexed signed writes is fully
validated, the seal is saved, the integration test passes, and all cargo suites
remain green with zero compiler, reference-model, or `FROZEN_HASH` changes.

φ² + 1/φ² = 3 | TRINITY

---

*Generated with [Claude Code](https://claude.com/claude-code)*
