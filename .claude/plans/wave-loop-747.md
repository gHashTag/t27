# Wave Loop 747 — Decomposed Plan

**Issue:** #1718 (expected)
**Branch:** `wave-loop-747`
**Date:** 2026-07-22
**Previous:** Wave Loop 746 (#1717, branch `wave-loop-746`)

## Goal

Validate a module-scope `[313][2]^6 Pt` non-power-of-two outer-dimension
array-of-struct variable initialized from a function call, with indexed signed
field writes. The witness is a 641,024-bit packed vector (20,032 elements,
~0.612 MiBit), continuing the odd outer-dimension ladder while staying far below
the ~4-MiBit simulation ceiling.

At the same time, use the closeout as a checkpoint to audit repository weak
points, scan fresh scientific literature, and prepare a decomposed backlog for
Wave Loop 748 and the next ring.

## Weak-point audit (current snapshot)

| Law / area | Finding | Severity |
|---|---|---|
| L1 TRACEABILITY | 424 commits in the last 30 days lack `Closes #N`; wave-loop closeouts themselves now carry it, but historical backlog is large. | Medium |
| L4 TESTABILITY | 57 `.t27` specs still lack `test`/`invariant`/`bench` blocks. | Medium |
| L7 UNITY | 19 `scripts/*.sh` wrappers remain on the critical path beyond the two permitted exceptions; plus 9 untracked `.sh` hooks under `.agents/` and `.codex/hooks/`. | Medium |
| L6 CEILING | `conformance/FORMAT-SPEC-001.json` `GF256.bias_caveat` still says bias is `UNRECONCILED` / `OPEN`. | Medium |
| FPGA SSOT | `fpga/HARDWARE_SSOT.md` is canonical (QMTech Wukong V1, IDCODE `0x13631093`, `cli/dlc10` loader), but `CLAUDE.md` and some stale docs still need alignment. | Medium |
| Dirty worktree | `.agents/` and `.codex/` are untracked; they must not be committed accidentally. | Low |

These are **not** W747 blockers; they are documented for the next ring stewards.

## Scientific / technical background

- IEEE Std 1800-2017 §7.4.1/7.4.3 — packed-array total width is the product of
  packed dimensions; no power-of-two restriction applies.
- Accellera `vlog-pp` discussion (Graham 2002) — packed arrays are contiguous
  bit vectors.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays and packed structs
  are synthesizable first-class objects.
- Icarus Verilog issue #1134 — assertion failures for unpacked arrays of packed
  structs; t27's scalar flattening avoids the trigger entirely.
- Icarus issue #1171 — elaboration freeze on very large packed vectors; W747
  remains far below the reported threshold.
- Yosys issue #2677 / #4653 / PR #4100 — native frontend lacks arrays of packed
  structs; t27 lowering to a single packed vector sidesteps the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals;
  flat `LogicArray` for multidimensional packed arrays in the reference model.
- Lutsig (CPP 2021) — verified lowering of array reads to bit-vector operations.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array
  scalarization passes.

## Fresh literature scan (2025–2026)

- **Takahe** (2026, github.com/Zaneham/Takahe) — open-source universal synthesis
  supporting binary, ternary / balanced ternary (`--radix 3`), duodecimal, and
  other radices. SystemVerilog/VHDL/ABEL-HDL input, nextpnr iCE40 FPGA export,
  BLIF, Yosys JSON, mapped Verilog. Includes `Setun-70` ternary processor test.
- **Tlsys** (2025/2026, Chinese Journal of Electronics) — first framework to
  synthesize ternary RTL designs into CNFET-based gate-level netlists, with
  ternary Verilog input guidelines and designs over 500,000 gates.
- **SONIC / SimulationEngine** (2025/2026, github.com/sonbit/SimulationEngine) —
  C# ternary EDA toolchain with event-driven simulator, REBEL-2 balanced ternary
  CPU, Verilog export for Xilinx Basys3 FPGA; accepted at ISMVL 2026.
- **Trinity B002** (2025/2026, Zenodo) — zero-DSP FPGA architecture for ternary
  inference on Xilinx 7-series, balanced ternary `{-1, 0, +1}`, full open-source
  flow with Yosys, NextPNR-Xilinx, OpenXC7, Docker synthesis; supports QMTech
  boards. Relevant to t27's long-term FPGA inference target.

Takeaway: ternary/MVL tooling is converging on HDL-compatible frontends and
packed-aggregate flattening, consistent with t27's scalarized packed-vector
strategy. The QMTech-board support in Trinity B002 aligns with t27's own FPGA
SSOT target.

## Tasks

1. [x] **Select variant.** Variant A: module-scope `[313][2]^6 Pt` variable
   initialized from a call, with indexed signed field writes and read-back.
2. [x] **Create generator.** Copy `scripts/gen_w746.py` to `scripts/gen_w747.py`,
   set `OUTER = 313` and `MID_IDX = 156`, verify module name uses `w747`.
3. [x] **Generate witness.** Run `python3 scripts/gen_w747.py` to produce
   `specs/scratch/w747_bench_module_313x2p6_aos_var_call_write.t27`.
4. [x] **Add integration test.** Append
   `accepts_w747_bench_module_313x2p6_aos_var_call_write` to
   `bootstrap/tests/icarus_lowerable.rs`.
5. [x] **Build and gate.** Run `cargo build --release -p t27c`, then direct
   `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save`.
6. [x] **Run cargo suites.** `cargo test -p t27c --bin t27c`, `cargo test -p tri`,
   `cargo test -p t27c --test icarus_lowerable`.
7. [x] **Save artifacts.** Empty Icarus baseline in
   `.trinity/icarus-baselines/specs/scratch/`, seal in `.trinity/seals/`.
8. [x] **Closeout report.** Write `docs/reports/FPGA_LOOP_CLOSEOUT_W747_2026-07-22.md`.
9. [x] **Experience update.** Prepend W747 learnings to `.trinity/experience.md`.
10. [x] **Current issue handoff.** Update `.trinity/current-issue.md` to W748.
11. [x] **Memory update.** Write
    `~/.claude/projects/-Users-playra-t27/memory/wave-loop-747.md` and append
    one-line pointer to `MEMORY.md`.
12. [ ] **Branch / merge.** Commit W747 with `Closes #1718`; create
    `wave-loop-748`.

## Decomposed backlog for next ring

1. [ ] **L1 cleanup.** Retroactively link or open issues for historical
       commits lacking `Closes #N`; enforce going forward.
2. [ ] **L4 backfill.** Add `test`/`invariant`/`bench` blocks to the 57
       missing-TDD specs, starting with `specs/numeric/gf*.t27` and
       `specs/tri/pipeline/*.t27` / `specs/tri/agent/*.t27`.
3. [ ] **L7 migration.** Retire or re-implement the 19 `scripts/*.sh` critical
       wrappers as `t27c` subcommands (e.g. `t27c verify`, `t27c reseal-apply`).
4. [ ] **FPGA SSOT reconciliation.** Align `CLAUDE.md`, `cli/dlc10`, and stale
       FPGA docs with `fpga/HARDWARE_SSOT.md`; decide whether to update `dlc10`
       or deprecate it.
5. [ ] **L6 reconciliation.** Resolve the `GF256` bias caveat in
       `conformance/FORMAT-SPEC-001.json`.
6. [ ] **Simulator stress wave.** Plan a Wave Loop near the ~4-MiBit packed
       vector boundary to measure Icarus/Yosys wall-clock limits.

## Risk register

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Generator copy forgets `MID_IDX` or module-name update | Medium | Low | Verify generated footer assertions and module header before running gates. |
| Icarus simulation path rejects `assert_ne` | High (known) | Low | Use `assert_eq` on changed elements only. |
| Mid-row expected value computed incorrectly | Low | Medium | Reuse corrected W632 formula: `e = r*64 + a5*32 + a4*16 + a3*8 + a2*4 + a1*2 + a0`. |
| Simulator capacity surprise near 0.612 MiBit | Very low | High | Direct `icarus-simulate` gate runs quickly; abort if elapsed exceeds prior waves significantly. |
| Offset-32768 identity check mischaracterised as wrap test | Medium | Low | Document in report that the check verifies period identity, not first-time wrap. |
| L1/L4/L7/L6 gaps block next major release | Medium | High | Prioritize in post-W747 backlog and assign ring stewards. |
| FPGA SSOT contradiction causes wrong physical target | Medium | High | Reconcile docs before any `dlc10` / openFPGALoader flashing. |
| Dirty worktree leaks into next commit | Medium | Medium | Leave `.agents`/`.codex` untracked; do not stage them. |

## Next Wave Loop 748 cooperation variants

1. **Variant A (recommended): `[315][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - 645,120-bit packed vector, 20,160 elements.
   - Continues the odd outer-dimension ladder and confirms non-p2 stride 315.
   - **Recommended.**

2. **Variant B: `[313][2]^6 Pt` bench-local (function-local) packed array var
   from call with indexed signed writes.**
   - Same width as W747 but tests the mutable `reg` declared inside a bench or
     function rather than at module scope.

3. **Variant C: `[313][2]^6 Pt` module-scope var with `if`-guarded indexed signed
   field writes.**
   - Stays at 0.612 MiBit and tests control-flow-guarded writes on a packed reg.

## Definition of done

- [x] `.claude/plans/wave-loop-747.md` exists and covers the above.
- [ ] `scripts/gen_w747.py` created with `OUTER = 313` and `MID_IDX = 156`.
- [ ] `specs/scratch/w747_bench_module_313x2p6_aos_var_call_write.t27` generated.
- [ ] Integration test `accepts_w747_bench_module_313x2p6_aos_var_call_write` added and passing.
- [ ] `cargo build --release -p t27c` passes.
- [ ] Direct `t27c` gates pass (parse, lowerable, simulate, cocotb, seal).
- [ ] All cargo test suites pass.
- [ ] Empty Icarus baseline saved.
- [ ] Closeout report, experience, and memory updated.
- [ ] Branch `wave-loop-748` created and W747 committed with `Closes #1718`.
