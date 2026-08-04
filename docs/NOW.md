# NOW — Wave Loop 851 close-out / Wave Loop 852 setup (2026-08-04)

Last updated: 2026-08-04

## Wave Loop 851 — module-scope `[521][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1642)

- Branch: `wave-loop-851`
- Parent branch: `wave-loop-850` HEAD
- Issue: #1642
- PR: #1643
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W851_2026-08-04.md`
- Plan: `.claude/plans/wave-loop-852.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`
- Master plan: `.claude/skills/wave-loop-master-plan.md`

### What landed

- `specs/scratch/w851_bench_module_521x2p6_aos_var_call_write.t27`
  - 33,344 elements, 1,067,008-bit packed vector (~1.018 MiBit).
  - Module-scope `pub var dst : [521][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w851.py`
  - Generator for the W851 witness; `OUTER = 521`, `MID_IDX = 260`.
  - Copy hazard fixed: destination path, module header f-string, and `MID_IDX`
    comment updated from stale `w850` / `519` / `259` references.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w851_bench_module_521x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w851_bench_module_521x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.

### Not changed

- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification

- `cargo build --release -p t27c`: OK (warnings, 0 errors).
- `cargo test --release --test icarus_lowerable accepts_w851_bench_module_521x2p6_aos_var_call_write`: 1/0.
- `cargo test --release --test icarus_lowerable` (full suite): 311/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W851: PASS.

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
- Full `./scripts/tri test` suite stalls on the pre-existing `w589_bench_module_17d_aos_var_call_write.t27` parse phase and was not completed this wave.

---

## Wave Loop 852 — module-scope `[523][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-852`
- Parent branch: `wave-loop-851` HEAD (after closeout)
- Issue: #1644 (created)
- PR: #1645 (expected)
- Plan: `.claude/plans/wave-loop-852.md`
- Master plan: `.claude/skills/wave-loop-master-plan.md`

### Goal

Continue the odd outer-dimension module-scope AoS ladder with `[523][2]^6 Pt`.
Expected 33,472 elements, 1,071,104-bit packed vector (~1.022 MiBit), still well
under the 4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants

- **A (recommended):** `[523][2]^6 Pt` module-scope var from call.
- **B:** `[521][3]^6 Pt` — grow second inner dimension to stress stride scaling.
- **C:** `[521][2]^6 Pt` with negative-index writes to exercise wrap-around.

---

*φ² + φ⁻² = 3 | TRINITY*
