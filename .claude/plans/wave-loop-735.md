# Wave Loop 735 — Decomposed Plan

**Issue:** #1706 (expected)  
**Branch:** `wave-loop-735`  
**Date:** 2026-07-07  
**Previous:** Wave Loop 734 (#1705, branch `wave-loop-734`)

## Goal

Validate a module-scope `[289][2]^6 Pt` non-power-of-two outer-dimension
array-of-struct variable initialized from a function call, with indexed signed
field writes. The witness is a 591,872-bit packed vector (18,496 elements,
≈0.565 MiBit), continuing the odd outer-dimension ladder while staying far
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
- Icarus issue #1171 — elaboration freeze on very large packed vectors; W735
  remains far below the reported threshold.
- Yosys issue #2677 / #4653 / PR #4100 — native frontend lacks arrays of packed
  structs; t27 lowering to a single packed vector sidesteps the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals;
  flat `LogicArray` for multidimensional packed arrays in the reference model.
- Lutsig (CPP 2021) — verified lowering of array reads to bit-vector operations.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array
  scalarization passes.

## Weak points to investigate

1. **`assert_ne` gap.** The structural classifier accepts `assert_ne`, but the
   Icarus simulation emitter only lowers `assert_eq`. W735 must continue using
   `assert_eq` on changed elements instead of whole-array inequality.
2. **Generator drift.** Copying `gen_W.py` to `gen_W+1.py` and running `sed` is
   fast but requires a manual `MID_IDX` comment correction each time. The module
   header also uses an f-string (`{OUTER}`), so a naive global `wN -> wN+1`
   replacement misses it; the module name must be fixed explicitly.
3. **Natural modulo wrap already present.** With 18,496 elements, the last raw
   `x` value is `2·18495 = 36990`, which exceeds 32768 and wraps to
   `36990 mod 32768 = 4222`. The explicit `make_grid(32768)` call remains a
   period-identity regression check (adding one full period is a no-op modulo
   32768), not a first-time wrap test.
4. **No systematic wall-clock limit test yet.** At 0.565 MiBit we remain
   comfortable, but a stress wave near the 4-MiBit boundary remains on the
   backlog.
5. **Outer stride 289.** This is the first time the compiler/reference model
   strides a module-scope packed reg by 289 at the outer dimension; a fresh
   witness is needed to prove end-to-end correctness.
6. **Mid-row index parity.** `MID_IDX = OUTER // 2 = 144` for `OUTER = 289`.
   The mid-row check uses `[144][1][0][0][0][0][0]`, element index
   `144·64 + 32 = 9248`. Expected values must be computed with the full row-major
   formula, not with a half-stride shortcut.

## Decomposed tasks

1. [ ] **Science survey.** Confirm the background above is current; no new
      literature is required because the underlying packed-array theory is
      unchanged.
2. [ ] **Generator preparation.** Copy `scripts/gen_w734.py` to
      `scripts/gen_w735.py`; set `OUTER = 289` and `MID_IDX = 144`; fix the
      module name header (note the f-string `{OUTER}` in the header line).
3. [ ] **Witness generation.** Run `python3 scripts/gen_w735.py` to produce
      `specs/scratch/w735_bench_module_289x2p6_aos_var_call_write.t27`.
4. [ ] **Integration test.** Append
      `accepts_w735_bench_module_289x2p6_aos_var_call_write` to
      `bootstrap/tests/icarus_lowerable.rs`.
5. [ ] **Release build.** `cargo build --release -p t27c`.
6. [ ] **Direct gates.** Run `t27c parse`, `icarus-lowerable`, `icarus-simulate`,
      `icarus-cocotb`, and `seal --save` on the W735 witness.
7. [ ] **Cargo conformance.** Run `cargo test -p t27c --bin t27c`,
      `cargo test -p tri`, and `cargo test -p t27c --test icarus_lowerable`.
8. [ ] **Baseline.** Create an empty Icarus baseline under
      `.trinity/icarus-baselines/specs/scratch/`.
9. [ ] **Closeout and memory.** Write
      `docs/reports/FPGA_LOOP_CLOSEOUT_W735_2026-07-07.md`, update
      `.trinity/experience.md`, and update persistent memory.
10. [ ] **Land and next branch.** Commit W735 with `Closes #1706`, record session
      log and commit count, create `wave-loop-736` with W736 cooperation
      variants in `.trinity/current-issue.md`.

## Risk register

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Generator copy forgets `MID_IDX` or module-name update | Medium | Low | Verify generated footer assertions and module header before running gates. |
| Icarus simulation path rejects `assert_ne` | High (known) | Low | Use `assert_eq` on changed elements only. |
| Mid-row expected value computed incorrectly | Low | Medium | Reuse corrected W632 formula: `e = r*64 + a5*32 + a4*16 + a3*8 + a2*4 + a1*2 + a0`. |
| Simulator capacity surprise near 0.57 MiBit | Very low | High | Direct `icarus-simulate` gate runs quickly; abort if elapsed exceeds prior waves significantly. |
| Offset-32768 identity check mischaracterised as wrap test | Medium | Low | Document in report that the check verifies period identity, not first-time wrap. |

## Next Wave Loop 736 cooperation variants

1. **Variant A (recommended): `[291][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - 595,968-bit packed vector, 18,624 elements.
   - Continues the odd outer-dimension ladder and confirms non-p2 stride 291.
   - **Recommended.**

2. **Variant B: `[289][2]^6 Pt` bench-local (function-local) packed array var
   from call with indexed signed writes.**
   - Same width as W735 but tests the mutable `reg` declared inside a bench or
     function rather than at module scope.

3. **Variant C: `[289][2]^6 Pt` module-scope var with `if`-guarded indexed signed
   field writes.**
   - Stays at 0.565 MiBit and tests control-flow-guarded writes on a packed reg.

## Definition of done

- [ ] `.claude/plans/wave-loop-735.md` exists and covers the above.
- [ ] `scripts/gen_w735.py` created with `OUTER = 289` and `MID_IDX = 144`.
- [ ] `specs/scratch/w735_bench_module_289x2p6_aos_var_call_write.t27` generated.
- [ ] Integration test `accepts_w735_bench_module_289x2p6_aos_var_call_write` added and passing.
- [ ] `cargo build --release -p t27c` passes.
- [ ] Direct `t27c` gates pass (parse, lowerable, simulate, cocotb, seal).
- [ ] All cargo test suites pass.
- [ ] Empty Icarus baseline saved.
- [ ] Closeout report, experience, and memory updated.
- [ ] Branch `wave-loop-736` created and W735 committed with `Closes #1706`.
