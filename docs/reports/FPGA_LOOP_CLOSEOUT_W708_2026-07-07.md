# FPGA LOOP Closeout — Wave Loop 708

**Date:** 2026-07-07
**Issue:** #1679
**Branch:** `wave-loop-708`
**Next branch:** `wave-loop-709`
**Variant:** A — module-scope `[235][2]^6 Pt` array-of-struct variable, initialized
from a function call, with indexed signed field writes and read-back.

---

## Summary

Wave Loop 708 extended the module-scope packed array-of-struct ladder to a
non-power-of-two outer dimension of **235**. The witness
`specs/scratch/w708_bench_module_235x2p6_aos_var_call_write.t27` declares a
479,488-bit (≈0.457 MiBit) mutable packed `reg`, initializes it from
`make_grid(0)`, and exercises signed indexed field writes on the first and last
elements, with frame-condition checks on an untouched mid-element.

No compiler or reference-model changes were required. All targeted gates and
`cargo test` suites passed.

---

## Technical details

- `pub struct Pt { x : i16, y : i16 }`
- `pub fn make_grid(offset : u16) -> [235][2][2][2][2][2][2] Pt`
- `pub const expected : [235][2][2][2][2][2][2] Pt = make_grid(0);`
- `pub var dst : [235][2][2][2][2][2][2] Pt = make_grid(0);`
- Element count: **15,040**
- Packed width: **479,488 bits** (59,936 bytes)
- Row-major LSB-first element index:
  `[r][a5][a4][a3][a2][a1][a0]` → `e = r·64 + a5·32 + a4·16 + a3·8 + a2·4 + a1·2 + a0`
- Leaf schedule: `x = (2·e + offset) mod 32768`, `y = (2·e + offset + 1) mod 32768`
- Mid-row index used for frame-condition check:
  `[117][1][0][0][0][0][0]` → `e = 117·64 + 32 = 7520` → `x = 15040`, `y = 15041`
- Last element index: `[234][1][1][1][1][1][1]` → `e = 15039` → `x = 30078`, `y = 30079`

Because the offset-0 schedule never wraps at this element count, the test retains
an explicit `make_grid(32768)` call to preserve the modulo-wrap regression signal.

---

## Files changed / added

- `scripts/gen_w708.py` — generator for the witness.
- `specs/scratch/w708_bench_module_235x2p6_aos_var_call_write.t27` — witness spec
  (~1,033 KB, 44,711 lines).
- `.trinity/seals/scratch_w708_bench_module_235x2p6_aos_var_call_write.json` — seal.
- `.trinity/icarus-baselines/specs/scratch/w708_bench_module_235x2p6_aos_var_call_write.json` —
  empty Icarus baseline.
- `bootstrap/tests/icarus_lowerable.rs` — added
  `accepts_w708_bench_module_235x2p6_aos_var_call_write`.
- `.trinity/current-issue.md` — updated for W708.
- `.claude/plans/wave-loop-708.md` — decomposed plan.
- `.trinity/experience.md` — appended W708 learnings.
- `docs/reports/FPGA_LOOP_CLOSEOUT_W708_2026-07-07.md` — this report.

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
| `cargo test -p t27c --test icarus_lowerable` | 168 passed; 0 failed |
| `t27c parse` W708 | PASS |
| `t27c icarus-lowerable` W708 | `lowerable` |
| `t27c icarus-simulate` W708 | PASSED (17 cycles) |
| `t27c icarus-cocotb` W708 | reference-model OK |
| `t27c seal --save` W708 | saved |

---

## Scientific / engineering background

- IEEE Std 1800-2017 §7.4.1/7.4.3: packed-array width is the product of packed
  dimensions, with no power-of-two restriction. The 479,488-bit vector emitted
  for W708 is legal SystemVerilog.
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
  with SystemVerilog benches via VCD probe comparison; the W708 cocotb check
  reuses this pattern.
- Lutsig (CPP 2021) and CIRCT `HWLegalizeModules.cpp`: flattening nested arrays
  to wide packed vectors is a verified compiler discipline, even for
  non-power-of-two outer dimensions.

---

## Risk register (retrospective)

| Risk | Likelihood | Impact | Outcome |
|------|------------|--------|---------|
| Outer dimension 235 breaks layout math | Low | High | Did not occur; cocotb reference model agreed with simulation. |
| Parse time blows up repository sweep | High | Low | Targeted gates used; full `tri test --fast` not required. |
| Icarus simulation path rejects witness | Low | High | Did not occur. |
| `assert_ne` confusion resurfaces | Low | Low | Avoided by using `assert_eq` on changed elements. |

---

## Next Wave Loop 709 cooperation variants

1. **Variant A (recommended):** `[237][2]^6 Pt` module-scope var from a call with
   indexed signed writes — 483,200 bits, 15,168 elements. Continue the ladder.
2. **Variant B:** `[235][2]^6 Pt` bench-local packed array var from a call with
   indexed signed writes — same size, different scope.
3. **Variant C:** `[235][2]^6 Pt` module-scope var with `if`-guarded indexed
   signed field writes — same size, control-flow coverage.

Recommended: **Variant A**.

---

## FROZEN_HASH

`bootstrap/stage0/FROZEN_HASH` remained unchanged at
`68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.

---

## Notes for the next wave

- The odd outer-dimension ladder (211, 213, 215, 217, 219, 221, 223, 225, 227,
  229, 231, 233, 235) is now established and dimension-agnostic; W709 can stride
  directly to 237 without compiler risk assessment.
- At ~0.46 MiBit the witness is still far below the ~4-MiBit Icarus/Yosys comfort
  threshold, but compile time is the variable to watch; record wall-clock time
  for W709.
- Continue using multi-line W584 brace style and the `cp` + `sed` generator
  workflow.
