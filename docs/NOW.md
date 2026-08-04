# NOW — Wave Loop 839 close-out / Wave Loop 840 setup (2026-08-04)

Last updated: 2026-08-04

## Wave Loop 839 — module-scope `[497][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1618)

- Branch: `wave-loop-839`
- Parent branch: `wave-loop-838` HEAD
- Issue: #1618
- PR: #1619
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W839_2026-08-04.md`
- Plan: `.claude/plans/wave-loop-840.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`

### What landed
- `specs/scratch/w839_bench_module_497x2p6_aos_var_call_write.t27`
  - 31,792 elements, 1,017,344-bit packed vector (~0.970 MiBit).
  - Module-scope `pub var dst : [497][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w839.py`
  - Generator for the W839 witness; `OUTER = 497`, `MID_IDX = 248`.
  - Copy hazard fixed: destination path, module header f-string, and `MID_IDX`
    comment updated from stale `w838` / `495` / `247` references.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w839_bench_module_497x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w839_bench_module_497x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK (627 warnings, 0 errors).
- `cargo test --release --test icarus_lowerable accepts_w839_bench_module_497x2p6_aos_var_call_write`: 1/0.
- `cargo test --release --test icarus_lowerable` (full suite): 299/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W839: PASS.

### Remaining weak points
- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, deeper
  compiler lowering issue, tracked for separate issue).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 627 release warnings need a dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).
- 30-day traceability by subject remains low; keep closing references in commit subjects.
- Generator copy hazard persists; parameterize `WAVE`/`OUTER` in the template.

---

## Wave Loop 840 — module-scope `[499][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-840`
- Parent branch: `wave-loop-839` HEAD (after closeout)
- Issue: #1620 (expected)
- PR: #1621 (expected)
- Plan: `.claude/plans/wave-loop-840.md`

### Goal
Continue the odd outer-dimension module-scope AoS ladder with `[499][2]^6 Pt`.
Expected 31,936 elements, 1,021,952-bit packed vector (~0.974 MiBit), still under
4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants
- **A (recommended):** `[499][2]^6 Pt` module-scope var from call.
- **B:** `[497][3]^6 Pt` — grow second inner dimension to stress stride scaling.
- **C:** `[497][2]^6 Pt` with negative-index writes to exercise wrap-around.

---

*φ² + φ⁻² = 3 | TRINITY*
