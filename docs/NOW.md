# NOW — Wave Loop 807 close-out / Wave Loop 808 setup (2026-07-24)

Last updated: 2026-07-24

## Wave Loop 807 — module-scope `[433][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1543)

- Branch: `wave-loop-807`
- Parent branch: `wave-loop-806` HEAD (`7729d479f`)
- Issue: #1543
- PR: (to open)
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W807_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-807.md`
- Cooperation W808: `.claude/plans/wave-loop-808.md`

### What landed
- `specs/scratch/w807_bench_module_433x2p6_aos_var_call_write.t27`
  - 27,712 elements, 886,784-bit packed vector (~0.845 MiBit).
  - Module-scope `pub var dst : [433][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w807.py`
  - Generator for the W807 witness; `OUTER = 433`, `MID_IDX = 216`.
  - Note: both the destination path and the module header f-string were manually
    fixed after copying from W806 (generator copy hazard).
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w807_bench_module_433x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w807_bench_module_433x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-808.md`
  - W807 learnings saved and W808 plan/cooperation variants created.

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
- `cargo test -p t27c --test icarus_lowerable`: 267/0.
- `cargo test -p t27c --test verilog_const_array`: 2/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W807: PASS.

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

## Wave Loop 808 — module-scope `[435][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-808`
- Parent branch: `wave-loop-807` HEAD (after closeout)
- Issue: #1545 (to open)
- PR: (to open)
- Plan: `.claude/plans/wave-loop-808.md`

### Goal
Continue the odd outer-dimension module-scope AoS ladder with `[435][2]^6 Pt`.
Expected 27,840 elements, 890,880-bit packed vector (~0.849 MiBit), still under
4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants
- **A (recommended):** `[435][2]^6 Pt` module-scope var from call.
- **B:** `[433][3]^6 Pt` — grow second inner dimension to stress stride scaling.
- **C:** `[433][2]^6 Pt` with negative-index writes to exercise wrap-around.

---
