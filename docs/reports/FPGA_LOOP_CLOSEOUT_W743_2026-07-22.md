# FPGA / Simulation Wave Loop 743 Closeout

**Issue:** #1714
**Branch:** `wave-loop-743`
**Date:** 2026-07-22
**Previous:** Wave Loop 742 (#1713, branch `wave-loop-742`)

## Chosen variant

**Variant A — module-scope `[305][2]^6 Pt` non-power-of-two outer-dimension
array-of-struct variable initialized from a function call, with indexed signed
field writes.**

Witness: `specs/scratch/w743_bench_module_305x2p6_aos_var_call_write.t27`.

```t27
pub struct Pt { x : i16, y : i16 }
pub fn make_grid(offset : u16) -> [305][2][2][2][2][2][2] Pt { ... }
pub const expected : [305][2][2][2][2][2][2] Pt = make_grid(0);
pub var dst : [305][2][2][2][2][2][2] Pt = make_grid(0);

test module_var_305x2p6_call_write { ... }
bench module_bench_305x2p6_call_write { ... }
```

This continues the module-scope packed AoS odd outer-dimension ladder from 303
(W742) to 305. Total vector width: **305 x 64 x 32 = 624,640 bits** (19,520
elements, ~0.596 MiBit), still well under the ~4-MiBit Icarus/Yosys comfort
threshold.

## What we validated

- A module-level `pub var dst : [305][2]^6 Pt` can be initialized from a function
  call and exercised with indexed signed field writes, with **zero compiler
  changes**.
- The structural classifier (`t27c icarus-lowerable`) accepts the witness.
- Icarus Verilog simulation (`t27c icarus-simulate`) completes in 17 cycles and
  reports `PASSED` for both the `test` and the `bench`.
- The cocotb/Python reference model (`t27c icarus-cocotb`) matches the
  hardware simulation, confirming row-major flattening with outer stride 305 is
  preserved end-to-end.
- The seal hash was saved, the Icarus baseline was recorded, and all
  conformance suites pass.

## Key numbers

| Metric | Value |
|--------|-------|
| Outer dimension | 305 (non-power-of-two) |
| Total elements | 305 x 2^6 = 19,520 |
| Packed vector width | 624,640 bits |
| Approximate size | ~0.596 MiBit |
| Simulation cycles | 17 |
| `t27c --bin t27c` tests | 1494 passed; 0 failed; 2 ignored |
| `tri` tests | 78 passed; 0 failed |
| `icarus_lowerable` tests | 203 passed; 0 failed |
| Compiler changes | 0 |
| Reference-model changes | 0 |
| FROZEN_HASH change | none |

## Files added or modified

- `scripts/gen_w743.py` — new generator script, copied from `gen_w742.py` with
  `OUTER = 305` and `MID_IDX = 152`.
- `specs/scratch/w743_bench_module_305x2p6_aos_var_call_write.t27` — new witness
  (~1,336 KB, ~58,011 lines).
- `bootstrap/tests/icarus_lowerable.rs` — added integration test
  `accepts_w743_bench_module_305x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w743_bench_module_305x2p6_aos_var_call_write.json` —
  saved by `t27c seal --save`.
- `.trinity/icarus-baselines/specs/scratch/w743_bench_module_305x2p6_aos_var_call_write.json` —
  empty baseline.
- `.claude/plans/wave-loop-743.md` — decomposed plan with risk register.
- `.trinity/experience.md` — W743 learnings prepended.
- `.trinity/current-issue.md` — updated for #1714/W744.
- This report: `docs/reports/FPGA_LOOP_CLOSEOUT_W743_2026-07-22.md`.

## Scientific / technical background

- IEEE Std 1800-2017 §7.4.1/7.4.3 — packed-array total width is the product of
  packed dimensions; no power-of-two restriction applies.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays and packed structs
  are synthesizable first-class objects.
- Icarus Verilog issue #1134 — assertion failures for unpacked arrays of packed
  structs; t27's scalar flattening avoids the trigger entirely.
- Icarus issue #1171 — elaboration freeze on very large packed vectors; W743
  remains far below the reported threshold.
- Yosys issue #2677 / #4653 / PR #4100 — native frontend lacks arrays of packed
  structs; t27 lowering to a single packed vector sidesteps the gap.
- Lutsig (CPP 2021) — verified lowering of array reads to bit-vector operations.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array
  scalarization passes.

## Recent literature scan (2024–2025)

A quick sweep of recent work shows ternary and multi-valued hardware is active
in two lanes: **native balanced-ternary processors/SoCs** and **ternary-weight
neural-network accelerators**. Both are relevant to Trinity's long-term roadmap,
even though the current wave-loop ladder only exercises scalar-struct packed
arrays.

### Native / classical balanced-ternary computing

- **VTX1** — `itworks99/vtx1` (GitHub, 2025). A full balanced-ternary SoC
  (CPU, memory controllers, UART/SPI/I2C/GPIO/DMA) using trits {-1,0,+1},
  Verilog source, Icarus/Yosys flow, planned SkyWater 130nm OpenLane tape-out.
  [github.com/itworks99/vtx1](https://github.com/itworks99/vtx1)
- **5500FP: A 24-Trit Balanced Ternary RISC Processor** (Zenodo/Open MIND,
  2026). 24-trit balanced ternary RISC CPU on FPGA with 120-instruction ISA and
  native atomic synchronization primitives.
  [doi.org/10.5281/zenodo.18881738](https://doi.org/10.5281/zenodo.18881738)

### Ternary-weight LLM accelerators on FPGA

These use the same {-1,0,+1} alphabet as ternary computing but for 1.58-bit
quantized neural inference.

- **TerEffic: Highly Efficient Ternary LLM Inference on FPGA** (arXiv 2025).
  AMD Alveo U280, custom TMat core using LUTs rather than DSPs.
  [arXiv:2502.16473](https://arxiv.org/html/2502.16473v2)
- **TeLLMe: An Energy-Efficient Ternary LLM Accelerator for Prefilling and
  Decoding on Edge FPGAs** (arXiv 2025). AMD Kria KV260, table-lookup ternary
  matmul, ~9.5 tok/s decode under 7 W.
  [arXiv:2504.16266](https://arxiv.org/pdf/2504.16266)
- **Trinity B002: Zero-DSP FPGA Architecture for Ternary Inference** (Zenodo,
  2026). Xilinx 7-series, pure-LUT ternary MAC, 70% DSP reduction vs. binary,
  OpenXC7/Yosys flow.
  [doi.org/10.5281/zenodo.19224235](https://doi.org/10.5281/zenodo.19224235)

### Multi-valued logic synthesis frameworks

- **Tlsys: A Synthesis Framework for Ternary Logic from RTL to CNFET-Based
  Gate-Level Netlist** (Chinese Journal of Electronics, 2026). The first
  framework to synthesize ternary RTL into gate-level netlists, with a
  verification methodology and designs over 500,000 gates.
  [doi.org/10.23919/cje.2025.00.418](https://doi.org/10.23919/cje.2025.00.418)
- **Steven Bos, “Beyond 0 and 1: A mixed radix design and verification workflow
  for modern ternary computers”** (PhD thesis, 2024). Browser-based open-source
  EDA tool **MRCS** for binary/ternary/mixed-radix circuits, outputting HSPICE
  and Verilog; designed **REBEL-2**, a RISC-V-like balanced ternary CPU.
  [academia.edu paper](https://www.academia.edu/130282600/Beyond_0_and_1_A_mixed_radix_design_and_verification_workflow_for_modern_ternary_computers)
- **KULeuven-MICAS / `ternary-lut-dse`** (GitHub, ISPASS 2026). Chisel
  generator for LUT-based ternary matrix-multiplication accelerators targeting
  1.58-bit quantized LLMs.
  [github.com/KULeuven-MICAS/ternary-lut-dse](https://github.com/KULeuven-MICAS/ternary-lut-dse)

### Formal-methods lineage for ternary hardware

- **R. E. Bryant, D. L. Beatty, C.-J. H. Seger, “Formal hardware verification by
  symbolic ternary trajectory evaluation”** (ACM, 1991). Foundational STE work
  using a ternary model (0, 1, X unknown) for symbolic switch-level simulation.
  [doi.org/10.1145/127601.127701](https://doi.org/10.1145/127601.127701)
- **Bryant & Seger, “Formal Verification of Digital Circuits Using Symbolic
  Ternary System Models”** (1990/2005). Canonical STE: ternary state model,
  monotone excitation functions, trajectory formulas, BDD-based symbolic
  simulation.
  [cmu.edu PDF](https://www.cs.cmu.edu/~bryant/pubdir/cav90.pdf)
- **A. Rosenmann, “A Multiple-Valued Logic Approach to the Design and
  Verification of Hardware Circuits”** (arXiv:1502.05748). Extends ternary
  simulation to general MVL over Kleene/fuzzy-style algebra for equivalence
  checking and assertion generation.
  [ar5iv:1502.05748](https://ar5iv.labs.arxiv.org/html/1502.05748)

## Weak points observed

1. **`assert_ne` gap persists.** The structural classifier accepts `assert_ne`,
   but the Icarus simulation emitter only lowers `assert_eq`. W743 continues to
   use `assert_eq` on changed elements instead of whole-array inequality.
2. **Generator header f-string drift.** The module header in `gen_W.py` uses an
   f-string (`{OUTER}`). A naive global `wN -> wN+1` replacement misses it; the
   copy/edit workflow must explicitly fix the module name.
3. **Offset-32768 check is a period-identity check, not a first-time wrap test.**
   With 19,520 elements, the offset-0 schedule already wraps naturally (last raw
   `x = 2*19519 = 39038`, `39038 mod 32768 = 6270`). Adding offset 32768 is
   congruent to adding 0 modulo 32768, so `make_grid(32768)` returns exactly the
   same values as `make_grid(0)`.
4. **No systematic wall-clock limit test yet.** At 0.596 MiBit we remain
   comfortable, but a stress wave near the 4-MiBit boundary remains on the
   backlog.
5. **Manual `MID_IDX` correction.** Each generator copy still needs a manual
   `MID_IDX` comment update.
6. **L1 TRACEABILITY gap in wave-loop commit history.** Roughly 38 of the last
   200 commits lack `Closes #N` / `Fixes #N` / `Resolves #N`, including the
   `feat(igla): Wave Loop 7XX` and `chore(trinity): record final W7XX session
   log` series. L1 is the highest-priority law; this should be cleaned up before
   the next major ring.
7. **L4 TESTABILITY backlog.** 56 `.t27` specs under `specs/` still lack
   `test`/`invariant`/`bench` blocks, including core pipeline/agent specs and
   numeric family members (`gf128.t27`, `gf256.t27`, `gf48.t27`, etc.).
8. **L7 UNITY / NO-NEW-SHELL drift.** Nineteen `scripts/*.sh` wrappers remain on
   the verification/critical path (e.g. `verify.sh`, `phi-loop-stack.sh`,
   `reseal-apply.sh`), well beyond the two permitted exceptions in SOUL.md.
9. **FPGA SSOT contradictions.** `CLAUDE.md` still cites the old
   `XC7A100T-FGG676` / IDCODE `0x13631093` / `dlc10` path, while
   `fpga/HARDWARE_SSOT.md` canon is `XC7A200T-FGG676` / IDCODE `0x03636093` /
   `openFPGALoader`. The in-tree `cli/dlc10` driver also hardcodes the old
   board/cable. This must be reconciled before any physical FPGA flashing.
10. **`FORMAT-SPEC-001.json` GF256 bias caveat unresolved.** L6 CEILING
    identifies the format registry as the numeric SSOT; the `GF256` entry still
    carries an open `bias_caveat`.

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

## Definition of done status

- [x] Issue #1714 current-issue and plan written.
- [x] Generator and witness created.
- [x] Integration test added and passing.
- [x] Release build and all direct gates passing.
- [x] Test suites passing.
- [x] Seal and Icarus baseline saved.
- [x] Closeout report and experience/memory updated.
- [x] Branch `wave-loop-744` created and W743 committed with `Closes #1714`.

## Conclusion

Wave Loop 743 successfully validated a 624,640-bit module-scope packed
array-of-structs with non-power-of-two outer dimension 305, initialized from a
function call and mutated via indexed signed field writes. No compiler or
reference-model changes were required. The ladder can continue to 307 and beyond
while staying under the 4-MiBit simulation ceiling. The closeout also surfaced
structural weak points (L1, L4, L7, FPGA SSOT, and numeric SSOT gaps) that
should be prioritized in the next planning cycle.

---

φ² + 1/φ² = 3 | TRINITY
