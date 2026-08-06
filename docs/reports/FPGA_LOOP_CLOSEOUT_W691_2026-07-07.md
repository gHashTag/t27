# FPGA LOOP Closeout — Wave Loop 691

**Date:** 2026-07-07
**Issue:** #1662
**Branch:** `wave-loop-691`
**Next branch:** `wave-loop-692`
**Variant:** A — module-scope `[201][2]^6 Pt` array-of-struct variable, initialized
from a function call, with indexed signed field writes and read-back.

---

## Summary

Wave Loop 691 extended the module-scope packed array-of-struct ladder to a
non-power-of-two outer dimension of **201**. The witness
`specs/scratch/w691_bench_module_201x2p6_aos_var_call_write.t27` declares a
411,648-bit (≈0.392 MiBit) mutable packed `reg`, initializes it from
`make_grid(0)`, and exercises signed indexed field writes on the first and last
elements, with frame-condition checks on an untouched mid-element.

No compiler or reference-model changes were required. All targeted gates and
`cargo test` suites passed.

---

## Technical details

- `pub struct Pt { x : i16, y : i16 }`
- `pub fn make_grid(offset : u16) -> [201][2][2][2][2][2][2] Pt`
- `pub const expected : [201][2][2][2][2][2][2] Pt = make_grid(0);`
- `pub var dst : [201][2][2][2][2][2][2] Pt = make_grid(0);`
- Element count: **12,864**
- Packed width: **411,648 bits** (51,456 bytes)
- Row-major LSB-first element index:
  `[r][a5][a4][a3][a2][a1][a0]` → `e = r·64 + a5·32 + a4·16 + a3·8 + a2·4 + a1·2 + a0`
- Leaf schedule: `x = (2·e + offset) mod 32768`, `y = (2·e + offset + 1) mod 32768`
- Mid-row index used for frame-condition check:
  `[100][1][0][0][0][0][0]` → `e = 100·64 + 32 = 6432` → `x = 12864`, `y = 12865`
- Last element index: `[200][1][1][1][1][1][1]` → `e = 12863` → `x = 25726`, `y = 25727`

Because the offset-0 schedule never wraps at this element count, the test retains
an explicit `make_grid(32768)` call to preserve the modulo-wrap regression signal.

---

## Files changed / added

- `scripts/gen_w691.py` — generator for the witness.
- `specs/scratch/w691_bench_module_201x2p6_aos_var_call_write.t27` — witness spec
  (~882 KB, ~38,251 lines).
- `.trinity/seals/scratch_w691_bench_module_201x2p6_aos_var_call_write.json` — seal.
- `.trinity/icarus-baselines/specs/scratch/w691_bench_module_201x2p6_aos_var_call_write.json` —
  empty Icarus baseline.
- `bootstrap/tests/icarus_lowerable.rs` — added
  `accepts_w691_bench_module_201x2p6_aos_var_call_write`.
- `.trinity/current-issue.md` — updated for W691.
- `.claude/plans/wave-loop-691.md` — decomposed plan.
- `.trinity/experience.md` — appended W691 learnings.
- `docs/reports/FPGA_LOOP_CLOSEOUT_W691_2026-07-07.md` — this report.

No changes to:

- `bootstrap/src/compiler.rs`
- `bootstrap/stage0/FROZEN_HASH`
- `scripts/cocotb_ref_model.py`

---

## Validation

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 151 passed; 0 failed |
| `t27c parse` W691 | PASS |
| `t27c icarus-lowerable` W691 | `lowerable` |
| `t27c icarus-simulate` W691 | PASSED (17 cycles) |
| `t27c icarus-cocotb` W691 | reference-model OK |
| `t27c seal --save` W691 | saved |

`FROZEN_HASH` remains `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.

---

## Scientific / engineering background

- IEEE Std 1800-2017 §7.4.1/7.4.3 — packed-array width is the product of packed
  dimensions; no power-of-two restriction applies.
- Accellera vlog-pp discussion (Graham 2002) — packed arrays as contiguous bit
  vectors.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays/structs are first-class
  synthesizable objects.
- Icarus issue #1134 — unpacked arrays of packed structs trigger assertion
  failures; t27 flattening avoids this entirely.
- Icarus issue #1171 — very large packed vectors can freeze elaboration; W691
  stays well below the reported threshold.
- Yosys docs / issue #2677 / #4653 / PR #4100 — multidimensional packed arrays
  supported, arrays of packed structs unsupported; t27 flattening avoids the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals; flat
  `LogicArray` for multidimensional packed arrays in the reference model.
- Lutsig (CPP 2021) — verified array-read lowering.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array
  scalarization.

---

## Weak points / risks closed

1. **Outer dimension 201 correctness.** Resolved by cocotb cross-check and
   explicit expected-value computation using the W632 element-index formula.
2. **Parser/scanner limits at ~38 k lines.** Resolved by continuing multi-line
   W584 brace-style literals; parse gate passes.
3. **Modulo-wrap regression signal.** Resolved by retaining `make_grid(32768)`
   explicit wrap check.
4. **`assert_ne` simulation gap.** Resolved by using `assert_eq` on changed
   elements in the bench block.
5. **Simulator capacity.** Resolved: 17 cycles, no elaboration freeze.

---

## Next Wave Loop 692 cooperation variants

1. **Variant A (recommended) — `[203][2]^6 Pt` module-scope var from a call with
   indexed signed writes.**
   415,744-bit packed vector, 13,056 elements, non-power-of-two outer dimension 203.
   Continues the odd outer-dimension ladder. **Recommended.**

2. **Variant B — `[201][2]^6 Pt` bench-local (function-local) packed array var
   from a call with indexed signed writes.**
   411,648-bit packed vector, 12,864 elements. Tests the same non-p2 outer
   dimension in a bench/function scope rather than module scope.

3. **Variant C — `[201][2]^6 Pt` module-scope var with `if`-guarded indexed signed
   field writes.**
   Stays at 0.392 MiBit and tests control-flow guarded indexed writes on a packed
   `reg`.

---

## Conclusion

Wave Loop 691 closes #1662. The module-scope packed AoS odd outer-dimension ladder
now reaches 201 with zero compiler changes. Branch `wave-loop-692` is ready for
the next wave.

`Closes #1662`
