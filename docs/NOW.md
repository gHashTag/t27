# NOW — Wave Loop 824 close-out / Wave Loop 825 setup (2026-08-01)

Last updated: 2026-08-01

## Wave Loop 824 — module-scope `[467][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1587)

- Branch: `wave-loop-824`
- Parent branch: `wave-loop-823` HEAD (`b032fe471`)
- Issue: #1587
- PR: #1588
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W824_2026-08-01.md`
- Plan: `.claude/plans/wave-loop-825.md`
- Cooperation W825: `.claude/plans/wave-loop-825.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`

### What landed
- `specs/scratch/w824_bench_module_467x2p6_aos_var_call_write.t27`
  - 29,888 elements, 956,416-bit packed vector (~0.912 MiBit).
  - Module-scope `pub var dst : [467][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w824.py`
  - Generator for the W824 witness; `OUTER = 467`, `MID_IDX = 233`.
  - Note: both the destination path and the module header f-string were manually
    fixed after copying from W823 (generator copy hazard). The `MID_IDX` comment
    was also corrected to `233`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w824_bench_module_467x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w824_bench_module_467x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-825.md`, `.claude/skills/wave-loop-autopilot.md`
  - W824 learnings saved and W825 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo test --release --test icarus_lowerable accepts_w824_bench_module_467x2p6_aos_var_call_write`: 1/0.
- `cargo test --release --test icarus_lowerable` (full suite): 284/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W824: PASS.

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

## Wave Loop 825 — module-scope `[469][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-825`
- Parent branch: `wave-loop-824` HEAD (after closeout)
- Issue: TBD (to open)
- PR: (to open)
- Plan: `.claude/plans/wave-loop-825.md`

### Goal
Continue the odd outer-dimension module-scope AoS ladder with `[469][2]^6 Pt`.
Expected 30,016 elements, 960,512-bit packed vector (~0.916 MiBit), still under
4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants
- **A (recommended):** `[469][2]^6 Pt` module-scope var from call.
- **B:** `[467][3]^6 Pt` — grow second inner dimension to stress stride scaling.
- **C:** `[467][2]^6 Pt` with negative-index writes to exercise wrap-around.

---

*φ² + φ⁻² = 3 | TRINITY*
