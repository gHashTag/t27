# NOW — Wave Loop 858 close-out / Wave Loop 859 setup (2026-08-05)

Last updated: 2026-08-05

## Wave Loop 858 — module-scope `[535][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1656)

- Branch: `wave-loop-858`
- Parent branch: `wave-loop-857` HEAD
- Issue: #1656
- PR: TBD
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W858_2026-08-05.md`
- Plan: `.claude/plans/wave-loop-859.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`
- Master plan: `.claude/skills/wave-loop-master-plan.md`

### What landed

- `specs/scratch/w858_bench_module_535x2p6_aos_var_call_write.t27`
  - 34,240 elements, 1,095,680-bit packed vector (~1.045 MiBit).
  - Module-scope `pub var dst : [535][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w858.py`
  - Generator for the W858 witness; `OUTER = 535`, `MID_IDX = 267`.
  - Copy hazard fixed: destination path, module header f-string, and `MID_IDX`
    comment updated from stale `w857` / `533` / `266` references.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w858_bench_module_535x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w858_bench_module_535x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.

### Not changed

- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification

- `cargo build --release -p t27c`: OK (warnings, 0 errors).
- `cargo test --release --test icarus_lowerable accepts_w858_bench_module_535x2p6_aos_var_call_write`: 1/0.
- `cargo test --release --test icarus_lowerable` (full suite): 318/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W858: PASS.

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

## Wave Loop 859 — module-scope `[537][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-859`
- Parent branch: `wave-loop-858` HEAD (after closeout)
- Issue: #1658 (expected)
- PR: TBD
- Plan: `.claude/plans/wave-loop-859.md`
- Master plan: `.claude/skills/wave-loop-master-plan.md`

### Goal

Continue the odd outer-dimension module-scope AoS ladder with `[537][2]^6 Pt`.
Expected 34,368 elements, 1,099,776-bit packed vector (~1.049 MiBit), still well
under the 4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants

- **A (recommended):** `[537][2]^6 Pt` module-scope var from call.
- **B:** `[535][3]^6 Pt` — grow second inner dimension to stress stride scaling.
- **C:** `[535][2]^6 Pt` with negative-index writes to exercise wrap-around.

---

*φ² + φ⁻² = 3 | TRINITY*
