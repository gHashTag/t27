# NOW — Wave Loop 816 close-out / Wave Loop 817 setup (2026-07-29)

Last updated: 2026-07-29

## Wave Loop 816 — module-scope `[451][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1561)

- Branch: `wave-loop-816`
- Parent branch: `wave-loop-815` HEAD (`e7d0c594d`)
- Issue: #1561
- PR: #1560
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W816_2026-07-29.md`
- Plan: `.claude/plans/wave-loop-816.md`
- Cooperation W817: `.claude/plans/wave-loop-817.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`

### What landed
- `specs/scratch/w816_bench_module_451x2p6_aos_var_call_write.t27`
  - 28,864 elements, 923,648-bit packed vector (~0.881 MiBit).
  - Module-scope `pub var dst : [451][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w816.py`
  - Generator for the W816 witness; `OUTER = 451`, `MID_IDX = 225`.
  - Note: both the destination path and the module header f-string were manually
    fixed after copying from W815 (generator copy hazard). The `MID_IDX` comment
    was also corrected to `225`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w816_bench_module_451x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w816_bench_module_451x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-817.md`, `.claude/skills/wave-loop-autopilot.md`
  - W816 learnings saved and W817 plan/cooperation variants created.

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
- `cargo test -p t27c --test icarus_lowerable`: 276/0.
- `cargo test -p t27c --test verilog_const_array`: 2/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W816: PASS.

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

## Wave Loop 817 — module-scope `[453][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-817`
- Parent branch: `wave-loop-816` HEAD (after closeout)
- Issue: #1563 (to open)
- PR: (to open)
- Plan: `.claude/plans/wave-loop-817.md`

### Goal
Continue the odd outer-dimension module-scope AoS ladder with `[453][2]^6 Pt`.
Expected 28,992 elements, 927,744-bit packed vector (~0.885 MiBit), still under
4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants
- **A (recommended):** `[453][2]^6 Pt` module-scope var from call.
- **B:** `[451][3]^6 Pt` — grow second inner dimension to stress stride scaling.
- **C:** `[451][2]^6 Pt` with negative-index writes to exercise wrap-around.

---

*φ² + φ⁻² = 3 | TRINITY*
