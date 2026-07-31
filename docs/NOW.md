# NOW — Wave Loop 823 close-out / Wave Loop 824 setup (2026-07-29)

Last updated: 2026-07-29

## Wave Loop 823 — module-scope `[465][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1585)

- Branch: `wave-loop-823`
- Parent branch: `wave-loop-822` HEAD (`fd1ef6dbe`)
- Issue: #1585
- PR: TBD
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W823_2026-07-29.md`
- Plan: `.claude/plans/wave-loop-824.md`
- Cooperation W824: `.claude/plans/wave-loop-824.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`

### What landed
- `specs/scratch/w823_bench_module_465x2p6_aos_var_call_write.t27`
  - 29,760 elements, 952,320-bit packed vector (~0.908 MiBit).
  - Module-scope `pub var dst : [465][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w823.py`
  - Generator for the W823 witness; `OUTER = 465`, `MID_IDX = 232`.
  - Note: both the destination path and the module header f-string were manually
    fixed after copying from W822 (generator copy hazard). The `MID_IDX` comment
    was also corrected to `232`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w823_bench_module_465x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w823_bench_module_465x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-824.md`, `.claude/skills/wave-loop-autopilot.md`
  - W823 learnings saved and W824 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo test --release --test icarus_lowerable accepts_w823_bench_module_465x2p6_aos_var_call_write`: 1/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W823: PASS.

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

## Wave Loop 824 — module-scope `[467][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-824`
- Parent branch: `wave-loop-823` HEAD (after closeout)
- Issue: TBD (to open)
- PR: (to open)
- Plan: `.claude/plans/wave-loop-824.md`

### Goal
Continue the odd outer-dimension module-scope AoS ladder with `[467][2]^6 Pt`.
Expected 29,888 elements, 956,416-bit packed vector (~0.912 MiBit), still under
4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants
- **A (recommended):** `[467][2]^6 Pt` module-scope var from call.
- **B:** `[465][3]^6 Pt` — grow second inner dimension to stress stride scaling.
- **C:** `[465][2]^6 Pt` with negative-index writes to exercise wrap-around.

---

*φ² + φ⁻² = 3 | TRINITY*
