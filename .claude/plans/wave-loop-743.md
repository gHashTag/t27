# Wave Loop 743 — Decomposed Plan

**Issue:** #1714 (expected)
**Branch:** `wave-loop-743`
**Date:** 2026-07-22
**Previous:** Wave Loop 742 (#1713, branch `wave-loop-742`)

## Goal

Validate a module-scope `[305][2]^6 Pt` non-power-of-two outer-dimension
array-of-struct variable initialized from a function call, with indexed signed
field writes. The witness is a 624,640-bit packed vector (19,520 elements,
~0.596 MiBit), continuing the odd outer-dimension ladder while staying far below
the ~4-MiBit simulation ceiling.

At the same time, use the closeout as a checkpoint to audit repository weak
points, scan recent scientific literature, and prepare a decomposed backlog for
Wave Loop 744 and the next ring.

## Scientific / technical background

- IEEE Std 1800-2017 §7.4.1/7.4.3 — packed-array total width is the product of
  packed dimensions; no power-of-two restriction applies.
- Accellera `vlog-pp` discussion (Graham 2002) — packed arrays are contiguous
  bit vectors.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays and packed structs
  are synthesizable first-class objects.
- Icarus Verilog issue #1134 — assertion failures for unpacked arrays of packed
  structs; t27's scalar flattening avoids the trigger entirely.
- Icarus issue #1171 — elaboration freeze on very large packed vectors; W743
  remains far below the reported threshold.
- Yosys issue #2677 / #4653 / PR #4100 — native frontend lacks arrays of packed
  structs; t27 lowering to a single packed vector sidesteps the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals;
  flat `LogicArray` for multidimensional packed arrays in the reference model.
- Lutsig (CPP 2021) — verified lowering of array reads to bit-vector operations.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array
  scalarization passes.

## Recent literature scan (2024–2025)

Ternary and multi-valued hardware work is active in three lanes relevant to
Trinity's roadmap:

1. **Native balanced-ternary processors:** VTX1 (GitHub 2025) and 5500FP
   (Zenodo 2026) implement full balanced-ternary CPUs/SoCs on FPGA, with planned
   SkyWater 130nm tape-out for VTX1.
2. **Ternary-weight neural-network accelerators:** TerEffic, TeLLMe, and
   Trinity B002 exploit {-1,0,+1} quantization for LLM inference on FPGA,
   achieving large efficiency gains by avoiding DSPs and fitting weights in
   on-chip SRAM.
3. **Multi-valued logic synthesis frameworks:** Tlsys (CJE 2026) and MRCS/Bos
   (2024) raise the abstraction level above manual netlist design, synthesizing
   ternary RTL to CNFET or CMOS gate-level netlists. KULeuven's
   `ternary-lut-dse` provides a Chisel generator for ternary LLM accelerators.
4. **Formal-methods lineage:** Bryant & Seger's Symbolic Trajectory Evaluation
   (1991/2005) and Rosenmann's MVL extension (2015) give a formal verification
   foundation for ternary/multi-valued hardware models.

Sources are listed in `docs/reports/FPGA_LOOP_CLOSEOUT_W743_2026-07-22.md`.

## Weak points to investigate (repo audit)

1. **`assert_ne` gap.** Structural classifier accepts it; Icarus simulation
   emitter does not. W743 uses `assert_eq` on changed elements.
2. **Generator drift.** Copying `gen_W.py` to `gen_W+1.py` and running `sed`
   requires manual `MID_IDX` comment correction and module-name fix (f-string
   `{OUTER}` in header).
3. **Natural modulo wrap already present.** With 19,520 elements, the last raw
   `x` is `2*19519 = 39038`, which exceeds 32768 and wraps to
   `39038 mod 32768 = 6270`. The `make_grid(32768)` call is a period-identity
   regression check, not a first-time wrap test.
4. **No systematic wall-clock limit test.** A stress wave near the 4-MiBit
   boundary remains on the backlog.
5. **Outer stride 305.** First module-scope packed reg strided by 305; fresh
   witness proves end-to-end correctness.
6. **Mid-row index parity.** `MID_IDX = OUTER // 2 = 152` for `OUTER = 305`.
   Mid-row check uses `[152][1][0][0][0][0][0]`, element index `152*64 + 32 =
   9756`. Expected values must use the full row-major formula.
7. **L1 TRACEABILITY gap.** ~38 of the last 200 commits lack `Closes #N`.
8. **L4 TESTABILITY backlog.** 56 `.t27` specs lack tests/invariants/benches.
9. **L7 UNITY drift.** 19 `scripts/*.sh` wrappers on the critical path.
10. **FPGA SSOT contradictions.** `CLAUDE.md` and `cli/dlc10` still cite the old
    board/cable/loader; must align with `fpga/HARDWARE_SSOT.md`.
11. **L6 CEILING gap.** `FORMAT-SPEC-001.json` GF256 `bias_caveat` is still open.

## Decomposed tasks

### W743 closeout

1. [x] **Generator preparation.** Copy `scripts/gen_w742.py` to
      `scripts/gen_w743.py`; set `OUTER = 305` and `MID_IDX = 152`; fix the
      module name header (note the f-string `{OUTER}` in the header line).
2. [x] **Witness generation.** Run `python3 scripts/gen_w743.py` to produce
      `specs/scratch/w743_bench_module_305x2p6_aos_var_call_write.t27`.
3. [x] **Integration test.** Append
      `accepts_w743_bench_module_305x2p6_aos_var_call_write` to
      `bootstrap/tests/icarus_lowerable.rs`.
4. [x] **Release build.** `cargo build --release -p t27c`.
5. [x] **Direct gates.** Run `t27c parse`, `icarus-lowerable`, `icarus-simulate`,
      `icarus-cocotb`, and `seal --save` on the W743 witness.
6. [x] **Cargo conformance.** Run `cargo test -p t27c --bin t27c`,
      `cargo test -p tri`, and `cargo test -p t27c --test icarus_lowerable`.
7. [x] **Baseline.** Create an empty Icarus baseline under
      `.trinity/icarus-baselines/specs/scratch/`.
8. [x] **Closeout and memory.** Write
      `docs/reports/FPGA_LOOP_CLOSEOUT_W743_2026-07-22.md`, update
      `.trinity/experience.md`, and update persistent memory.
9. [x] **Land and next branch.** Commit W743 with `Closes #1714`, record session
      log and commit count, create `wave-loop-744` with W744 cooperation
      variants in `.trinity/current-issue.md`.

### Broader backlog (post-W743)

10. [ ] **L1 cleanup.** Retroactively link or open issues for the wave-loop
       `feat(igla)` / `chore(trinity)` commits so that every merged change has a
       `Closes #N` reference.
11. [ ] **L4 backfill.** Add `test`/`invariant`/`bench` blocks to the 56
       missing-TDD specs, starting with `specs/numeric/gf*.t27` and
       `specs/tri/pipeline/*.t27` / `specs/tri/agent/*.t27`.
12. [ ] **L7 migration.** Retire or re-implement the 19 `scripts/*.sh` critical
       wrappers as `t27c` subcommands (e.g. `t27c verify`, `t27c reseal-apply`).
13. [ ] **FPGA SSOT reconciliation.** Align `CLAUDE.md`, `cli/dlc10`, and stale
       FPGA docs with `fpga/HARDWARE_SSOT.md`; decide whether to update `dlc10`
       or deprecate it.
14. [ ] **L6 reconciliation.** Resolve the `GF256` bias caveat in
       `conformance/FORMAT-SPEC-001.json`.
15. [ ] **Simulator stress wave.** Plan a Wave Loop near the 4-MiBit packed
       vector boundary to measure Icarus/Yosys wall-clock limits.

## Risk register

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Generator copy forgets `MID_IDX` or module-name update | Medium | Low | Verify generated footer assertions and module header before running gates. |
| Icarus simulation path rejects `assert_ne` | High (known) | Low | Use `assert_eq` on changed elements only. |
| Mid-row expected value computed incorrectly | Low | Medium | Reuse corrected W632 formula: `e = r*64 + a5*32 + a4*16 + a3*8 + a2*4 + a1*2 + a0`. |
| Simulator capacity surprise near 0.596 MiBit | Very low | High | Direct `icarus-simulate` gate runs quickly; abort if elapsed exceeds prior waves significantly. |
| Offset-32768 identity check mischaracterised as wrap test | Medium | Low | Document in report that the check verifies period identity, not first-time wrap. |
| L1/L4/L7/L6 gaps block next major release | Medium | High | Prioritize in post-W743 backlog and assign ring stewards. |
| FPGA SSOT contradiction causes wrong physical target | Medium | High | Reconcile docs before any `dlc10` / openFPGALoader flashing. |

## Next Wave Loop 744 cooperation variants

1. **Variant A (recommended): `[307][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - 628,736-bit packed vector, 19,648 elements.
   - Continues the odd outer-dimension ladder and confirms non-p2 stride 307.
   - **Recommended.**

2. **Variant B: `[305][2]^6 Pt` bench-local (function-local) packed array var
   from call with indexed signed writes.**
   - Same width as W743 but tests the mutable `reg` declared inside a bench or
     function rather than at module scope.

3. **Variant C: `[305][2]^6 Pt` module-scope var with `if`-guarded indexed signed
   field writes.**
   - Stays at 0.596 MiBit and tests control-flow-guarded writes on a packed reg.

## Definition of done

- [x] `.claude/plans/wave-loop-743.md` exists and covers the above.
- [x] `scripts/gen_w743.py` created with `OUTER = 305` and `MID_IDX = 152`.
- [x] `specs/scratch/w743_bench_module_305x2p6_aos_var_call_write.t27` generated.
- [x] Integration test `accepts_w743_bench_module_305x2p6_aos_var_call_write` added and passing.
- [x] `cargo build --release -p t27c` passes.
- [x] Direct `t27c` gates pass (parse, lowerable, simulate, cocotb, seal).
- [x] All cargo test suites pass.
- [x] Empty Icarus baseline saved.
- [x] Closeout report, experience, and memory updated.
- [x] Branch `wave-loop-744` created and W743 committed with `Closes #1714`.
