# Wave Loop 776 — Full-cycle decomposed plan

## Charter
Investigate repository weak points, research scientific literature, create a
decomposed plan, implement Wave Loop 776, write a closeout report with three
cooperation variants for Wave Loop 777, and save skills / experience.

## 1. Weak-point audit
- [x] Branch / worktree hygiene: `wave-loop-775` is current; W774 PR #1484 and
  W775 PR #1486 are open awaiting review, so W776 will branch from
  `wave-loop-775` HEAD to avoid blocking.
- [x] Traceability: 13 of 69 recent 30-day commit subjects carry
  `Closes #N`/`Fixes #N`/`Refs #N` (~18.8%). Subjects remain low, but
  closeout bodies still contain issue links.
- [x] Testability: 57 of 882 `.t27` specs lack `test`/`invariant`/`bench`
  (~6.5%). This is within the historical band but should be tracked.
- [x] UNITY: 19 `*.sh` scripts remain under `scripts/`. No new shell scripts
  will be added on the W776 critical path.
- [x] Worktree needles: untracked W485 scratch artifacts
  (`w485_bench_local_array_hoist.t27`, `w485_host_helper_shadow.t27`,
  `w485_wildcard_binding.t27`) and stale `.claude/plans/w485*` remain
  uncommitted and unrelated to the wave ladder.
- [x] CI weak points: pre-existing Yosys/Icarus FPGA synthesis formal failures
  (`sby` pip package missing, Yosys static-cast Verilog-2005 limitation in
  `build/fpga/generated/uart.v`) are unchanged and out of scope for this wave.

## 2. Literature scan (2025-2026)
- SystemVerilog packed array / struct semantics: IEEE 1800-2017 7.4.1/7.4.3
  confirm packed width is the product of dimensions, with no power-of-two
  restriction. AMD UG901 2026.1 lists packed arrays and packed structs as
  supported in Vivado synthesis/simulation.
- Tool gaps: Yosys 0.65-dev docs still note that arrays of packed structs/unions
  are not supported (YosysHQ/yosys#2677, #2908, #5837). t27's scalar
  packed-vector flattening avoids the unsupported construct entirely.
- Ternary/MVL ecosystem (IEEE ISMVL 2025/2026):
  - REBEL-6 — 32-trit balanced ternary ISA with RV32I-to-REBEL C compiler.
  - SONIC — event-driven gate-level ternary VLSI simulator with delta cycles.
  - TVHDL — balanced ternary extension to IEEE 1076-2008 VHDL.
  - VTX1 — open-source balanced-ternary SoC with Icarus/Yosys RTL-to-silicon flow.

## 3. Implementation plan
- Create `wave-loop-776` from `wave-loop-775` HEAD.
- Copy `scripts/gen_w775.py` to `scripts/gen_w776.py`.
- Update constants: `OUTER = 371`, `MID_IDX = 185`.
- Update module name prefix to `w776_bench_module_371x2p6_aos_var_call_write`.
- Run generator to produce
  `specs/scratch/w776_bench_module_371x2p6_aos_var_call_write.t27`.
- Add integration test `accepts_w776_bench_module_371x2p6_aos_var_call_write`
  in `bootstrap/tests/icarus_lowerable.rs` after the W775 test.
- Build `cargo build --release -p t27c`.
- Seal the witness with `t27c seal --save`.
- Verify: `parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
  `icarus-cocotb`.
- Run cargo suites: `cargo test -p t27c --bin t27c`, `cargo test -p tri`,
  `cargo test -p t27c --test icarus_lowerable`.
- Create GitHub issue for W776.
- Commit with `Closes #<issue>`.
- Push `wave-loop-776` and open PR.

## 4. Closeout / learning
- Write `docs/reports/FPGA_LOOP_CLOSEOUT_W776_2026-07-24.md`.
- Write `.claude/plans/wave-loop-777.md` with three cooperation variants.
- Update `.trinity/experience.md`, `.claude/skills/t27-wave-loop.md`,
  `.trinity/current-issue.md`, and `docs/NOW.md`.
- Save memory file `wave-loop-776.md` and prepend pointer in `MEMORY.md`.

## 5. Cooperation variants for Wave Loop 777
- **Variant A (recommended):** continue the odd outer-dimension ladder with
  `[373][2]^6 Pt` (~0.725 MiBit).
- **Variant B:** keep width at ~0.723 MiBit but move the packed var to
  bench/function scope at `[371][2]^6 Pt`.
- **Variant C:** add `if`-guarded indexed signed field writes at the current
  `[371][2]^6 Pt` width.
