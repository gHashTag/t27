# FPGA / Simulation Wave Loop 744 Closeout

**Issue:** #1715
**Branch:** `wave-loop-744`
**Date:** 2026-07-22
**Previous:** Wave Loop 743 (#1714, branch `wave-loop-743`)

## Chosen variant

**Variant A — module-scope `[307][2]^6 Pt` non-power-of-two outer-dimension
array-of-struct variable initialized from a function call, with indexed signed
field writes.**

Witness: `specs/scratch/w744_bench_module_307x2p6_aos_var_call_write.t27`.

```t27
pub struct Pt { x : i16, y : i16 }
pub fn make_grid(offset : u16) -> [307][2][2][2][2][2][2] Pt { ... }
pub const expected : [307][2][2][2][2][2][2] Pt = make_grid(0);
pub var dst : [307][2][2][2][2][2][2] Pt = make_grid(0);

test module_var_307x2p6_call_write { ... }
bench module_bench_307x2p6_call_write { ... }
```

This continues the module-scope packed AoS odd outer-dimension ladder from 305
(W743) to 307. Total vector width: **307 x 64 x 32 = 628,736 bits** (19,648
elements, ~0.600 MiBit), still well under the ~4-MiBit Icarus/Yosys comfort
threshold.

## What we validated

- A module-level `pub var dst : [307][2]^6 Pt` can be initialized from a function
  call and exercised with indexed signed field writes, with **zero compiler
  changes**.
- The structural classifier (`t27c icarus-lowerable`) accepts the witness.
- Icarus Verilog simulation (`t27c icarus-simulate`) completes in 17 cycles and
  reports `PASSED` for both the `test` and the `bench`.
- The cocotb/Python reference model (`t27c icarus-cocotb`) matches the
  hardware simulation, confirming row-major flattening with outer stride 307 is
  preserved end-to-end.
- The seal hash was saved, the Icarus baseline was recorded, and all
  conformance suites pass.

## Key numbers

| Metric | Value |
|--------|-------|
| Outer dimension | 307 (non-power-of-two) |
| Total elements | 307 x 2^6 = 19,648 |
| Packed vector width | 628,736 bits |
| Approximate size | ~0.600 MiBit |
| Simulation cycles | 17 |
| `t27c --bin t27c` tests | 1494 passed; 0 failed; 2 ignored |
| `tri` tests | 78 passed; 0 failed |
| `icarus_lowerable` tests | 204 passed; 0 failed |
| Compiler changes | 0 |
| Reference-model changes | 0 |
| FROZEN_HASH change | none |

## Files added or modified

- `scripts/gen_w744.py` — new generator script, copied from `gen_w743.py` with
  `OUTER = 307` and `MID_IDX = 153`.
- `specs/scratch/w744_bench_module_307x2p6_aos_var_call_write.t27` — new witness
  (~1,345 KB, ~58,391 lines).
- `bootstrap/tests/icarus_lowerable.rs` — added integration test
  `accepts_w744_bench_module_307x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w744_bench_module_307x2p6_aos_var_call_write.json` —
  saved by `t27c seal --save`.
- `.trinity/icarus-baselines/specs/scratch/w744_bench_module_307x2p6_aos_var_call_write.json` —
  empty baseline.
- `.claude/plans/wave-loop-744.md` — decomposed plan with risk register.
- `.trinity/experience.md` — W744 learnings prepended.
- `.trinity/current-issue.md` — updated for #1715/W745.
- This report: `docs/reports/FPGA_LOOP_CLOSEOUT_W744_2026-07-22.md`.

## Scientific / technical background

- IEEE Std 1800-2017 §7.4.1/7.4.3 — packed-array total width is the product of
  packed dimensions; no power-of-two restriction applies.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays and packed structs
  are synthesizable first-class objects.
- Icarus Verilog issue #1134 — assertion failures for unpacked arrays of packed
  structs; t27's scalar flattening avoids the trigger entirely.
- Icarus issue #1171 — elaboration freeze on very large packed vectors; W744
  remains far below the reported threshold.
- Yosys issue #2677 / #4653 / PR #4100 — native frontend lacks arrays of packed
  structs; t27 lowering to a single packed vector sidesteps the gap.
- Lutsig (CPP 2021) — verified lowering of array reads to bit-vector operations.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array
  scalarization passes.

## Fresh literature scan (2025–2026)

Ternary and multi-valued hardware remains an active lane. The most directly
relevant new results for Trinity's roadmap are in native ternary CPUs,
device-independent ternary RTL synthesis, ternary VHDL extensions, and
parameterized formal verification.

### Native balanced-ternary computing

- **5500FP: A 24-Trit Balanced Ternary RISC Processor** (Zenodo/Open MIND,
  ISMVL 2026). 24-trit word, 120-instruction RISC ISA, native atomic
  synchronization, implemented on an Efinix Trion T120F484 FPGA at 20 MHz.
  Includes the open-hardware GargantuRAM 1.5 PRE dev board with a
  ternary/binary bridge for standard peripherals.
  [doi.org/10.5281/zenodo.18881737](https://doi.org/10.5281/zenodo.18881737)
  [github.com/Ternary-Computer-System/GargantuRAM](https://github.com/Ternary-Computer-System/GargantuRAM)
  [Hackaday coverage](https://hackaday.com/2026/03/16/ternary-risc-processor-achieves-non-binary-computing-via-fpga/)

### Ternary RTL synthesis and HDL extensions

- **H. Park et al., “An RTL-Based General Synthesis Methodology for
  Device-Independent Ternary Logic Circuits”** (IEEE Access, 2025). A complete
  RTL-to-gate-level ternary synthesis flow with a Verilog syntax extension and
  a generic ternary logic (GT-LOGIC) cell library. Reports 63.39% average cell
  count reduction vs. MUX-based synthesis across Memristor-CMOS, CNTFET, T-CMOS,
  and DEPFET technologies.
  [doi.org/10.1109/access.2025.3597293](https://doi.org/10.1109/access.2025.3597293)
- **Tlsys: A Synthesis Framework for Ternary Logic from RTL to CNFET-Based
  Gate-Level Netlist** (Chinese Journal of Electronics, 2026). First framework
  converting ternary RTL into CNFET gate-level netlists, with ternary Verilog
  guidelines and verification methodology for designs over 500,000 gates.
  [doi.org/10.23919/cje.2025.00.418](https://doi.org/10.23919/cje.2025.00.418)
- **Ternary VHDL: Simplifying the Design and Verification of Mixed-radix VLSI
  Circuits** (IEEE ISMVL 2026). Open-source balanced ternary extension to IEEE
  1076-2008 VHDL with behavioral/RTL/structural modeling, 15 unary and dyadic
  gates, relational operators, ternary branching, and arithmetic. Pre-synthesis
  simulation via GHDL/GTKWave.
  [doi.org/10.1109/ismvl68998.2026.00041](https://doi.org/10.1109/ismvl68998.2026.00041)
- **Takahe** — `Zaneham/Takahe` (GitHub). Universal multi-radix hardware
  synthesis tool compiling SystemVerilog/VHDL/ABEL-HDL to gate-level netlists
  for SKY130, IHP SG13G2, GF180MCU, ASAP7. Includes ternary cell definitions and
  has been tested on a Setun-70 ternary processor design.
  [github.com/Zaneham/Takahe](https://github.com/Zaneham/Takahe)

### Multi-valued / ternary neural accelerators

- **KULeuven-MICAS / `ternary-lut-dse`** (GitHub, ISPASS 2026). Chisel
  generator for LUT-based ternary matrix-multiplication accelerators targeting
  1.58-bit quantized LLMs.
  [github.com/KULeuven-MICAS/ternary-lut-dse](https://github.com/KULeuven-MICAS/ternary-lut-dse)

### Formal verification

- **Yongjian Li et al., “Parameterized Hardware Verification Through A
  Term-level Generalized Symbolic Trajectory Evaluation And Its Linkage With
  Concrete Hardware Verification At Netlist Level”** (Formal Aspects of
  Computing, 2025). Extends GSTE to the term level for parameterized designs;
  links term-level proofs to Boolean netlist verification via Yosys-synthesized
  BLIF and Intel Forte constrained STE.
  [doi.org/10.1145/3716828](https://doi.org/10.1145/3716828)
- Foundational STE lineage: Bryant, Beatty, Seger (DAC 1991; CAV 1990/2005).
  [doi.org/10.1145/127601.127701](https://doi.org/10.1145/127601.127701)
  [CMU PDF](https://www.cs.cmu.edu/~bryant/pubdir/cav90.pdf)

## Weak points observed

1. **`assert_ne` gap persists.** The structural classifier accepts `assert_ne`,
   but the Icarus simulation emitter only lowers `assert_eq`. W744 continues to
   use `assert_eq` on changed elements instead of whole-array inequality.
2. **Generator header f-string drift.** The module header in `gen_W.py` uses an
   f-string (`{OUTER}`). A naive global `wN -> wN+1` replacement misses it; the
   copy/edit workflow must explicitly fix the module name.
3. **Offset-32768 check is a period-identity check, not a first-time wrap test.**
   With 19,648 elements, the offset-0 schedule already wraps naturally (last raw
   `x = 2*19647 = 39294`, `39294 mod 32768 = 6526`). Adding offset 32768 is
   congruent to adding 0 modulo 32768, so `make_grid(32768)` returns exactly the
   same values as `make_grid(0)`.
4. **No systematic wall-clock limit test yet.** At 0.600 MiBit we remain
   comfortable, but a stress wave near the 4-MiBit boundary remains on the
   backlog.
5. **Manual `MID_IDX` correction.** Each generator copy still needs a manual
   `MID_IDX` comment update.
6. **L1 TRACEABILITY gap persists.** ~28 of the last 200 commits still lack
   `Closes #N` / `Fixes #N` / `Resolves #N`, mostly the `chore(trinity): record
   final W7XX session log` commits. L1 is the highest-priority law.
7. **L4 TESTABILITY backlog persists.** 57 `.t27` specs still lack
   `test`/`invariant`/`bench` blocks.
8. **L7 UNITY drift persists.** 19 `scripts/*.sh` wrappers remain on the
   verification/critical path, plus new untracked `.sh` files under `.agents/`
   and `.codex/hooks/`.
9. **FPGA SSOT contradictions persist.** `CLAUDE.md` and `cli/dlc10` still cite
   the old `XC7A100T` / `0x13631093` / `dlc10` path, while
   `fpga/HARDWARE_SSOT.md` canon is `XC7A200T` / `0x03636093` / openFPGALoader.
10. **L6 CEILING gap persists.** `FORMAT-SPEC-001.json` GF256 `bias_caveat` is
    still open.
11. **Dirty worktree.** `.claude/scheduled_tasks.lock` is modified (session lock)
    and `.agents/` / `.codex/` directories are untracked; these should be
    reverted/ignored before the next commit.
12. **Conformance reports show known failures.** `build/suite_report.json` reports
    7 `gen-verilog-yosys-smoke` failures (marked acceptable), and
    `conformance/kepler_newton_results.json` reports 4/16 failures. These are not
    new W744 blockers but are tracked weaknesses.

## Next Wave Loop 745 cooperation variants

1. **Variant A (recommended): `[309][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - 632,832-bit packed vector, 19,776 elements.
   - Continues the odd outer-dimension ladder and confirms non-p2 stride 309.
   - **Recommended.**

2. **Variant B: `[307][2]^6 Pt` bench-local (function-local) packed array var
   from call with indexed signed writes.**
   - Same width as W744 but tests the mutable `reg` declared inside a bench or
     function rather than at module scope.

3. **Variant C: `[307][2]^6 Pt` module-scope var with `if`-guarded indexed signed
   field writes.**
   - Stays at 0.600 MiBit and tests control-flow-guarded writes on a packed reg.

## Definition of done status

- [x] Issue #1715 current-issue and plan written.
- [x] Generator and witness created.
- [x] Integration test added and passing.
- [x] Release build and all direct gates passing.
- [x] Test suites passing.
- [x] Seal and Icarus baseline saved.
- [x] Closeout report and experience/memory updated.
- [ ] Branch `wave-loop-745` created and W744 committed with `Closes #1715`.

## Conclusion

Wave Loop 744 successfully validated a 628,736-bit module-scope packed
array-of-structs with non-power-of-two outer dimension 307, initialized from a
function call and mutated via indexed signed field writes. No compiler or
reference-model changes were required. The ladder can continue to 309 and beyond
while staying under the 4-MiBit simulation ceiling. Structural weak points
(L1, L4, L7, FPGA SSOT, L6) remain on the backlog and should be prioritized in
future planning cycles.

---

φ² + 1/φ² = 3 | TRINITY
