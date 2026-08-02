# NOW — Wave Loop 837 close-out / Wave Loop 838 setup (2026-08-01)

Last updated: 2026-08-01

## Wave Loop 837 — module-scope `[493][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1614)

- Branch: `wave-loop-837`
- Parent branch: `wave-loop-836` HEAD
- Issue: #1614
- PR: #1615
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W837_2026-08-01.md`
- Plan: `.claude/plans/wave-loop-838.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`

### What landed
- `specs/scratch/w837_bench_module_493x2p6_aos_var_call_write.t27`
  - 31,552 elements, 1,009,664-bit packed vector (~0.963 MiBit).
  - Module-scope `pub var dst : [493][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w837.py`
  - Generator for the W837 witness; `OUTER = 493`, `MID_IDX = 246`.
  - Copy hazard fixed: destination path, module header f-string, and `MID_IDX`
    comment updated from stale `w836` / `491` / `245` references.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w837_bench_module_493x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w837_bench_module_493x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-838.md`, `.claude/skills/wave-loop-autopilot.md`
  - W837 learnings saved and W838 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo test --release --test icarus_lowerable accepts_w837_bench_module_493x2p6_aos_var_call_write`: 1/0.
- `cargo test --release --test icarus_lowerable` (full suite): 297/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W837: PASS.

### Remaining weak points
- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, deeper
  compiler lowering issue, tracked for separate issue).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 626 release warnings need a dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).
- 30-day traceability by subject remains low; keep closing references in commit subjects.
- Generator copy hazard persists; parameterize `WAVE`/`OUTER` in the template.

---

## Wave Loop 838 — module-scope `[495][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-838`
- Parent branch: `wave-loop-837` HEAD (after closeout)
- Issue: #1616 (expected)
- PR: #1617 (expected)
- Plan: `.claude/plans/wave-loop-838.md`

### Goal
Continue the odd outer-dimension module-scope AoS ladder with `[495][2]^6 Pt`.
Expected 31,680 elements, 1,013,760-bit packed vector (~0.967 MiBit), still under
4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants
- **A (recommended):** `[495][2]^6 Pt` module-scope var from call.
- **B:** `[493][3]^6 Pt` — grow second inner dimension to stress stride scaling.
- **C:** `[493][2]^6 Pt` with negative-index writes to exercise wrap-around.

---

*φ² + φ⁻² = 3 | TRINITY*
