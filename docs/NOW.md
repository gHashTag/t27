# NOW — Wave Loop 847 close-out / Wave Loop 848 setup (2026-08-04)

Last updated: 2026-08-04

## Wave Loop 847 — module-scope `[513][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1634)

- Branch: `wave-loop-847`
- Parent branch: `wave-loop-846` HEAD
- Issue: #1634
- PR: #1635
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W847_2026-08-04.md`
- Plan: `.claude/plans/wave-loop-848.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`

### What landed

- `specs/scratch/w847_bench_module_513x2p6_aos_var_call_write.t27`
  - 32,832 elements, 1,050,624-bit packed vector (~1.002 MiBit).
  - First wave to cross the 1-MiBit line.
  - Module-scope `pub var dst : [513][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w847.py`
  - Generator for the W847 witness; `OUTER = 513`, `MID_IDX = 256`.
  - Copy hazard fixed: destination path, module header f-string, and `MID_IDX`
    comment updated from stale `w846` / `511` / `255` references.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w847_bench_module_513x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w847_bench_module_513x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.

### Not changed

- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification

- `cargo build --release -p t27c`: OK (626 warnings, 0 errors).
- `cargo test --release --test icarus_lowerable accepts_w847_bench_module_513x2p6_aos_var_call_write`: 1/0.
- `cargo test --release --test icarus_lowerable` (full suite): 307/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W847: PASS.

### Research / weak points

- Icarus Verilog has no documented 1-MiBit hard cap; the LRM only requires 65,536-bit
  packed-array support and Icarus warns around 1 Gbit. Practical limits are memory
  dependent.
- Siracusa et al. (IEEE TC 2021) FPGA Roofline model frames the ladder as a probe
  of memory quanta `Q` growth.
- Vericert (Herklotz et al., OOPSLA 2021) and Vitis HLS UG1399 provide verified-HLS
  and commercial analogs for packed AoS/SoA mapping.

### Remaining weak points

- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, deeper
  compiler lowering issue, tracked for separate issue).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 626 release warnings need a dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).
- 30-day traceability by subject remains low; keep closing references in commit subjects.
- Generator copy hazard persists; parameterize `WAVE`/`OUTER` in the template.

---

## Wave Loop 848 — module-scope `[515][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-848`
- Parent branch: `wave-loop-847` HEAD (after closeout)
- Issue: #1636 (expected)
- PR: #1637 (expected)
- Plan: `.claude/plans/wave-loop-848.md`

### Goal

Continue the odd outer-dimension module-scope AoS ladder with `[515][2]^6 Pt`.
Expected 32,960 elements, 1,054,720-bit packed vector (~1.006 MiBit), still well
under the 4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants

- **A (recommended):** `[515][2]^6 Pt` module-scope var from call.
- **B:** `[513][3]^6 Pt` — grow second inner dimension to stress stride scaling.
- **C:** `[513][2]^6 Pt` with negative-index writes to exercise wrap-around.

---

*φ² + φ⁻² = 3 | TRINITY*
