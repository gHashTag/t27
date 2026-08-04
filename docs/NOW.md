# NOW — Wave Loop 840 close-out / Wave Loop 841 setup (2026-08-04)

Last updated: 2026-08-04

## Wave Loop 840 — module-scope `[499][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1620)

- Branch: `wave-loop-840`
- Parent branch: `wave-loop-839` HEAD
- Issue: #1620
- PR: #1621
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W840_2026-08-04.md`
- Plan: `.claude/plans/wave-loop-841.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`

### What landed
- `specs/scratch/w840_bench_module_499x2p6_aos_var_call_write.t27`
  - 31,936 elements, 1,021,952-bit packed vector (~0.974 MiBit).
  - Module-scope `pub var dst : [499][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w840.py`
  - Generator for the W840 witness; `OUTER = 499`, `MID_IDX = 249`.
  - Copy hazard fixed: destination path, module header f-string, and `MID_IDX`
    comment updated from stale `w839` / `497` / `248` references.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w840_bench_module_499x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w840_bench_module_499x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK (627 warnings, 0 errors).
- `cargo test --release --test icarus_lowerable accepts_w840_bench_module_499x2p6_aos_var_call_write`: 1/0.
- `cargo test --release --test icarus_lowerable` (full suite): 300/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W840: PASS.

### Remaining weak points
- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, deeper
  compiler lowering issue, tracked for separate issue).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 627 release warnings need a dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).
- 30-day traceability by subject remains low; keep closing references in commit subjects.
- Generator copy hazard persists; parameterize `WAVE`/`OUTER` in the template.

---

## Wave Loop 841 — module-scope `[501][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-841`
- Parent branch: `wave-loop-840` HEAD (after closeout)
- Issue: #1622 (expected)
- PR: #1623 (expected)
- Plan: `.claude/plans/wave-loop-841.md`

### Goal
Continue the odd outer-dimension module-scope AoS ladder with `[501][2]^6 Pt`.
Expected 32,064 elements, 1,026,048-bit packed vector (~0.978 MiBit), still under
4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants
- **A (recommended):** `[501][2]^6 Pt` module-scope var from call.
- **B:** `[499][3]^6 Pt` — grow second inner dimension to stress stride scaling.
- **C:** `[499][2]^6 Pt` with negative-index writes to exercise wrap-around.

---

*φ² + φ⁻² = 3 | TRINITY*
