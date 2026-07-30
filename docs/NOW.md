# NOW — Wave Loop 819 close-out / Wave Loop 820 setup (2026-07-29)

Last updated: 2026-07-29

## Wave Loop 819 — module-scope `[457][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1565)

- Branch: `wave-loop-819`
- Parent branch: `wave-loop-818` HEAD (`0d2aaedbe`)
- Issue: #1565
- PR: #1566
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W819_2026-07-29.md`
- Plan: `.claude/plans/wave-loop-819.md`
- Cooperation W820: `.claude/plans/wave-loop-820.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`

### What landed
- `specs/scratch/w819_bench_module_457x2p6_aos_var_call_write.t27`
  - 29,184 elements, 933,888-bit packed vector (~0.891 MiBit).
  - Module-scope `pub var dst : [457][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w819.py`
  - Generator for the W819 witness; `OUTER = 457`, `MID_IDX = 228`.
  - Note: both the destination path and the module header f-string were manually
    fixed after copying from W818 (generator copy hazard). The `MID_IDX` comment
    was also corrected to `228`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w819_bench_module_457x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w819_bench_module_457x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-820.md`, `.claude/skills/wave-loop-autopilot.md`
  - W819 learnings saved and W820 plan/cooperation variants created.

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
- `cargo test -p t27c --test icarus_lowerable`: 279/0.
- `cargo test -p t27c --test verilog_const_array`: 2/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W819: PASS.

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

## Wave Loop 820 — module-scope `[459][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-820`
- Parent branch: `wave-loop-819` HEAD (after closeout)
- Issue: #1567 (to open)
- PR: (to open)
- Plan: `.claude/plans/wave-loop-820.md`

### Goal
Continue the odd outer-dimension module-scope AoS ladder with `[459][2]^6 Pt`.
Expected 29,376 elements, 940,032-bit packed vector (~0.897 MiBit), still under
4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants
- **A (recommended):** `[459][2]^6 Pt` module-scope var from call.
- **B:** `[457][3]^6 Pt` — grow second inner dimension to stress stride scaling.
- **C:** `[457][2]^6 Pt` with negative-index writes to exercise wrap-around.

---

*φ² + φ⁻² = 3 | TRINITY*
