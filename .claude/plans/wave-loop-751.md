# Wave Loop 751 — Decomposed Plan

**Issue:** #1722 (expected)
**Branch:** `wave-loop-751`
**Date:** 2026-07-23
**Previous:** Wave Loop 750 (#1721, branch `wave-loop-750`)

## Goal

Validate a module-scope `[321][2]^6 Pt` non-power-of-two outer-dimension
array-of-struct variable initialized from a function call, with indexed signed
field writes. The witness is a 657,408-bit packed vector (20,544 elements,
~0.627 MiBit), continuing the odd outer-dimension ladder while staying far below
the ~4-MiBit simulation ceiling.

At the same time, use the closeout as a checkpoint to audit repository weak
points, scan fresh scientific literature, and prepare a decomposed backlog for
Wave Loop 752 and the next ring.

## Weak-point audit (current snapshot)

| Law / area | Finding | Severity |
|---|---|---|
| L1 TRACEABILITY | 518 commits in the last 30 days; 407 carry `Closes #N`, leaving 111 without issue linkage. Wave-loop closeouts themselves now carry it, but historical backlog is large. | Medium |
| L4 TESTABILITY | 852 `.t27` specs exist; 445 still lack `test`/`invariant`/`bench` blocks. | High |
| L7 UNITY | 19 `scripts/*.sh` wrappers remain on the critical path beyond the two permitted exceptions; plus 9 untracked `.sh` hooks under `.agents/` and `.codex/hooks/`. | Medium |
| L6 CEILING | `conformance/FORMAT-SPEC-001.json` `GF256.bias_caveat` still says bias is `UNRECONCILED` / `OPEN`. | Medium |
| FPGA SSOT | `fpga/HARDWARE_SSOT.md` is canonical (QMTech Wukong V1, IDCODE `0x13631093`, `cli/dlc10` loader), but `CLAUDE.md` and some stale docs still need alignment. | Medium |
| Dirty worktree | `.agents/` and `.codex/` are tracked now as agent-skill artifacts; they must remain intentional and not bloat the repo. | Low |

These are **not** W751 blockers; they are documented for the next ring stewards.

## Scientific / technical background

- IEEE Std 1800-2017 §7.4.1/7.4.3 — packed-array total width is the product of
  packed dimensions; no power-of-two restriction applies.
- Accellera `vlog-pp` discussion (Graham 2002) — packed arrays are contiguous
  bit vectors.
- Sutherland, "Synthesizable SystemVerilog" — packed arrays and packed structs
  are synthesizable first-class objects.
- Icarus Verilog issue #1134 — assertion failures for unpacked arrays of packed
  structs; t27's scalar flattening avoids the trigger entirely.
- Icarus issue #1171 — elaboration freeze on very large packed vectors; W751
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
- **Takahe** (2026, GitHub `Zaneham/Takahe`) — multi-radix open-source hardware
  synthesis tool supporting SystemVerilog/VHDL/ABEL-HDL; binary, ternary
  (`--radix 3` balanced ternary), duodecimal, and other radices; targets SKY130,
  IHP, GF180, ASAP7, nextpnr JSON/iCE40. Includes `Setun-70` ternary processor
  design (~153 cells). Repo: <https://github.com/Zaneham/Takahe>.
- **5500FP** (2026, Zenodo) — 24-trit balanced ternary RISC processor
  implemented on FPGA (Efinix Trion T20F256), 120-instruction ISA, native atomic
  primitives, real ±3.3V ternary I/O, open hardware development board
  GargantuRAM. DOI: <https://doi.org/10.5281/zenodo.18881738>; Hackaday
  coverage: <https://hackaday.com/2026/03/16/ternary-risc-processor-achieves-non-binary-computing-via-fpga/>.
- **In-memory balanced ternary logic with tri-valued memristors** (2026,
  *European Physical Journal Plus*) — balanced ternary gates/decoders using
  memristor resistance states for in-memory MVL computing.
  DOI: <https://doi.org/10.1140/epjp/s13360-026-07895-z>.
- **Energy-optimized GNRFET+RRAM ternary logic** (2026, ICOECIT) — hybrid
  graphene-nanoribbon FET and resistive RAM architecture for energy-efficient
  ternary STI/THA, −36% delay / −50% power / −68% PDP vs. CNTFET-RRAM.
  DOI: <https://doi.org/10.1109/icoecit68303.2026.11497012>.
- **Trinity v2.0.x / B002** (2025/2026, Zenodo) — zero-DSP ternary-weight
  autoregressive LLM inference on QMTech XC7A100T using OpenXC7
  (Yosys/nextpnr-xilinx/Project X-Ray), ~63 tok/s @ 92 MHz, ~1 W.
  DOIs: <https://doi.org/10.5281/zenodo.18939352>,
  <https://doi.org/10.5281/zenodo.19224235>.
- **OpenXC7 / nextpnr-xilinx / Project X-Ray** — fully open-source Xilinx
  7-series toolchain (Yosys + nextpnr-xilinx + prjxray + fasm2bit), used for
  QMTech XC7A100T ternary/φ-numeric projects without Vivado.
  Repos: <https://github.com/openXC7/nextpnr-xilinx>,
  <https://github.com/openXC7/toolchain-installer>.

Takeaway: ternary/MVL tooling is converging on HDL-compatible frontends
(ternary Verilog/VHDL), binary-coded-ternary/BCT on standard FPGAs, and
open-source flows. This is consistent with t27's scalarized packed-vector
strategy and its FPGA SSOT (QMTech Wukong V1 / XC7A100T) target.

## Tasks

1. [ ] **Select variant.** Variant A: module-scope `[321][2]^6 Pt` variable
   initialized from a call, with indexed signed field writes and read-back.
2. [ ] **Create generator.** Copy `scripts/gen_w750.py` to `scripts/gen_w751.py`,
   set `OUTER = 321` and `MID_IDX = 160`, verify module name uses `w751`.
3. [ ] **Generate witness.** Run `python3 scripts/gen_w751.py` to produce
   `specs/scratch/w751_bench_module_321x2p6_aos_var_call_write.t27`.
4. [ ] **Add integration test.** Append
   `accepts_w751_bench_module_321x2p6_aos_var_call_write` to
   `bootstrap/tests/icarus_lowerable.rs`.
5. [ ] **Build and gate.** Run `cargo build --release -p t27c`, then direct
   `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save`.
6. [ ] **Run cargo suites.** `cargo test -p t27c --bin t27c`, `cargo test -p tri`,
   `cargo test -p t27c --test icarus_lowerable`.
7. [ ] **Save artifacts.** Empty Icarus baseline in
   `.trinity/icarus-baselines/specs/scratch/`, seal in `.trinity/seals/`.
8. [ ] **Closeout report.** Write `docs/reports/FPGA_LOOP_CLOSEOUT_W751_2026-07-23.md`.
9. [ ] **Experience update.** Prepend W751 learnings to `.trinity/experience.md`.
10. [ ] **Current issue handoff.** Update `.trinity/current-issue.md` to W752.
11. [ ] **Memory update.** Write
    `~/.claude/projects/-Users-playra-t27/memory/wave-loop-751.md` and append
    one-line pointer to `MEMORY.md`.
12. [ ] **Branch / merge.** Commit W751 with `Closes #1722`; create
    `wave-loop-752`.

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
