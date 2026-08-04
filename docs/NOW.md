# NOW — Wave Loop 844 close-out / Wave Loop 845 setup (2026-08-04)

Last updated: 2026-08-04

## Wave Loop 844 — module-scope `[507][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1628)

- Branch: `wave-loop-844`
- Parent branch: `wave-loop-843` HEAD
- Issue: #1628
- PR: #1629
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W844_2026-08-04.md`
- Plan: `.claude/plans/wave-loop-845.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`

### What landed
- `specs/scratch/w844_bench_module_507x2p6_aos_var_call_write.t27`
  - 32,448 elements, 1,038,336-bit packed vector (~0.990 MiBit).
  - Module-scope `pub var dst : [507][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w844.py`
  - Generator for the W844 witness; `OUTER = 507`, `MID_IDX = 253`.
  - Copy hazard fixed: destination path, module header f-string, and `MID_IDX`
    comment updated from stale `w843` / `505` / `252` references.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w844_bench_module_507x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w844_bench_module_507x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK (626 warnings, 0 errors).
- `cargo test --release --test icarus_lowerable accepts_w844_bench_module_507x2p6_aos_var_call_write`: 1/0.
- `cargo test --release --test icarus_lowerable` (full suite): 304/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W844: PASS.

### Remaining weak points
- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, deeper
  compiler lowering issue, tracked for separate issue).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 626 release warnings need a dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).
- 30-day traceability by subject remains low; keep closing references in commit subjects.
- Generator copy hazard persists; parameterize `WAVE`/`OUTER` in the template.

---

## Wave Loop 845 — module-scope `[509][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-845`
- Parent branch: `wave-loop-844` HEAD (after closeout)
- Issue: #1630 (expected)
- PR: #1631 (expected)
- Plan: `.claude/plans/wave-loop-845.md`

### Goal
Continue the odd outer-dimension module-scope AoS ladder with `[509][2]^6 Pt`.
Expected 32,576 elements, 1,042,432-bit packed vector (~0.994 MiBit), still under
4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants
- **A (recommended):** `[509][2]^6 Pt` module-scope var from call.
- **B:** `[507][3]^6 Pt` — grow second inner dimension to stress stride scaling.
- **C:** `[507][2]^6 Pt` with negative-index writes to exercise wrap-around.

---

*φ² + φ⁻² = 3 | TRINITY*
