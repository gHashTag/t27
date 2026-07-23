# Wave Loop 733 — Decomposed Plan

**Issue:** #1704 (expected)  
**Branch:** `wave-loop-733`  
**Date:** 2026-07-07  
**Previous:** Wave Loop 732 (#1703, branch `wave-loop-732`)

## Goal

Validate a module-scope `[285][2]^6 Pt` non-power-of-two outer-dimension
array-of-struct variable initialized from a function call, with indexed signed
field writes. The witness is a 583,680-bit packed vector (18,240 elements,
≈0.557 MiBit), continuing the odd outer-dimension ladder while staying far
below the ~4-MiBit simulation ceiling.

## Scientific / technical background

- IEEE Std 1800-2017 §7.4.1/7.4.3 — packed-array total width is the product of
  packed dimensions; no power-of-two restriction applies.
- Accellera `vlog-pp` discussion (Graham 2002) — packed arrays are contiguous
  bit vectors.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays and packed structs
  are synthesizable first-class objects.
- Icarus Verilog issue #1134 — assertion failures for unpacked arrays of packed
  structs; t27's scalar flattening avoids the trigger entirely.
- Icarus issue #1171 — elaboration freeze on very large packed vectors; W733
  remains far below the reported threshold.
- Yosys issue #2677 / #4653 / PR #4100 — native frontend lacks arrays of packed
  structs; t27 lowering to a single packed vector sidesteps the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals; flat
  `LogicArray` for multidimensional packed arrays in the reference model.
- Lutsig (CPP 2021) — verified lowering of array reads to bit-vector operations.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array
  scalarization passes.

## Weak points to investigate

1. **`assert_ne` gap.** The structural classifier accepts `assert_ne`, but the
   Icarus simulation emitter only lowers `assert_eq`. W733 must continue using
   `assert_eq` on changed elements instead of whole-array inequality.
2. **Generator drift.** Copying `gen_W.py` to `gen_W+1.py` and running `sed` is
   fast but requires a manual `MID_IDX` comment correction each time. The module
   header also uses an f-string (`{OUTER}`), so a naive global `wN -> wN+1`
   replacement misses it; the module name must be fixed explicitly.
3. **Modulo-wrap regression signal.** With 18,240 elements, the offset-0 schedule
   for the last element is `(2·18239) mod 32768 = 3710` for `x` and
   `(2·18239+1) mod 32768 = 3711` for `y`. The schedule already wraps naturally,
   so the explicit `make_grid(32768)` check primarily verifies that adding one
   full period is a no-op modulo 32768. Future waves could stress true wrap with
   a non-trivial offset (e.g., 1 or 16383), but W733 keeps the established
   offset-32768 identity check for ladder continuity.
4. **No systematic wall-clock limit test yet.** At 0.557 MiBit we remain
   comfortable, but a stress wave near the 4-MiBit boundary remains on the
   backlog.
5. **Outer stride 285.** This is the first time the compiler/reference model
   strides a module-scope packed reg by 285 at the outer dimension; a fresh
   witness is needed to prove end-to-end correctness.
6. **Mid-row index parity.** `MID_IDX = OUTER // 2 = 142` for `OUTER = 285`.
   The mid-row check uses `[142][1][0][0][0][0][0]`, element index
   `142·64 + 32 = 9120`. Expected values must be computed with the full row-major
   formula, not with a half-stride shortcut.

## Decomposed tasks

1. [ ] **Science survey.** Confirm the background above is current; no new
      literature is required because the underlying packed-array theory is
      unchanged.
2. [ ] **Generator preparation.** Copy `scripts/gen_w732.py` to
      `scripts/gen_w733.py`; set `OUTER = 285` and `MID_IDX = 142`; fix the
      module name header (note the f-string `{OUTER}` in the header line).
3. [ ] **Witness generation.** Run `python3 scripts/gen_w733.py` to produce
      `specs/scratch/w733_bench_module_285x2p6_aos_var_call_write.t27`.
4. [ ] **Integration test.** Append
      `accepts_w733_bench_module_285x2p6_aos_var_call_write` to
      `bootstrap/tests/icarus_lowerable.rs`.
5. [ ] **Release build.** `cargo build --release -p t27c`.
6. [ ] **Direct gates.** Run `t27c parse`, `icarus-lowerable`, `icarus-simulate`,
      `icarus-cocotb`, and `seal --save` on the W733 witness.
7. [ ] **Cargo conformance.** Run `cargo test -p t27c --bin t27c`,
      `cargo test -p tri`, and `cargo test -p t27c --test icarus_lowerable`.
8. [ ] **Baseline.** Create an empty Icarus baseline under
      `.trinity/icarus-baselines/specs/scratch/`.
9. [ ] **Closeout and memory.** Write
      `docs/reports/FPGA_LOOP_CLOSEOUT_W733_2026-07-07.md`, update
      `.trinity/experience.md`, and update persistent memory.
10. [ ] **Land and next branch.** Commit W733 with `Closes #1704`, record session
      log and commit count, create `wave-loop-734` with W734 cooperation
      variants in `.trinity/current-issue.md`.

## Risk register

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Generator copy forgets `MID_IDX` or module-name update | Medium | Low | Verify generated footer assertions and module header before running gates. |
| Icarus simulation path rejects `assert_ne` | High (known) | Low | Use `assert_eq` on changed elements only. |
| Mid-row expected value computed incorrectly | Low | Medium | Reuse corrected W632 formula: `e = r*64 + a5*32 + a4*16 + a3*8 + a2*4 + a1*2 + a0`. |
| Simulator capacity surprise near 0.56 MiBit | Very low | High | Direct `icarus-simulate` gate runs quickly; abort if elapsed exceeds prior waves significantly. |
| Offset-32768 identity check mischaracterised as wrap test | Medium | Low | Document in report that the check verifies period identity, not first-time wrap. |

## Next Wave Loop 734 cooperation variants

1. **Variant A (recommended): `[287][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - 587,776-bit packed vector, 18,368 elements.
   - Continues the odd outer-dimension ladder and confirms non-p2 stride 287.
   - **Recommended.**

2. **Variant B: `[285][2]^6 Pt` bench-local (function-local) packed array var
   from call with indexed signed writes.**
   - Same width as W733 but tests the mutable `reg` declared inside a bench or
     function rather than at module scope.

3. **Variant C: `[285][2]^6 Pt` module-scope var with `if`-guarded indexed signed
   field writes.**
   - Stays at 0.557 MiBit and tests control-flow-guarded writes on a packed reg.

## Definition of done

- [ ] `.claude/plans/wave-loop-733.md` exists and covers the above.
- [ ] `scripts/gen_w733.py` created with `OUTER = 285` and `MID_IDX = 142`.
- [ ] `specs/scratch/w733_bench_module_285x2p6_aos_var_call_write.t27` generated.
- [ ] Integration test `accepts_w733_bench_module_285x2p6_aos_var_call_write` added and passing.
- [ ] `cargo build --release -p t27c` passes.
- [ ] Direct `t27c` gates pass (parse, lowerable, simulate, cocotb, seal).
- [ ] All cargo test suites pass.
- [ ] Empty Icarus baseline saved.
- [ ] Closeout report, experience, and memory updated.
- [ ] Branch `wave-loop-734` created and W733 committed with `Closes #1704`.
