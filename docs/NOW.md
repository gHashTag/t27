# NOW — Wave Loop 828 close-out / Wave Loop 829 setup (2026-08-01)

Last updated: 2026-08-01

## Wave Loop 828 — module-scope `[475][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1597)

- Branch: `wave-loop-828`
- Parent branch: `wave-loop-827` HEAD (`5febd15`)
- Issue: #1597
- PR: #1598
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W828_2026-08-01.md`
- Plan: `.claude/plans/wave-loop-829.md`
- Cooperation W829: `.claude/plans/wave-loop-829.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`

### What landed
- `specs/scratch/w828_bench_module_475x2p6_aos_var_call_write.t27`
  - 30,400 elements, 972,800-bit packed vector (~0.927 MiBit).
  - Module-scope `pub var dst : [475][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w828.py`
  - Generator for the W828 witness; `OUTER = 475`, `MID_IDX = 237`.
  - Note: both the destination path and the module header f-string were manually
    fixed after copying from W827 (generator copy hazard). The `MID_IDX` comment
    was also corrected to `237`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w828_bench_module_475x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w828_bench_module_475x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-829.md`, `.claude/skills/wave-loop-autopilot.md`
  - W828 learnings saved and W829 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo test --release --test icarus_lowerable accepts_w828_bench_module_475x2p6_aos_var_call_write`: 1/0.
- `cargo test --release --test icarus_lowerable` (full suite): 288/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W828: PASS.

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

## Wave Loop 829 — module-scope `[477][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-829`
- Parent branch: `wave-loop-828` HEAD (after closeout)
- Issue: #1599 (expected)
- PR: #1600 (expected)
- Plan: `.claude/plans/wave-loop-829.md`

### Goal
Continue the odd outer-dimension module-scope AoS ladder with `[477][2]^6 Pt`.
Expected 30,528 elements, 980,992-bit packed vector (~0.934 MiBit), still under
4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants
- **A (recommended):** `[477][2]^6 Pt` module-scope var from call.
- **B:** `[475][3]^6 Pt` — grow second inner dimension to stress stride scaling.
- **C:** `[475][2]^6 Pt` with negative-index writes to exercise wrap-around.

---

*φ² + φ⁻² = 3 | TRINITY*
