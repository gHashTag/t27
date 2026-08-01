# NOW — Wave Loop 830 close-out / Wave Loop 831 setup (2026-08-01)

Last updated: 2026-08-01

## Wave Loop 830 — module-scope `[479][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1601)

- Branch: `wave-loop-830`
- Parent branch: `wave-loop-829` HEAD (`f0ebad1`)
- Issue: #1601
- PR: #1602
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W830_2026-08-01.md`
- Plan: `.claude/plans/wave-loop-831.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`

### What landed
- `specs/scratch/w830_bench_module_479x2p6_aos_var_call_write.t27`
  - 30,656 elements, 980,992-bit packed vector (~0.935 MiBit).
  - Module-scope `pub var dst : [479][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w830.py`
  - Generator for the W830 witness; `OUTER = 479`, `MID_IDX = 239`.
  - Copy hazard fixed: destination path, module header f-string, and `MID_IDX`
    comment updated from stale `w829` / `477` / `238` references.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w830_bench_module_479x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w830_bench_module_479x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-831.md`, `.claude/skills/wave-loop-autopilot.md`
  - W830 learnings saved and W831 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo test --release --test icarus_lowerable accepts_w830_bench_module_479x2p6_aos_var_call_write`: 1/0.
- `cargo test --release --test icarus_lowerable` (full suite): 290/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W830: PASS.

### Remaining weak points
- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, deeper
  compiler lowering issue, tracked for separate issue).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 626 release warnings need a dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).
- 30-day traceability by subject remains low; keep closing references in commit subjects.
- Generator copy hazard: parameterize destination path and module header in the
  generator template to eliminate stale wave-number references on copy.

---

## Wave Loop 831 — module-scope `[481][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-831`
- Parent branch: `wave-loop-830` HEAD (after closeout)
- Issue: #1603 (expected)
- PR: #1604 (expected)
- Plan: `.claude/plans/wave-loop-831.md`

### Goal
Continue the odd outer-dimension module-scope AoS ladder with `[481][2]^6 Pt`.
Expected 30,784 elements, 985,088-bit packed vector (~0.939 MiBit), still under
4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants
- **A (recommended):** `[481][2]^6 Pt` module-scope var from call.
- **B:** `[479][3]^6 Pt` — grow second inner dimension to stress stride scaling.
- **C:** `[479][2]^6 Pt` with negative-index writes to exercise wrap-around.

---

*φ² + φ⁻² = 3 | TRINITY*
