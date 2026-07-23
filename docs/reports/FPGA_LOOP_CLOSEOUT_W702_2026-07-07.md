# FPGA LOOP Closeout — Wave Loop 702

**Date:** 2026-07-07
**Issue:** #1673
**Branch:** `wave-loop-702`
**Next branch:** `wave-loop-703`
**Variant:** A — module-scope `[223][2]^6 Pt` array-of-struct variable, initialized
from a function call, with indexed signed field writes and read-back.

---

## Summary

Wave Loop 702 extended the module-scope packed array-of-struct ladder to a
non-power-of-two outer dimension of **223**. The witness
`specs/scratch/w702_bench_module_223x2p6_aos_var_call_write.t27` declares a
456,704-bit (≈0.436 MiBit) mutable packed `reg`, initializes it from
`make_grid(0)`, and exercises signed indexed field writes on the first and last
elements, with frame-condition checks on an untouched mid-element.

No compiler or reference-model changes were required. All targeted gates and
`cargo test` suites passed.

---

## Technical details

- `pub struct Pt { x : i16, y : i16 }`
- `pub fn make_grid(offset : u16) -> [223][2][2][2][2][2][2] Pt`
- `pub const expected : [223][2][2][2][2][2][2] Pt = make_grid(0);`
- `pub var dst : [223][2][2][2][2][2][2] Pt = make_grid(0);`
- Element count: **14,272**
- Packed width: **456,704 bits** (56,588 bytes)
- Row-major LSB-first element index:
  `[r][a5][a4][a3][a2][a1][a0]` → `e = r·64 + a5·32 + a4·16 + a3·8 + a2·4 + a1·2 + a0`
- Leaf schedule: `x = (2·e + offset) mod 32768`, `y = (2·e + offset + 1) mod 32768`
- Mid-row index used for frame-condition check:
  `[111][1][0][0][0][0][0]` → `e = 111·64 + 32 = 7136` → `x = 14272`, `y = 14273`
- Last element index: `[222][1][1][1][1][1][1]` → `e = 14271` → `x = 28542`, `y = 28543`

Because the offset-0 schedule never wraps at this element count, the test retains
an explicit `make_grid(32768)` call to preserve the modulo-wrap regression signal.

---

## Files changed / added

- `scripts/gen_w702.py` — generator for the witness.
- `specs/scratch/w702_bench_module_223x2p6_aos_var_call_write.t27` — witness spec
  (~980 KB, 42,431 lines).
- `.trinity/seals/scratch_w702_bench_module_223x2p6_aos_var_call_write.json` — seal.
- `.trinity/icarus-baselines/specs/scratch/w702_bench_module_223x2p6_aos_var_call_write.json` —
  empty Icarus baseline.
- `bootstrap/tests/icarus_lowerable.rs` — added
  `accepts_w702_bench_module_223x2p6_aos_var_call_write`.
- `.trinity/current-issue.md` — updated for W702.
- `.claude/plans/wave-loop-702.md` — decomposed plan.
- `.trinity/experience.md` — appended W702 learnings.
- `docs/reports/FPGA_LOOP_CLOSEOUT_W702_2026-07-07.md` — this report.

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
| `cargo test -p t27c --test icarus_lowerable` | 162 passed; 0 failed |
| `t27c parse` W702 | PASS |
| `t27c icarus-lowerable` W702 | `lowerable` |
| `t27c icarus-simulate` W702 | PASSED (17 cycles) |
| `t27c icarus-cocotb` W702 | reference-model OK |
| `t27c seal --save` W702 | saved |

---

## Scientific / engineering background

- IEEE Std 1800-2017 §7.4.1/7.4.3: packed-array width is the product of packed
  dimensions, with no power-of-two restriction. The 456,704-bit vector emitted
  for W702 is legal SystemVerilog.
- Accellera vlog-pp discussion (Graham 2002): packed arrays are contiguous bit
  vectors; t27's row-major flattening matches this model.
- Sutherland, "Synthesizable SystemVerilog": packed arrays and packed structs are
  first-class synthesizable objects.
- Icarus issue #1134 / #1171: unpacked arrays of packed structs and very wide
  packed vectors can trigger elaboration problems; t27's scalar flattening and
  staying under the reported threshold avoid both.
- Yosys issues #2677 / #4653 / PR #4100: native arrays-of-structs are not
  supported; t27's packed-vector lowering sidesteps the gap.
- cocotb PR #3608 / discussion #2933: Python reference models integrate cleanly
  with SystemVerilog benches via VCD probe comparison; the W702 cocotb check
  reuses this pattern.
- Lutsig (CPP 2021) and CIRCT `HWLegalizeModules.cpp`: flattening nested arrays
  to wide packed vectors is a verified compiler discipline, even for
  non-power-of-two outer dimensions.

---

## Risk register (retrospective)

| Risk | Likelihood | Impact | Outcome |
|------|------------|--------|---------|
| Outer dimension 223 breaks layout math | Low | High | Did not occur; cocotb reference model agreed with simulation. |
| Parse time blows up repository sweep | High | Low | Targeted gates used; full `tri test --fast` not required. |
| Icarus simulation path rejects witness | Low | High | Did not occur. |
| `assert_ne` confusion resurfaces | Low | Low | Avoided by using `assert_eq` on changed elements. |

---

## Next Wave Loop 703 cooperation variants

1. **Variant A (recommended):** `[225][2]^6 Pt` module-scope var from a call with
   indexed signed writes — 460,800 bits, 14,400 elements. Continue the ladder.
2. **Variant B:** `[223][2]^6 Pt` bench-local packed array var from a call with
   indexed signed writes — same size, different scope.
3. **Variant C:** `[223][2]^6 Pt` module-scope var with `if`-guarded indexed
   signed field writes — same size, control-flow coverage.

Recommended: **Variant A**.

---

## FROZEN_HASH

`bootstrap/stage0/FROZEN_HASH` remained unchanged at
`68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.

---

## Notes for the next wave

- The odd outer-dimension ladder (211, 213, 215, 217, 219, 221, 223) is now
  established and dimension-agnostic; W703 can stride directly to 225 without
  compiler risk assessment.
- At ~0.44 MiBit the witness is still far below the ~4-MiBit Icarus/Yosys comfort
  threshold, but compile time is the variable to watch; record wall-clock time
  for W703.
- Continue using multi-line W584 brace style and the `cp` + `sed` generator
  workflow.
