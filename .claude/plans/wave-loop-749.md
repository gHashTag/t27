# Wave Loop 749 — Decomposed Plan

**Issue:** #1720 (expected)
**Branch:** `wave-loop-749`
**Date:** 2026-07-23
**Previous:** Wave Loop 748 (#1719, branch `wave-loop-748`)

## Goal

Validate a module-scope `[317][2]^6 Pt` non-power-of-two outer-dimension
array-of-struct variable initialized from a function call, with indexed signed
field writes. The witness is a 649,216-bit packed vector (20,288 elements,
~0.620 MiBit), continuing the odd outer-dimension ladder while staying far below
the ~4-MiBit simulation ceiling.

At the same time, use the closeout as a checkpoint to audit repository weak
points, scan fresh scientific literature, and prepare a decomposed backlog for
Wave Loop 750 and the next ring.

## Weak-point audit (current snapshot)

| Law / area | Finding | Severity |
|---|---|---|
| L1 TRACEABILITY | 514 commits in the last 30 days; 405 carry `Closes #N`, leaving 109 without issue linkage. Wave-loop closeouts themselves now carry it, but historical backlog is large. | Medium |
| L4 TESTABILITY | 852 `.t27` specs exist; 445 still lack `test`/`invariant`/`bench` blocks. | High |
| L7 UNITY | 19 `scripts/*.sh` wrappers remain on the critical path beyond the two permitted exceptions; plus 9 untracked `.sh` hooks under `.agents/` and `.codex/hooks/`. | Medium |
| L6 CEILING | `conformance/FORMAT-SPEC-001.json` `GF256.bias_caveat` still says bias is `UNRECONCILED` / `OPEN`. | Medium |
| FPGA SSOT | `fpga/HARDWARE_SSOT.md` is canonical (QMTech Wukong V1, IDCODE `0x13631093`, `cli/dlc10` loader), but `CLAUDE.md` and some stale docs still need alignment. | Medium |
| Dirty worktree | `.agents/` and `.codex/` are tracked now as agent-skill artifacts; they must remain intentional and not bloat the repo. | Low |

These are **not** W749 blockers; they are documented for the next ring stewards.

## Scientific / technical background

- IEEE Std 1800-2017 §7.4.1/7.4.3 — packed-array total width is the product of
  packed dimensions; no power-of-two restriction applies.
- Accellera `vlog-pp` discussion (Graham 2002) — packed arrays are contiguous
  bit vectors.
- Sutherland, "Synthesizable SystemVerilog" — packed arrays and packed structs
  are synthesizable first-class objects.
- Icarus Verilog issue #1134 — assertion failures for unpacked arrays of packed
  structs; t27's scalar flattening avoids the trigger entirely.
- Icarus issue #1171 — elaboration freeze on very large packed vectors; W749
  remains far below the reported threshold.
- Yosys issue #2677 / #4653 / PR #4100 — native frontend lacks arrays of packed
  structs; t27 lowering to a single packed vector sidesteps the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals;
  flat `LogicArray` for multidimensional packed arrays in the reference model.
- Lutsig (CPP 2021) — verified lowering of array reads to bit-vector operations.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array
  scalarization passes.

## Fresh literature scan (2025–2026)

- **Tlsys** (2026, *Chinese Journal of Electronics*) — first framework to
  synthesize ternary RTL designs into CNFET-based gate-level netlists, with
  ternary Verilog input guidelines and designs over 500,000 gates.
  DOI: <https://doi.org/10.23919/cje.2025.00.418>.
- **Ternary VHDL (TVHDL)** (ISMVL 2026) — balanced-ternary extension to IEEE
  1076-2008 VHDL; open-source library with 15 unary/dyadic gates, relational
  operators, shifts, and arithmetic; GHDL/GTKwave simulation.
  DOI: <https://doi.org/10.1109/ismvl68998.2026.00041>.
- **SONIC / SimulationEngine** (2025/2026, USN Ternary Research Group) — C#
  EDA toolchain for ternary/mixed-radix VLSI circuits, successor to MRCS,
  event-driven gate-level simulator with delta cycles, REBEL-2 balanced ternary
  CPU, Verilog export and Basys3 emitter. Accepted at ISMVL 2026.
  DOI: <https://doi.org/10.1109/ismvl68998.2026.00042>; repo:
  <https://github.com/sonbit/SimulationEngine>.
- **REBEL-6** (ISMVL 2025) — 32-trit balanced ternary ISA with an RV32I-to-REBEL
  (R2R) compiler pipeline for C, successor/comparison to REBEL-2.
  DOI: <https://doi.org/10.1109/ismvl64713.2025.00028>.
- **Trinity v2.0.x / B002** (2025/2026, Zenodo) — zero-DSP ternary-weight
  autoregressive LLM inference on QMTech XC7A100T using OpenXC7
  (Yosys/nextpnr-xilinx/Project X-Ray), ~63 tok/s @ 92 MHz, ~1 W.
  DOIs: <https://doi.org/10.5281/zenodo.18939352>,
  <https://doi.org/10.5281/zenodo.19224235>.
- **RTL-Based General Synthesis Methodology for Ternary Logic** (2025, IEEE Access)
  — generic ternary RTL-to-gate-level synthesis, GT-LOGIC library/mapping,
  63.39% average cell-count reduction vs. MUX-based synthesis, demonstrated on
  memristor-CMOS, CNTFET, T-CMOS, and DEPFET devices.
  URL: <https://sah.borca.ai/papers/281362292>.

Takeaway: ternary/MVL tooling is converging on HDL-compatible frontends
(ternary Verilog/VHDL), binary-coded-ternary/BCT on standard FPGAs, and
open-source flows. This is consistent with t27's scalarized packed-vector
strategy and its FPGA SSOT (QMTech Wukong V1 / XC7A100T) target.

## Tasks

1. [ ] **Select variant.** Variant A: module-scope `[317][2]^6 Pt` variable
   initialized from a call, with indexed signed field writes and read-back.
2. [ ] **Create generator.** Copy `scripts/gen_w748.py` to `scripts/gen_w749.py`,
   set `OUTER = 317` and `MID_IDX = 158`, verify module name uses `w749`.
3. [ ] **Generate witness.** Run `python3 scripts/gen_w749.py` to produce
   `specs/scratch/w749_bench_module_317x2p6_aos_var_call_write.t27`.
4. [ ] **Add integration test.** Append
   `accepts_w749_bench_module_317x2p6_aos_var_call_write` to
   `bootstrap/tests/icarus_lowerable.rs`.
5. [ ] **Build and gate.** Run `cargo build --release -p t27c`, then direct
   `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save`.
6. [ ] **Run cargo suites.** `cargo test -p t27c --bin t27c`, `cargo test -p tri`,
   `cargo test -p t27c --test icarus_lowerable`.
7. [ ] **Save artifacts.** Empty Icarus baseline in
   `.trinity/icarus-baselines/specs/scratch/`, seal in `.trinity/seals/`.
8. [ ] **Closeout report.** Write `docs/reports/FPGA_LOOP_CLOSEOUT_W749_2026-07-23.md`.
9. [ ] **Experience update.** Prepend W749 learnings to `.trinity/experience.md`.
10. [ ] **Current issue handoff.** Update `.trinity/current-issue.md` to W750.
11. [ ] **Memory update.** Write
    `~/.claude/projects/-Users-playra-t27/memory/wave-loop-749.md` and append
    one-line pointer to `MEMORY.md`.
12. [ ] **Branch / merge.** Commit W749 with `Closes #1720`; create
    `wave-loop-750`.

## Decomposed backlog for next ring

1. [ ] **L1 cleanup.** Retroactively link or open issues for historical
       commits lacking `Closes #N`; enforce going forward.
2. [ ] **L4 backfill.** Add `test`/`invariant`/`bench` blocks to the 445
       missing-TDD specs, starting with `specs/fpga/*.t27` and
       `specs/numeric/gf*.t27`.
3. [ ] **L7 migration.** Retire or re-implement the 19 `scripts/*.sh` critical
       wrappers as `t27c` subcommands (e.g. `t27c verify`, `t27c reseal-apply`).
4. [ ] **FPGA SSOT reconciliation.** Align `CLAUDE.md`, `cli/dlc10`, and stale
       FPGA docs with `fpga/HARDWARE_SSOT.md`.
5. [ ] **L6 CEILING.** Resolve or document `FORMAT-SPEC-001.json`
       `GF256.bias_caveat`.
6. [ ] **Simulator stress wave.** Plan a Wave Loop near the ~4-MiBit
       packed-vector boundary to measure Icarus/Yosys wall-clock limits.
