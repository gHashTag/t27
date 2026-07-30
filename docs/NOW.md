# NOW — Wave Loop 818 close-out / Wave Loop 819 setup (2026-07-29)

Last updated: 2026-07-29

## Wave Loop 818 — module-scope `[455][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1564)

- Branch: `wave-loop-818`
- Parent branch: `wave-loop-817` HEAD (`c2c6e5dcd`)
- Issue: #1564
- PR: #1566
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W818_2026-07-29.md`
- Plan: `.claude/plans/wave-loop-818.md`
- Cooperation W819: `.claude/plans/wave-loop-819.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`

### What landed
- `specs/scratch/w818_bench_module_455x2p6_aos_var_call_write.t27`
  - 29,120 elements, 931,840-bit packed vector (~0.889 MiBit).
  - Module-scope `pub var dst : [455][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w818.py`
  - Generator for the W818 witness; `OUTER = 455`, `MID_IDX = 227`.
  - Note: both the destination path and the module header f-string were manually
    fixed after copying from W817 (generator copy hazard). The `MID_IDX` comment
    was also corrected to `227`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w818_bench_module_455x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w818_bench_module_455x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-819.md`, `.claude/skills/wave-loop-autopilot.md`
  - W818 learnings saved and W819 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo clippy -p t27c`: OK (626 warnings, 0 errors).
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p flash-spi`: 2/0.
- `cargo test -p t27c --test bitnet_pipeline`: 20/0.
- `cargo test -p t27c --test bitnet_top`: 17/0.
- `cargo test -p t27c --test icarus_lowerable`: 278/0.
- `cargo test -p t27c --test verilog_const_array`: 2/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W818: PASS.

### Remaining weak points
- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, deeper
  compiler lowering issue, tracked for separate issue).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 627 release warnings and 626 clippy warnings need dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).
- 30-day traceability by subject remains low; keep closing references in commit subjects.
- Generator copy hazard: parameterize destination path and module header in the
  generator template to eliminate stale wave-number references on copy.

---

## Wave Loop 819 — module-scope `[457][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-819`
- Parent branch: `wave-loop-818` HEAD (after closeout)
- Issue: #1565 (to open)
- PR: (to open)
- Plan: `.claude/plans/wave-loop-819.md`

### Goal
Continue the odd outer-dimension module-scope AoS ladder with `[457][2]^6 Pt`.
Expected 29,184 elements, 933,888-bit packed vector (~0.891 MiBit), still under
4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants
- **A (recommended):** `[457][2]^6 Pt` module-scope var from call.
- **B:** `[455][3]^6 Pt` — grow second inner dimension to stress stride scaling.
- **C:** `[455][2]^6 Pt` with negative-index writes to exercise wrap-around.

---

*φ² + φ⁻² = 3 | TRINITY*
