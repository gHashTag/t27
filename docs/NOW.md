# NOW — Wave Loop 849 close-out / Wave Loop 850 setup (2026-08-04)

Last updated: 2026-08-04

## Wave Loop 849 — module-scope `[517][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1638)

- Branch: `wave-loop-849`
- Parent branch: `wave-loop-848` HEAD
- Issue: #1638
- PR: #1639
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W849_2026-08-04.md`
- Plan: `.claude/plans/wave-loop-850.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`

### What landed

- `specs/scratch/w849_bench_module_517x2p6_aos_var_call_write.t27`
  - 33,088 elements, 1,058,816-bit packed vector (~1.010 MiBit).
  - Module-scope `pub var dst : [517][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w849.py`
  - Generator for the W849 witness; `OUTER = 517`, `MID_IDX = 258`.
  - Copy hazard fixed: destination path, module header f-string, and `MID_IDX`
    comment updated from stale `w848` / `515` / `257` references.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w849_bench_module_517x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w849_bench_module_517x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.

### Not changed

- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification

- `cargo build --release -p t27c`: OK (warnings, 0 errors).
- `cargo test --release --test icarus_lowerable accepts_w849_bench_module_517x2p6_aos_var_call_write`: 1/0.
- `cargo test --release --test icarus_lowerable` (full suite): 309/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W849: PASS.

### Research / weak points

- Icarus Verilog has no documented 1-MiBit hard cap; the LRM only requires 65,536-bit
  packed-array support and Icarus warns around 1 Gbit. Upstream commit `128c621`
  fixed a bound-normalization path that could accidentally create billion-bit vectors.
  Historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed vectors;
  modern versions do not hit it.
- Siracusa et al. (IEEE TC 2021) FPGA Roofline model frames the ladder as a probe
  of memory quanta `Q` growth.
- Vericert/CompCert provide verified-compilation analogs for bit-exact source-to-hardware
  mapping.
- Vitis HLS UG1399 provides the commercial analog for packed interface structs.

### Remaining weak points

- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, deeper
  compiler lowering issue, tracked for separate issue).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- Release warnings need a dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).
- 30-day traceability by subject remains low; keep closing references in commit subjects.
- Generator copy hazard persists; parameterize `WAVE`/`OUTER` in the template.

---

## Wave Loop 850 — module-scope `[519][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-850`
- Parent branch: `wave-loop-849` HEAD (after closeout)
- Issue: #1640 (expected)
- PR: #1641 (expected)
- Plan: `.claude/plans/wave-loop-850.md`

### Goal

Continue the odd outer-dimension module-scope AoS ladder with `[519][2]^6 Pt`.
Expected 33,216 elements, 1,062,912-bit packed vector (~1.014 MiBit), still well
under the 4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants

- **A (recommended):** `[519][2]^6 Pt` module-scope var from call.
- **B:** `[517][3]^6 Pt` — grow second inner dimension to stress stride scaling.
- **C:** `[517][2]^6 Pt` with negative-index writes to exercise wrap-around.

---

*φ² + φ⁻² = 3 | TRINITY*
