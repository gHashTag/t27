# NOW — Wave Loop 831 close-out / Wave Loop 832 setup (2026-08-01)

Last updated: 2026-08-01

## Wave Loop 831 — module-scope `[481][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1603)

- Branch: `wave-loop-831`
- Parent branch: `wave-loop-830` HEAD (`c068100`)
- Issue: #1603
- PR: #1603
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W831_2026-08-01.md`
- Plan: `.claude/plans/wave-loop-832.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`

### What landed
- `specs/scratch/w831_bench_module_481x2p6_aos_var_call_write.t27`
  - 30,784 elements, 985,088-bit packed vector (~0.939 MiBit).
  - Module-scope `pub var dst : [481][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w831.py`
  - Generator for the W831 witness; `OUTER = 481`, `MID_IDX = 240`.
  - Copy hazard fixed: destination path, module header f-string, and `MID_IDX`
    comment updated from stale `w830` / `479` / `239` references.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w831_bench_module_481x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w831_bench_module_481x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-832.md`, `.claude/skills/wave-loop-autopilot.md`
  - W831 learnings saved and W832 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo test --release --test icarus_lowerable accepts_w831_bench_module_481x2p6_aos_var_call_write`: 1/0.
- `cargo test --release --test icarus_lowerable` (full suite): 291/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W831: PASS.

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

## Wave Loop 832 — module-scope `[483][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-832`
- Parent branch: `wave-loop-831` HEAD (after closeout)
- Issue: #1605 (expected)
- PR: #1606 (expected)
- Plan: `.claude/plans/wave-loop-832.md`

### Goal
Continue the odd outer-dimension module-scope AoS ladder with `[483][2]^6 Pt`.
Expected 31,040 elements, 993,280-bit packed vector (~0.947 MiBit), still under
4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants
- **A (recommended):** `[483][2]^6 Pt` module-scope var from call.
- **B:** `[481][3]^6 Pt` — grow second inner dimension to stress stride scaling.
- **C:** `[481][2]^6 Pt` with negative-index writes to exercise wrap-around.

---

*φ² + φ⁻² = 3 | TRINITY*
