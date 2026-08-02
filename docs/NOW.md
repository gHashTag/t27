# NOW — Wave Loop 835 close-out / Wave Loop 836 setup (2026-08-01)

Last updated: 2026-08-01

## Wave Loop 835 — module-scope `[489][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1610)

- Branch: `wave-loop-835`
- Parent branch: `wave-loop-834` HEAD
- Issue: #1610
- PR: #1611
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W835_2026-08-01.md`
- Plan: `.claude/plans/wave-loop-836.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`

### What landed
- `specs/scratch/w835_bench_module_489x2p6_aos_var_call_write.t27`
  - 31,296 elements, 1,001,472-bit packed vector (~0.955 MiBit).
  - Module-scope `pub var dst : [489][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w835.py`
  - Generator for the W835 witness; `OUTER = 489`, `MID_IDX = 244`.
  - Copy hazard fixed: destination path, module header f-string, and `MID_IDX`
    comment updated from stale `w834` / `487` / `243` references.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w835_bench_module_489x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w835_bench_module_489x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-836.md`, `.claude/skills/wave-loop-autopilot.md`
  - W835 learnings saved and W836 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo test --release --test icarus_lowerable accepts_w835_bench_module_489x2p6_aos_var_call_write`: 1/0.
- `cargo test --release --test icarus_lowerable` (full suite): 295/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W835: PASS.

### Remaining weak points
- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, deeper
  compiler lowering issue, tracked for separate issue).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 626 release warnings need a dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).
- 30-day traceability by subject remains low; keep closing references in commit subjects.
- Generator copy hazard persists; parameterize `WAVE`/`OUTER` in the template.

---

## Wave Loop 836 — module-scope `[491][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-836`
- Parent branch: `wave-loop-835` HEAD (after closeout)
- Issue: #1612 (expected)
- PR: #1613 (expected)
- Plan: `.claude/plans/wave-loop-836.md`

### Goal
Continue the odd outer-dimension module-scope AoS ladder with `[491][2]^6 Pt`.
Expected 31,424 elements, 1,005,568-bit packed vector (~0.959 MiBit), still under
4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants
- **A (recommended):** `[491][2]^6 Pt` module-scope var from call.
- **B:** `[489][3]^6 Pt` — grow second inner dimension to stress stride scaling.
- **C:** `[489][2]^6 Pt` with negative-index writes to exercise wrap-around.

---

*φ² + φ⁻² = 3 | TRINITY*
