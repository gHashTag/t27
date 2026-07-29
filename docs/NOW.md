# NOW — Wave Loop 815 close-out / Wave Loop 816 setup (2026-07-29)

Last updated: 2026-07-29

## Wave Loop 815 — module-scope `[449][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1559)

- Branch: `wave-loop-815`
- Parent branch: `wave-loop-814` HEAD (`315a82d5c`)
- Issue: #1559
- PR: (to open)
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W815_2026-07-29.md`
- Plan: `.claude/plans/wave-loop-815.md`
- Cooperation W816: `.claude/plans/wave-loop-816.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`

### What landed
- `specs/scratch/w815_bench_module_449x2p6_aos_var_call_write.t27`
  - 28,736 elements, 919,552-bit packed vector (~0.877 MiBit).
  - Module-scope `pub var dst : [449][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w815.py`
  - Generator for the W815 witness; `OUTER = 449`, `MID_IDX = 224`.
  - Note: both the destination path and the module header f-string were manually
    fixed after copying from W814 (generator copy hazard). The `MID_IDX` comment
    was also corrected to `224`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w815_bench_module_449x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w815_bench_module_449x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-816.md`, `.claude/skills/wave-loop-autopilot.md`
  - W815 learnings saved and W816 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo clippy -p t27c`: OK (780 warnings, 0 errors).
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p flash-spi`: 2/0.
- `cargo test -p t27c --test bitnet_pipeline`: 20/0.
- `cargo test -p t27c --test bitnet_top`: 17/0.
- `cargo test -p t27c --test icarus_lowerable`: 275/0.
- `cargo test -p t27c --test verilog_const_array`: 2/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W815: PASS.

### Remaining weak points
- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, deeper
  compiler lowering issue, tracked for separate issue).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 626 release warnings and 780 clippy warnings need dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).
- 30-day traceability by subject remains low; keep closing references in commit subjects.
- Generator copy hazard: parameterize destination path and module header in the
  generator template to eliminate stale wave-number references on copy.

---

## Wave Loop 816 — module-scope `[451][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-816`
- Parent branch: `wave-loop-815` HEAD (after closeout)
- Issue: #1561 (to open)
- PR: (to open)
- Plan: `.claude/plans/wave-loop-816.md`

### Goal
Continue the odd outer-dimension module-scope AoS ladder with `[451][2]^6 Pt`.
Expected 28,864 elements, 923,648-bit packed vector (~0.881 MiBit), still under
4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants
- **A (recommended):** `[451][2]^6 Pt` module-scope var from call.
- **B:** `[449][3]^6 Pt` — grow second inner dimension to stress stride scaling.
- **C:** `[449][2]^6 Pt` with negative-index writes to exercise wrap-around.

---

*φ² + φ⁻² = 3 | TRINITY*
