# NOW — Wave Loop 846 close-out / Wave Loop 847 setup (2026-08-04)

Last updated: 2026-08-04

## Wave Loop 846 — module-scope `[511][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1632)

- Branch: `wave-loop-846`
- Parent branch: `wave-loop-845` HEAD
- Issue: #1632
- PR: #1633
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W846_2026-08-04.md`
- Plan: `.claude/plans/wave-loop-847.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`

### What landed
- `specs/scratch/w846_bench_module_511x2p6_aos_var_call_write.t27`
  - 32,704 elements, 1,046,528-bit packed vector (~0.998 MiBit).
  - Module-scope `pub var dst : [511][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w846.py`
  - Generator for the W846 witness; `OUTER = 511`, `MID_IDX = 255`.
  - Copy hazard fixed: destination path, module header f-string, and `MID_IDX`
    comment updated from stale `w845` / `509` / `254` references.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w846_bench_module_511x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w846_bench_module_511x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK (626 warnings, 0 errors).
- `cargo test --release --test icarus_lowerable accepts_w846_bench_module_511x2p6_aos_var_call_write`: 1/0.
- `cargo test --release --test icarus_lowerable` (full suite): 306/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W846: PASS.

### Remaining weak points
- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, deeper
  compiler lowering issue, tracked for separate issue).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 626 release warnings need a dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).
- 30-day traceability by subject remains low; keep closing references in commit subjects.
- Generator copy hazard persists; parameterize `WAVE`/`OUTER` in the template.

---

## Wave Loop 847 — module-scope `[513][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-847`
- Parent branch: `wave-loop-846` HEAD (after closeout)
- Issue: #1634 (expected)
- PR: #1635 (expected)
- Plan: `.claude/plans/wave-loop-847.md`

### Goal
Continue the odd outer-dimension module-scope AoS ladder with `[513][2]^6 Pt`.
Expected 32,832 elements, 1,050,624-bit packed vector (~1.002 MiBit), crossing
the 1-MiBit line for the first time while remaining well under the 4-MiBit
cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants
- **A (recommended):** `[513][2]^6 Pt` module-scope var from call.
- **B:** `[511][3]^6 Pt` — grow second inner dimension to stress stride scaling.
- **C:** `[511][2]^6 Pt` with negative-index writes to exercise wrap-around.

---

*φ² + φ⁻² = 3 | TRINITY*
