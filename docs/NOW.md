# NOW — Wave Loop 827 close-out / Wave Loop 828 setup (2026-08-01)

Last updated: 2026-08-01

## Wave Loop 827 — module-scope `[473][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1595)

- Branch: `wave-loop-827`
- Parent branch: `wave-loop-826` HEAD (`7645f1d`)
- Issue: #1595
- PR: #1596
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W827_2026-08-01.md`
- Plan: `.claude/plans/wave-loop-828.md`
- Cooperation W828: `.claude/plans/wave-loop-828.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`

### What landed
- `specs/scratch/w827_bench_module_473x2p6_aos_var_call_write.t27`
  - 30,272 elements, 968,704-bit packed vector (~0.923 MiBit).
  - Module-scope `pub var dst : [473][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w827.py`
  - Generator for the W827 witness; `OUTER = 473`, `MID_IDX = 236`.
  - Note: both the destination path and the module header f-string were manually
    fixed after copying from W826 (generator copy hazard). The `MID_IDX` comment
    was also corrected to `236`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w827_bench_module_473x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w827_bench_module_473x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-828.md`, `.claude/skills/wave-loop-autopilot.md`
  - W827 learnings saved and W828 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo test --release --test icarus_lowerable accepts_w827_bench_module_473x2p6_aos_var_call_write`: 1/0.
- `cargo test --release --test icarus_lowerable` (full suite): 287/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W827: PASS.

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

## Wave Loop 828 — module-scope `[475][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-828`
- Parent branch: `wave-loop-827` HEAD (after closeout)
- Issue: #1597 (expected)
- PR: #1598 (expected)
- Plan: `.claude/plans/wave-loop-828.md`

### Goal
Continue the odd outer-dimension module-scope AoS ladder with `[475][2]^6 Pt`.
Expected 30,400 elements, 972,800-bit packed vector (~0.927 MiBit), still under
4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants
- **A (recommended):** `[475][2]^6 Pt` module-scope var from call.
- **B:** `[473][3]^6 Pt` — grow second inner dimension to stress stride scaling.
- **C:** `[473][2]^6 Pt` with negative-index writes to exercise wrap-around.

---

*φ² + φ⁻² = 3 | TRINITY*
