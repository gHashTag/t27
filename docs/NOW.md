# NOW — Wave Loop 845 close-out / Wave Loop 846 setup (2026-08-04)

Last updated: 2026-08-04

## Wave Loop 845 — module-scope `[509][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1630)

- Branch: `wave-loop-845`
- Parent branch: `wave-loop-844` HEAD
- Issue: #1630
- PR: #1631
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W845_2026-08-04.md`
- Plan: `.claude/plans/wave-loop-846.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`

### What landed
- `specs/scratch/w845_bench_module_509x2p6_aos_var_call_write.t27`
  - 32,576 elements, 1,042,432-bit packed vector (~0.994 MiBit).
  - Module-scope `pub var dst : [509][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w845.py`
  - Generator for the W845 witness; `OUTER = 509`, `MID_IDX = 254`.
  - Copy hazard fixed: destination path, module header f-string, and `MID_IDX`
    comment updated from stale `w844` / `507` / `253` references.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w845_bench_module_509x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w845_bench_module_509x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK (626 warnings, 0 errors).
- `cargo test --release --test icarus_lowerable accepts_w845_bench_module_509x2p6_aos_var_call_write`: 1/0.
- `cargo test --release --test icarus_lowerable` (full suite): 305/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W845: PASS.

### Remaining weak points
- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, deeper
  compiler lowering issue, tracked for separate issue).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 626 release warnings need a dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).
- 30-day traceability by subject remains low; keep closing references in commit subjects.
- Generator copy hazard persists; parameterize `WAVE`/`OUTER` in the template.

---

## Wave Loop 846 — module-scope `[511][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-846`
- Parent branch: `wave-loop-845` HEAD (after closeout)
- Issue: #1632 (expected)
- PR: #1633 (expected)
- Plan: `.claude/plans/wave-loop-846.md`

### Goal
Continue the odd outer-dimension module-scope AoS ladder with `[511][2]^6 Pt`.
Expected 32,704 elements, 1,046,528-bit packed vector (~0.998 MiBit), still under
4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants
- **A (recommended):** `[511][2]^6 Pt` module-scope var from call.
- **B:** `[509][3]^6 Pt` — grow second inner dimension to stress stride scaling.
- **C:** `[509][2]^6 Pt` with negative-index writes to exercise wrap-around.

---

*φ² + φ⁻² = 3 | TRINITY*
